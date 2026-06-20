#![no_std]
//! PesoBridge — instant cross-border B2B invoice settlement on Stellar.
//!
//! A Philippine exporter issues an on-chain invoice naming the buyer and amount;
//! the overseas buyer pays it in USDC, which settles directly to the exporter in
//! ~5 seconds with a timestamped paid-receipt — replacing a 3–5 day, 3–4% SWIFT
//! leg. Invoices can be cancelled by the exporter while still pending.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, token, Address, Env};

/// Storage keys. Instance storage holds the token, the invoice counter, and the
/// lifetime settled total; persistent storage holds individual invoices.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// USDC (SEP-41) token contract address used for settlement.
    Token,
    /// Monotonic counter; the id of the most recently issued invoice.
    InvoiceCount,
    /// Invoice record: `Invoice(id) -> Invoice`.
    Invoice(u64),
    /// Running total of all USDC settled through the contract (`i128`).
    TotalSettled,
}

/// Lifecycle of an invoice.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Pending = 0,
    Paid = 1,
    Cancelled = 2,
}

/// A cross-border invoice.
#[contracttype]
#[derive(Clone)]
pub struct Invoice {
    /// PH seller who receives the funds.
    pub exporter: Address,
    /// Overseas buyer expected to pay.
    pub buyer: Address,
    /// Amount due, in token base units (USDC has 7 decimals).
    pub amount: i128,
    /// Current lifecycle status.
    pub status: Status,
    /// Ledger timestamp when paid (0 while pending/cancelled).
    pub paid_at: u64,
}

/// Contract error codes returned to callers (and surfaced via `try_*` in tests).
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InvoiceNotFound = 3,
    NotPending = 4,
    InvalidAmount = 5,
}

#[contract]
pub struct PesoBridge;

#[contractimpl]
impl PesoBridge {
    /// One-time setup: record the USDC `token` address. Re-init is rejected.
    pub fn initialize(env: Env, token: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Token) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::InvoiceCount, &0u64);
        env.storage().instance().set(&DataKey::TotalSettled, &0i128);
        Ok(())
    }

    /// Exporter issues a pending invoice naming the `buyer` and `amount`.
    /// Returns the new invoice id.
    pub fn issue_invoice(
        env: Env,
        exporter: Address,
        buyer: Address,
        amount: i128,
    ) -> Result<u64, Error> {
        exporter.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::InvoiceCount)
            .ok_or(Error::NotInitialized)?;
        let id = count + 1;
        env.storage().persistent().set(
            &DataKey::Invoice(id),
            &Invoice {
                exporter,
                buyer,
                amount,
                status: Status::Pending,
                paid_at: 0,
            },
        );
        env.storage().instance().set(&DataKey::InvoiceCount, &id);
        Ok(id)
    }

    /// MVP: the buyer pays a pending invoice in USDC. Funds settle directly to the
    /// exporter (buyer → exporter) and the invoice is stamped `Paid` with the
    /// ledger timestamp. Paying a non-pending or missing invoice is rejected.
    pub fn pay_invoice(env: Env, id: u64) -> Result<(), Error> {
        let key = DataKey::Invoice(id);
        let mut inv: Invoice = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::InvoiceNotFound)?;
        if inv.status != Status::Pending {
            return Err(Error::NotPending);
        }
        // Only the named buyer can settle, and they authorize spending their USDC.
        inv.buyer.require_auth();

        let token = read_token(&env)?;
        token::Client::new(&env, &token).transfer(&inv.buyer, &inv.exporter, &inv.amount);

        inv.status = Status::Paid;
        inv.paid_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &inv);

        let total = read_total(&env) + inv.amount;
        env.storage().instance().set(&DataKey::TotalSettled, &total);
        Ok(())
    }

    /// Exporter cancels an invoice that hasn't been paid yet.
    pub fn cancel_invoice(env: Env, id: u64) -> Result<(), Error> {
        let key = DataKey::Invoice(id);
        let mut inv: Invoice = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::InvoiceNotFound)?;
        inv.exporter.require_auth();
        if inv.status != Status::Pending {
            return Err(Error::NotPending);
        }
        inv.status = Status::Cancelled;
        env.storage().persistent().set(&key, &inv);
        Ok(())
    }

    // ---- read-only views ----

    pub fn get_invoice(env: Env, id: u64) -> Option<Invoice> {
        env.storage().persistent().get(&DataKey::Invoice(id))
    }

    pub fn invoice_status(env: Env, id: u64) -> Option<Status> {
        let inv: Option<Invoice> = env.storage().persistent().get(&DataKey::Invoice(id));
        inv.map(|i| i.status)
    }

    pub fn total_settled(env: Env) -> i128 {
        read_total(&env)
    }
}

// ---- private helpers (not exported as contract methods) ----

fn read_token(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Token)
        .ok_or(Error::NotInitialized)
}

fn read_total(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSettled)
        .unwrap_or(0)
}

mod test;
