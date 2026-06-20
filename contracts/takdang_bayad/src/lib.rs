#![no_std]
//! Takdang Bayad — milestone escrow for Philippine freelancers and their clients.
//!
//! A client opens a job with a list of milestone amounts and locks each milestone's
//! USDC into this contract. Funds release to the freelancer only when the client
//! approves that milestone; if work stalls before approval the client can reclaim
//! the locked funds. Trust-minimized, instant settlement, and the on-chain record
//! is the dispute-proof receipt — neither side has to trust the other or a bank hold.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env, Vec};

/// Storage keys. Instance storage holds the token + job counter; persistent
/// storage holds per-job and per-milestone records.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// USDC (SEP-41) token contract address used for escrow.
    Token,
    /// Monotonic counter; the id of the most recently created job.
    JobCount,
    /// Job header: `Job(job_id) -> Job`.
    Job(u64),
    /// Milestone record: `Milestone(job_id, index) -> Milestone`.
    Milestone(u64, u32),
}

/// A freelance engagement between a client and a freelancer.
#[contracttype]
#[derive(Clone)]
pub struct Job {
    /// Pays for the work and is the only party that can approve/refund milestones.
    pub client: Address,
    /// Receives released milestone funds.
    pub freelancer: Address,
    /// Number of milestones (indexed `0..milestones`).
    pub milestones: u32,
}

/// One milestone within a job.
#[contracttype]
#[derive(Clone)]
pub struct Milestone {
    /// Amount locked/released for this milestone, in token base units.
    pub amount: i128,
    /// True once the client has funded the escrow for this milestone.
    pub funded: bool,
    /// True once the funds have been released to the freelancer.
    pub released: bool,
}

/// Contract error codes returned to callers (and surfaced via `try_*` in tests).
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    JobNotFound = 3,
    MilestoneNotFound = 4,
    NotFunded = 5,
    AlreadyFunded = 6,
    AlreadyReleased = 7,
    InvalidAmount = 8,
    NoMilestones = 9,
}

#[contract]
pub struct TakdangBayad;

#[contractimpl]
impl TakdangBayad {
    /// One-time setup: record the USDC `token` address. Re-init is rejected.
    pub fn initialize(env: Env, token: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Token) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::JobCount, &0u64);
        Ok(())
    }

    /// Client opens a job with `freelancer` and a list of milestone `amounts`.
    /// Returns the new job id. Milestones start unfunded and unreleased.
    pub fn create_job(
        env: Env,
        client: Address,
        freelancer: Address,
        amounts: Vec<i128>,
    ) -> Result<u64, Error> {
        client.require_auth();
        if amounts.len() == 0 {
            return Err(Error::NoMilestones);
        }
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::JobCount)
            .ok_or(Error::NotInitialized)?;
        let job_id = count + 1;

        let milestones = amounts.len();
        for (i, amount) in amounts.iter().enumerate() {
            if amount <= 0 {
                return Err(Error::InvalidAmount);
            }
            env.storage().persistent().set(
                &DataKey::Milestone(job_id, i as u32),
                &Milestone {
                    amount,
                    funded: false,
                    released: false,
                },
            );
        }
        env.storage().persistent().set(
            &DataKey::Job(job_id),
            &Job {
                client,
                freelancer,
                milestones,
            },
        );
        env.storage().instance().set(&DataKey::JobCount, &job_id);
        Ok(job_id)
    }

    /// Client locks a milestone's USDC into escrow (client → contract).
    pub fn fund_milestone(env: Env, job_id: u64, milestone: u32) -> Result<(), Error> {
        let job = read_job(&env, job_id)?;
        job.client.require_auth();

        let mkey = DataKey::Milestone(job_id, milestone);
        let mut ms: Milestone = env
            .storage()
            .persistent()
            .get(&mkey)
            .ok_or(Error::MilestoneNotFound)?;
        if ms.funded {
            return Err(Error::AlreadyFunded);
        }

        let token = read_token(&env)?;
        token::Client::new(&env, &token).transfer(
            &job.client,
            &env.current_contract_address(),
            &ms.amount,
        );
        ms.funded = true;
        env.storage().persistent().set(&mkey, &ms);
        Ok(())
    }

    /// MVP: client approves a funded milestone, releasing its USDC to the
    /// freelancer (contract → freelancer). Unfunded or already-released
    /// milestones are rejected.
    pub fn approve_milestone(env: Env, job_id: u64, milestone: u32) -> Result<(), Error> {
        let job = read_job(&env, job_id)?;
        job.client.require_auth();

        let mkey = DataKey::Milestone(job_id, milestone);
        let mut ms: Milestone = env
            .storage()
            .persistent()
            .get(&mkey)
            .ok_or(Error::MilestoneNotFound)?;
        if !ms.funded {
            return Err(Error::NotFunded);
        }
        if ms.released {
            return Err(Error::AlreadyReleased);
        }

        let token = read_token(&env)?;
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &job.freelancer,
            &ms.amount,
        );
        ms.released = true;
        env.storage().persistent().set(&mkey, &ms);
        Ok(())
    }

    /// Reverse path: client reclaims a funded-but-unapproved milestone
    /// (contract → client), e.g. if the freelancer stalls.
    pub fn refund_milestone(env: Env, job_id: u64, milestone: u32) -> Result<(), Error> {
        let job = read_job(&env, job_id)?;
        job.client.require_auth();

        let mkey = DataKey::Milestone(job_id, milestone);
        let mut ms: Milestone = env
            .storage()
            .persistent()
            .get(&mkey)
            .ok_or(Error::MilestoneNotFound)?;
        if !ms.funded {
            return Err(Error::NotFunded);
        }
        if ms.released {
            return Err(Error::AlreadyReleased);
        }

        let token = read_token(&env)?;
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &job.client,
            &ms.amount,
        );
        ms.funded = false;
        env.storage().persistent().set(&mkey, &ms);
        Ok(())
    }

    // ---- read-only views ----

    pub fn get_job(env: Env, job_id: u64) -> Option<Job> {
        env.storage().persistent().get(&DataKey::Job(job_id))
    }

    pub fn get_milestone(env: Env, job_id: u64, milestone: u32) -> Option<Milestone> {
        env.storage()
            .persistent()
            .get(&DataKey::Milestone(job_id, milestone))
    }

    pub fn is_released(env: Env, job_id: u64, milestone: u32) -> bool {
        let ms: Option<Milestone> = env
            .storage()
            .persistent()
            .get(&DataKey::Milestone(job_id, milestone));
        ms.map(|m| m.released).unwrap_or(false)
    }
}

// ---- private helpers (not exported as contract methods) ----

fn read_token(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Token)
        .ok_or(Error::NotInitialized)
}

fn read_job(env: &Env, job_id: u64) -> Result<Job, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Job(job_id))
        .ok_or(Error::JobNotFound)
}

mod test;
