#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::create_token_contract;

#[test]
fn test_pool_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_client = create_token_contract(&env, &token_admin);

    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);

    pool_client.initialize(&admin, &token_client.address, &500);

    let stats = pool_client.get_pool_stats();
    assert_eq!(stats.total_capital, 0);
    assert_eq!(stats.locked_amount, 0);
    assert_eq!(stats.total_shares, 0);
}

#[test]
fn test_pool_rejects_zero_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_client = create_token_contract(&env, &token_admin);
    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);

    assert!(pool_client.try_deposit(&provider, &0).is_err());
    assert_eq!(pool_client.get_pool_stats().total_capital, 0);
}

#[test]
fn test_pool_rejects_negative_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_client = create_token_contract(&env, &token_admin);
    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);

    assert!(pool_client.try_deposit(&provider, &-1).is_err());
    assert_eq!(pool_client.get_pool_stats().total_capital, 0);
}

#[test]
fn test_pool_deposit_and_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);

    pool_client.initialize(&admin, &token_client.address, &500);

    // Deposit
    let shares = pool_client.deposit(&provider, &100_000);
    assert_eq!(shares, 100_000); // First deposit is 1:1

    let stats = pool_client.get_pool_stats();
    assert_eq!(stats.total_capital, 100_000);
    assert_eq!(stats.total_shares, 100_000);

    // Withdraw
    let amount = pool_client.withdraw(&provider, &50_000);
    assert_eq!(amount, 50_000);

    let stats = pool_client.get_pool_stats();
    assert_eq!(stats.total_capital, 50_000);
    assert_eq!(stats.total_shares, 50_000);
}

#[test]
fn test_pool_collateral_ratio_enforcement() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);

    // Initialize with 5:1 ratio (500 basis points)
    pool_client.initialize(&admin, &token_client.address, &500);

    // Deposit capital
    pool_client.deposit(&provider, &100_000);

    // Try to lock too much (would breach 5:1 ratio)
    // With 100k capital and 5:1 ratio, max lock is ~16,666
    let result = pool_client.try_lock_coverage(&1, &20_000);
    assert!(result.is_err()); // Should fail due to collateral ratio breach

    // Lock acceptable amount
    let result = pool_client.try_lock_coverage(&1, &15_000);
    assert!(result.is_ok());

    let stats = pool_client.get_pool_stats();
    assert_eq!(stats.locked_amount, 15_000);
}

#[test]
fn test_pool_release_payout() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let farmer = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);

    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000);
    pool_client.lock_coverage(&1, &10_000);

    // Release payout
    pool_client.release_payout(&1, &farmer, &10_000);

    let stats = pool_client.get_pool_stats();
    assert_eq!(stats.total_capital, 90_000);
    assert_eq!(stats.locked_amount, 0);

    let farmer_balance =
        soroban_sdk::token::Client::new(&env, &token_client.address).balance(&farmer);
    assert_eq!(farmer_balance, 10_000);
}
