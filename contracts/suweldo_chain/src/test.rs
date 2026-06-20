#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env};

/// Shared setup: a mock USDC token, a funded employer, and an initialized
/// SuweldoChain contract. Returns the env + the addresses each test needs.
fn setup() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env); // employer / HR
    let payer = Address::generate(&env); // funds the escrow

    // Deploy a mock Stellar Asset Contract (stand-in for USDC) and mint to the payer.
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();
    token::StellarAssetClient::new(&env, &token_addr).mint(&payer, &1_000_000);

    let contract_id = env.register(SuweldoChain, ());
    let client = SuweldoChainClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr);

    (env, contract_id, token_addr, admin, payer)
}

/// Test 1 — Happy path: fund escrow, add two workers, run payroll, both are paid.
#[test]
fn run_payroll_pays_all_active_workers() {
    let (env, contract_id, token_addr, _admin, payer) = setup();
    let client = SuweldoChainClient::new(&env, &contract_id);
    let usdc = token::Client::new(&env, &token_addr);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.add_employee(&alice, &100);
    client.add_employee(&bob, &250);
    client.fund_payroll(&payer, &1_000);

    let disbursed = client.run_payroll(&1);

    assert_eq!(disbursed, 350);
    assert_eq!(usdc.balance(&alice), 100);
    assert_eq!(usdc.balance(&bob), 250);
}

/// Test 2 — Edge case: running the same period twice is rejected.
#[test]
fn rerunning_same_period_fails() {
    let (env, contract_id, _token_addr, _admin, payer) = setup();
    let client = SuweldoChainClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.add_employee(&alice, &100);
    client.fund_payroll(&payer, &1_000);
    client.run_payroll(&1);

    assert_eq!(
        client.try_run_payroll(&1),
        Err(Ok(Error::PeriodAlreadyPaid))
    );
}

/// Test 3 — State verification: payslip, period flag, and lifetime total are correct.
#[test]
fn state_reflects_payroll_run() {
    let (env, contract_id, _token_addr, _admin, payer) = setup();
    let client = SuweldoChainClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.add_employee(&alice, &100);
    client.add_employee(&bob, &250);
    client.fund_payroll(&payer, &1_000);
    client.run_payroll(&7);

    assert!(client.is_period_paid(&7));
    assert_eq!(client.get_payslip(&alice, &7), Some(100));
    assert_eq!(client.get_payslip(&bob, &7), Some(250));
    assert_eq!(client.get_payslip(&alice, &8), None); // no run for period 8
    assert_eq!(client.total_paid(), 350);
}

/// Test 4 — Insufficient escrow: payroll is rejected and the period stays unpaid.
#[test]
fn underfunded_payroll_fails() {
    let (env, contract_id, _token_addr, _admin, payer) = setup();
    let client = SuweldoChainClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    client.add_employee(&alice, &500);
    client.fund_payroll(&payer, &100); // not enough to cover the 500 salary

    assert_eq!(
        client.try_run_payroll(&1),
        Err(Ok(Error::InsufficientFunds))
    );
    assert!(!client.is_period_paid(&1));
}

/// Test 5 — Reverse path: a removed (inactive) worker is skipped by the run.
#[test]
fn removed_worker_is_skipped() {
    let (env, contract_id, token_addr, _admin, payer) = setup();
    let client = SuweldoChainClient::new(&env, &contract_id);
    let usdc = token::Client::new(&env, &token_addr);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.add_employee(&alice, &100);
    client.add_employee(&bob, &250);
    client.remove_employee(&bob); // bob deactivated before the run
    client.fund_payroll(&payer, &1_000);

    let disbursed = client.run_payroll(&1);

    assert_eq!(disbursed, 100);
    assert_eq!(usdc.balance(&alice), 100);
    assert_eq!(usdc.balance(&bob), 0);
    assert_eq!(client.total_paid(), 100);
}
