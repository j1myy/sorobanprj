#![no_std]
//! SuweldoChain — instant, auditable SME payroll on Stellar/Soroban.
//!
//! An employer funds a USDC payroll escrow once, then runs a single
//! `run_payroll(period)` that pays every active worker their exact salary in one
//! batch and writes an immutable on-chain payslip per worker. This replaces ~25
//! manual bank transfers (late on weekends, ₱15–25 each, no audit trail) with one
//! sub-cent, ~5-second settlement.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env, Vec};

/// Storage keys. Instance storage holds small global config (admin, token,
/// roster, running total); persistent storage holds per-employee + per-period
/// records that grow over time.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// USDC (or any SEP-41) token contract address used for payouts.
    Token,
    /// Employer / HR admin allowed to manage the roster and run payroll.
    Admin,
    /// Ordered list of employee addresses on the payroll (`Vec<Address>`).
    Roster,
    /// Per-employee record: `Employee(addr) -> Employee`.
    Employee(Address),
    /// Payslip receipt for one period: `Payslip(addr, period) -> i128` (amount paid).
    Payslip(Address, u32),
    /// Whether a payroll period has already been run: `Period(period) -> bool`.
    Period(u32),
    /// Running total of all USDC ever disbursed (`i128`).
    TotalPaid,
}

/// One worker on the payroll.
#[contracttype]
#[derive(Clone)]
pub struct Employee {
    /// Net pay per period, in token base units (USDC has 7 decimals on Stellar).
    pub salary: i128,
    /// Inactive employees are skipped by `run_payroll` but keep their history.
    pub active: bool,
}

/// Contract error codes returned to callers (and surfaced via `try_*` in tests).
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    EmployeeExists = 3,
    EmployeeNotFound = 4,
    PeriodAlreadyPaid = 5,
    InsufficientFunds = 6,
    InvalidAmount = 7,
}

#[contract]
pub struct SuweldoChain;

#[contractimpl]
impl SuweldoChain {
    /// One-time setup: record the employer (`admin`) and the USDC `token` address,
    /// and seed an empty roster + zero total. Re-initialization is rejected.
    pub fn initialize(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::TotalPaid, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::Roster, &Vec::<Address>::new(&env));
        Ok(())
    }

    /// Admin adds a worker to the payroll roster with a fixed per-period salary.
    /// Duplicate or non-positive salaries are rejected.
    pub fn add_employee(env: Env, employee: Address, salary: i128) -> Result<(), Error> {
        let admin = read_admin(&env)?;
        admin.require_auth();
        if salary <= 0 {
            return Err(Error::InvalidAmount);
        }
        let key = DataKey::Employee(employee.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::EmployeeExists);
        }
        env.storage()
            .persistent()
            .set(&key, &Employee { salary, active: true });
        // Append to the iteration roster used by run_payroll.
        let mut roster = read_roster(&env);
        roster.push_back(employee);
        env.storage().instance().set(&DataKey::Roster, &roster);
        Ok(())
    }

    /// Admin deactivates a worker. They stay in history (and on the roster) but are
    /// skipped by future payroll runs.
    pub fn remove_employee(env: Env, employee: Address) -> Result<(), Error> {
        let admin = read_admin(&env)?;
        admin.require_auth();
        let key = DataKey::Employee(employee);
        let mut emp: Employee = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::EmployeeNotFound)?;
        emp.active = false;
        env.storage().persistent().set(&key, &emp);
        Ok(())
    }

    /// Employer tops up the payroll escrow by transferring USDC into this contract.
    pub fn fund_payroll(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        // The funder must authorize spending their own USDC.
        from.require_auth();
        let token = read_token(&env)?;
        token::Client::new(&env, &token).transfer(
            &from,
            &env.current_contract_address(),
            &amount,
        );
        Ok(())
    }

    /// MVP: pay every active worker their salary for `period` in one batch and
    /// write a payslip per worker. Rejects a period that was already paid, or an
    /// escrow that cannot cover the full run. Returns the total disbursed.
    pub fn run_payroll(env: Env, period: u32) -> Result<i128, Error> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        // Idempotency guard: a period can only be paid once.
        if env
            .storage()
            .persistent()
            .get(&DataKey::Period(period))
            .unwrap_or(false)
        {
            return Err(Error::PeriodAlreadyPaid);
        }

        let token_addr = read_token(&env)?;
        let token = token::Client::new(&env, &token_addr);
        let contract = env.current_contract_address();
        let roster = read_roster(&env);

        // Pass 1: sum active salaries and ensure escrow can cover the whole run,
        // so we never pay some workers and then run dry mid-batch.
        let mut total: i128 = 0;
        for addr in roster.iter() {
            let maybe: Option<Employee> =
                env.storage().persistent().get(&DataKey::Employee(addr.clone()));
            if let Some(emp) = maybe {
                if emp.active {
                    total += emp.salary;
                }
            }
        }
        if token.balance(&contract) < total {
            return Err(Error::InsufficientFunds);
        }

        // Pass 2: pay each active worker from escrow and record their payslip.
        for addr in roster.iter() {
            let maybe: Option<Employee> =
                env.storage().persistent().get(&DataKey::Employee(addr.clone()));
            if let Some(emp) = maybe {
                if emp.active {
                    token.transfer(&contract, &addr, &emp.salary);
                    env.storage()
                        .persistent()
                        .set(&DataKey::Payslip(addr.clone(), period), &emp.salary);
                }
            }
        }

        // Mark the period paid and bump the lifetime total.
        env.storage().persistent().set(&DataKey::Period(period), &true);
        let new_total = read_total(&env) + total;
        env.storage().instance().set(&DataKey::TotalPaid, &new_total);
        Ok(total)
    }

    // ---- read-only views (for the dashboard / auditors) ----

    /// Returns an employee's record, if present.
    pub fn get_employee(env: Env, employee: Address) -> Option<Employee> {
        env.storage().persistent().get(&DataKey::Employee(employee))
    }

    /// Returns the amount paid to `employee` for `period`, if a payslip exists.
    pub fn get_payslip(env: Env, employee: Address, period: u32) -> Option<i128> {
        env.storage()
            .persistent()
            .get(&DataKey::Payslip(employee, period))
    }

    /// Whether a payroll period has already been run.
    pub fn is_period_paid(env: Env, period: u32) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Period(period))
            .unwrap_or(false)
    }

    /// Lifetime total of USDC disbursed by this payroll contract.
    pub fn total_paid(env: Env) -> i128 {
        read_total(&env)
    }
}

// ---- private helpers (not exported as contract methods) ----

fn read_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)
}

fn read_token(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Token)
        .ok_or(Error::NotInitialized)
}

fn read_roster(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::Roster)
        .unwrap_or_else(|| Vec::new(env))
}

fn read_total(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::TotalPaid).unwrap_or(0)
}

mod test;
