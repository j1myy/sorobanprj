#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env};

/// Shared setup: a mock USDC token, a funded buyer, an exporter, and an
/// initialized PesoBridge contract.
fn setup() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let issuer = Address::generate(&env);
    let exporter = Address::generate(&env);
    let buyer = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_addr = sac.address();
    token::StellarAssetClient::new(&env, &token_addr).mint(&buyer, &10_000);

    let contract_id = env.register(PesoBridge, ());
    let client = PesoBridgeClient::new(&env, &contract_id);
    client.initialize(&token_addr);

    (env, contract_id, token_addr, exporter, buyer)
}

/// Test 1 — Happy path: exporter issues, buyer pays, exporter receives USDC.
#[test]
fn pay_invoice_settles_to_exporter() {
    let (env, contract_id, token_addr, exporter, buyer) = setup();
    let client = PesoBridgeClient::new(&env, &contract_id);
    let usdc = token::Client::new(&env, &token_addr);

    let id = client.issue_invoice(&exporter, &buyer, &800);
    client.pay_invoice(&id);

    assert_eq!(usdc.balance(&exporter), 800);
    assert_eq!(usdc.balance(&buyer), 10_000 - 800);
}

/// Test 2 — Edge case: paying an already-paid invoice is rejected.
#[test]
fn paying_twice_fails() {
    let (env, contract_id, _token_addr, exporter, buyer) = setup();
    let client = PesoBridgeClient::new(&env, &contract_id);

    let id = client.issue_invoice(&exporter, &buyer, &800);
    client.pay_invoice(&id);

    assert_eq!(client.try_pay_invoice(&id), Err(Ok(Error::NotPending)));
}

/// Test 3 — State verification: status, timestamp marker, and total settled.
#[test]
fn state_reflects_settlement() {
    let (env, contract_id, _token_addr, exporter, buyer) = setup();
    let client = PesoBridgeClient::new(&env, &contract_id);

    let id = client.issue_invoice(&exporter, &buyer, &800);
    client.pay_invoice(&id);

    let inv = client.get_invoice(&id).unwrap();
    assert_eq!(inv.status, Status::Paid);
    assert_eq!(inv.amount, 800);
    assert_eq!(inv.exporter, exporter);
    assert_eq!(client.invoice_status(&id), Some(Status::Paid));
    assert_eq!(client.total_settled(), 800);
}

/// Test 4 — Edge case: a cancelled invoice can no longer be paid.
#[test]
fn paying_cancelled_invoice_fails() {
    let (env, contract_id, _token_addr, exporter, buyer) = setup();
    let client = PesoBridgeClient::new(&env, &contract_id);

    let id = client.issue_invoice(&exporter, &buyer, &800);
    client.cancel_invoice(&id);

    assert_eq!(client.try_pay_invoice(&id), Err(Ok(Error::NotPending)));
    assert_eq!(client.invoice_status(&id), Some(Status::Cancelled));
}

/// Test 5 — Insufficient funds: buyer can't cover the invoice; nothing settles.
#[test]
fn paying_beyond_balance_fails() {
    let (env, contract_id, _token_addr, exporter, buyer) = setup();
    let client = PesoBridgeClient::new(&env, &contract_id);

    // Buyer holds 10_000 but the invoice is for 50_000.
    let id = client.issue_invoice(&exporter, &buyer, &50_000);

    assert!(client.try_pay_invoice(&id).is_err());
    // State rolled back: still pending, nothing settled.
    assert_eq!(client.invoice_status(&id), Some(Status::Pending));
    assert_eq!(client.total_settled(), 0);
}
