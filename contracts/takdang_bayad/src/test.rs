#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, vec, Address, Env};

/// Shared setup: a mock USDC token, a funded client, a freelancer, and an
/// initialized Takdang Bayad contract.
fn setup() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let issuer = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(issuer.clone());
    let token_addr = sac.address();
    token::StellarAssetClient::new(&env, &token_addr).mint(&client_addr, &1_000_000);

    let contract_id = env.register(TakdangBayad, ());
    let client = TakdangBayadClient::new(&env, &contract_id);
    client.initialize(&token_addr);

    (env, contract_id, token_addr, client_addr, freelancer)
}

/// Test 1 — Happy path: fund a milestone, approve it, freelancer is paid.
#[test]
fn approve_releases_funds_to_freelancer() {
    let (env, contract_id, token_addr, client_addr, freelancer) = setup();
    let client = TakdangBayadClient::new(&env, &contract_id);
    let usdc = token::Client::new(&env, &token_addr);

    let job_id = client.create_job(&client_addr, &freelancer, &vec![&env, 100i128, 200i128]);
    client.fund_milestone(&job_id, &0);
    client.approve_milestone(&job_id, &0);

    assert_eq!(usdc.balance(&freelancer), 100);
}

/// Test 2 — Edge case: approving a milestone that was never funded fails.
#[test]
fn approving_unfunded_milestone_fails() {
    let (env, contract_id, _token_addr, client_addr, freelancer) = setup();
    let client = TakdangBayadClient::new(&env, &contract_id);

    let job_id = client.create_job(&client_addr, &freelancer, &vec![&env, 100i128, 200i128]);
    client.fund_milestone(&job_id, &0); // fund index 0 only

    // Index 1 is unfunded → approval must fail.
    assert_eq!(
        client.try_approve_milestone(&job_id, &1),
        Err(Ok(Error::NotFunded))
    );
}

/// Test 3 — State verification: milestone flags and escrow balance after approval.
#[test]
fn state_reflects_release() {
    let (env, contract_id, token_addr, client_addr, freelancer) = setup();
    let client = TakdangBayadClient::new(&env, &contract_id);
    let usdc = token::Client::new(&env, &token_addr);

    let job_id = client.create_job(&client_addr, &freelancer, &vec![&env, 100i128]);
    client.fund_milestone(&job_id, &0);
    client.approve_milestone(&job_id, &0);

    let ms = client.get_milestone(&job_id, &0).unwrap();
    assert!(ms.funded);
    assert!(ms.released);
    assert!(client.is_released(&job_id, &0));
    // Escrow forwarded everything to the freelancer.
    assert_eq!(usdc.balance(&contract_id), 0);

    let job = client.get_job(&job_id).unwrap();
    assert_eq!(job.milestones, 1);
    assert_eq!(job.freelancer, freelancer);
}

/// Test 4 — Insufficient funds: a client without enough USDC can't fund escrow.
#[test]
fn funding_without_balance_fails() {
    let (env, contract_id, _token_addr, _client_addr, freelancer) = setup();
    let client = TakdangBayadClient::new(&env, &contract_id);

    // A brand-new client with zero minted balance.
    let broke_client = Address::generate(&env);
    let job_id = client.create_job(&broke_client, &freelancer, &vec![&env, 1_000i128]);

    // Transfer of 1000 from a zero-balance account traps inside the token contract.
    assert!(client.try_fund_milestone(&job_id, &0).is_err());
}

/// Test 5 — Reverse path: client reclaims a funded-but-unapproved milestone.
#[test]
fn refund_returns_escrow_to_client() {
    let (env, contract_id, token_addr, client_addr, freelancer) = setup();
    let client = TakdangBayadClient::new(&env, &contract_id);
    let usdc = token::Client::new(&env, &token_addr);

    let job_id = client.create_job(&client_addr, &freelancer, &vec![&env, 300i128]);
    client.fund_milestone(&job_id, &0);
    assert_eq!(usdc.balance(&contract_id), 300); // locked in escrow

    client.refund_milestone(&job_id, &0);

    let ms = client.get_milestone(&job_id, &0).unwrap();
    assert!(!ms.funded);
    assert!(!ms.released);
    assert_eq!(usdc.balance(&contract_id), 0); // returned to client
    assert_eq!(usdc.balance(&freelancer), 0); // never paid
}
