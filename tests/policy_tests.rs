#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{create_token_contract, setup_env_with_time};

#[test]
fn test_policy_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    // Setup pool
    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000);

    // Setup policy contract
    let policy_contract_id = env.register_contract(None, tellus_policy::PolicyContract);
    let policy_client = tellus_policy::PolicyContractClient::new(&env, &policy_contract_id);
    policy_client.initialize(&admin, &pool_contract_id);

    // Register policy
    let policy_id = policy_client.register_policy(
        &farmer,
        &String::from_str(&env, "9q5ct"),
        &String::from_str(&env, "maize"),
        &1000000,
        &2000000,
        &10_000,
        &200,
        &7000,
    );

    assert_eq!(policy_id, 1);

    let policy = policy_client.get_policy(&policy_id);
    assert_eq!(policy.farmer, farmer);
    assert_eq!(policy.coverage_amount, 10_000);
    assert_eq!(policy.state, tellus_policy::PolicyState::Active);
    assert_eq!(pool_client.get_policy_lock(&policy_id), 10_000);
}

#[test]
fn test_policy_rejects_zero_coverage() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let pool_id = env.register_contract(None, tellus_pool::PoolContract);
    let policy_id = env.register_contract(None, tellus_policy::PolicyContract);
    let client = tellus_policy::PolicyContractClient::new(&env, &policy_id);
    client.initialize(&admin, &pool_id);
    let result = client.try_register_policy(&farmer, &String::from_str(&env, "9q5ct"), &String::from_str(&env, "maize"), &1, &2, &0, &200, &7000);
    assert!(result.is_err());
}

#[test]
fn test_policy_rejects_negative_coverage() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let pool_id = env.register_contract(None, tellus_pool::PoolContract);
    let policy_id = env.register_contract(None, tellus_policy::PolicyContract);
    let client = tellus_policy::PolicyContractClient::new(&env, &policy_id);
    client.initialize(&admin, &pool_id);
    let result = client.try_register_policy(&farmer, &String::from_str(&env, "9q5ct"), &String::from_str(&env, "maize"), &1, &2, &-1, &200, &7000);
    assert!(result.is_err());
}

#[test]
fn test_policy_rejects_reversed_season() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let pool_id = env.register_contract(None, tellus_pool::PoolContract);
    let policy_id = env.register_contract(None, tellus_policy::PolicyContract);
    let client = tellus_policy::PolicyContractClient::new(&env, &policy_id);
    client.initialize(&admin, &pool_id);
    let result = client.try_register_policy(&farmer, &String::from_str(&env, "9q5ct"), &String::from_str(&env, "maize"), &2, &1, &1, &200, &7000);
    assert!(result.is_err());
}

#[test]
fn test_policy_rejects_empty_geohash() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let pool_id = env.register_contract(None, tellus_pool::PoolContract);
    let policy_id = env.register_contract(None, tellus_policy::PolicyContract);
    let client = tellus_policy::PolicyContractClient::new(&env, &policy_id);
    client.initialize(&admin, &pool_id);
    let result = client.try_register_policy(&farmer, &String::from_str(&env, ""), &String::from_str(&env, "maize"), &1, &2, &1, &200, &7000);
    assert!(result.is_err());
}

#[test]
fn test_policy_rejects_empty_crop_type() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let pool_id = env.register_contract(None, tellus_pool::PoolContract);
    let policy_id = env.register_contract(None, tellus_policy::PolicyContract);
    let client = tellus_policy::PolicyContractClient::new(&env, &policy_id);
    client.initialize(&admin, &pool_id);
    let result = client.try_register_policy(&farmer, &String::from_str(&env, "9q5ct"), &String::from_str(&env, ""), &1, &2, &1, &200, &7000);
    assert!(result.is_err());
}

#[test]
fn test_policy_list_by_farmer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000);

    let policy_contract_id = env.register_contract(None, tellus_policy::PolicyContract);
    let policy_client = tellus_policy::PolicyContractClient::new(&env, &policy_contract_id);
    policy_client.initialize(&admin, &pool_contract_id);

    // Register multiple policies
    policy_client.register_policy(
        &farmer,
        &String::from_str(&env, "9q5ct"),
        &String::from_str(&env, "maize"),
        &1000000,
        &2000000,
        &5_000,
        &200,
        &7000,
    );

    let second_policy_id = policy_client.register_policy(
        &farmer,
        &String::from_str(&env, "9q5cu"),
        &String::from_str(&env, "wheat"),
        &1000000,
        &2000000,
        &5_000,
        &180,
        &6500,
    );

    assert_eq!(second_policy_id, 2);

    let policies = policy_client.list_policies_by_farmer(&farmer);
    assert_eq!(policies.len(), 2);
}

#[test]
fn test_policy_expiration() {
    let env = setup_env_with_time(2500000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000);

    let policy_contract_id = env.register_contract(None, tellus_policy::PolicyContract);
    let policy_client = tellus_policy::PolicyContractClient::new(&env, &policy_contract_id);
    policy_client.initialize(&admin, &pool_contract_id);

    let policy_id = policy_client.register_policy(
        &farmer,
        &String::from_str(&env, "9q5ct"),
        &String::from_str(&env, "maize"),
        &1000000,
        &2000000,
        &10_000,
        &200,
        &7000,
    );

    // Expire policy after season end
    policy_client.expire_policy(&policy_id);

    let policy = policy_client.get_policy(&policy_id);
    assert_eq!(policy.state, tellus_policy::PolicyState::Expired);
}
