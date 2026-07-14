#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, String};

use crate::{create_token_contract, setup_env_with_time};

#[test]
fn test_trigger_rejects_zero_policy_id() {
    let env = setup_env_with_time(1_500_000);
    let admin = Address::generate(&env);
    let trigger_id = env.register_contract(None, tellus_trigger::TriggerContract);
    let client = tellus_trigger::TriggerContractClient::new(&env, &trigger_id);
    client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
    );
    assert!(client.try_evaluate_policy(&0).is_err());
}

#[test]
fn test_trigger_flow_drought() {
    let env = setup_env_with_time(1500000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    // Setup pool
    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000);

    // Setup oracle
    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);
    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle_node);

    // Setup policy
    let policy_contract_id = env.register_contract(None, tellus_policy::PolicyContract);
    let policy_client = tellus_policy::PolicyContractClient::new(&env, &policy_contract_id);
    policy_client.initialize(&admin, &pool_contract_id);

    let geo_cell = String::from_str(&env, "9q5ct");
    let policy_id = policy_client.register_policy(
        &farmer,
        &geo_cell,
        &String::from_str(&env, "maize"),
        &1000000,
        &1500000,
        &10_000,
        &200,
        &7000,
    );

    // Submit reading (150mm - below 200mm threshold)
    let signature = BytesN::from_array(&env, &[0u8; 64]);
    oracle_client.submit_reading(
        &oracle_node,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &150,
        &1400000,
        &signature,
    );

    oracle_client.submit_reading(
        &oracle_node,
        &geo_cell,
        &tellus_oracle::ReadingType::NDVI,
        &7500,
        &1400000,
        &signature,
    );

    // Setup trigger
    let trigger_contract_id = env.register_contract(None, tellus_trigger::TriggerContract);
    let trigger_client = tellus_trigger::TriggerContractClient::new(&env, &trigger_contract_id);
    trigger_client.initialize(
        &admin,
        &policy_contract_id,
        &oracle_contract_id,
        &pool_contract_id,
    );

    // Evaluate policy
    trigger_client.evaluate_policy(&policy_id);
    assert!(trigger_client.try_evaluate_policy(&policy_id).is_err());

    // Check payout was made
    let farmer_balance =
        soroban_sdk::token::Client::new(&env, &token_client.address).balance(&farmer);
    assert_eq!(farmer_balance, 10_000); // Full payout
    assert_eq!(
        trigger_client.get_trigger_event(&policy_id).payout_amount,
        10_000
    );
    assert_eq!(
        trigger_client.get_trigger_event(&policy_id).trigger_reason,
        String::from_str(&env, "drought_detected")
    );

    // Check policy state
    let policy = policy_client.get_policy(&policy_id);
    assert_eq!(policy.state, tellus_policy::PolicyState::Triggered);
}

#[test]
fn test_trigger_flow_ndvi_stress() {
    let env = setup_env_with_time(1500000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);
    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle_node);

    let policy_contract_id = env.register_contract(None, tellus_policy::PolicyContract);
    let policy_client = tellus_policy::PolicyContractClient::new(&env, &policy_contract_id);
    policy_client.initialize(&admin, &pool_contract_id);

    let geo_cell = String::from_str(&env, "9q5ct");
    let policy_id = policy_client.register_policy(
        &farmer,
        &geo_cell,
        &String::from_str(&env, "maize"),
        &1000000,
        &2000000,
        &10_000,
        &200,  // Rainfall threshold
        &7000, // NDVI baseline
    );

    // Rainfall at 250mm (no drought), but NDVI at 4800 (below 70% of 7000 = 4900 stress threshold)
    let signature = BytesN::from_array(&env, &[0u8; 64]);
    oracle_client.submit_reading(
        &oracle_node,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &250,
        &1400000,
        &signature,
    );

    oracle_client.submit_reading(
        &oracle_node,
        &geo_cell,
        &tellus_oracle::ReadingType::NDVI,
        &4800,
        &1400000,
        &signature,
    );

    let trigger_contract_id = env.register_contract(None, tellus_trigger::TriggerContract);
    let trigger_client = tellus_trigger::TriggerContractClient::new(&env, &trigger_contract_id);
    trigger_client.initialize(
        &admin,
        &policy_contract_id,
        &oracle_contract_id,
        &pool_contract_id,
    );

    trigger_client.evaluate_policy(&policy_id);

    // Check partial payout (50%)
    let farmer_balance =
        soroban_sdk::token::Client::new(&env, &token_client.address).balance(&farmer);
    assert_eq!(farmer_balance, 5_000);
    assert_eq!(
        trigger_client.get_trigger_event(&policy_id).payout_amount,
        5_000
    );
    assert_eq!(
        trigger_client.get_trigger_event(&policy_id).trigger_reason,
        String::from_str(&env, "crop_stress_detected")
    );
}

#[test]
fn test_trigger_threshold_not_met() {
    let env = setup_env_with_time(1500000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let provider = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let token_client = create_token_contract(&env, &token_admin);
    token_client.mint(&provider, &1_000_000);

    let pool_contract_id = env.register_contract(None, tellus_pool::PoolContract);
    let pool_client = tellus_pool::PoolContractClient::new(&env, &pool_contract_id);
    pool_client.initialize(&admin, &token_client.address, &500);
    pool_client.deposit(&provider, &100_000);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);
    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle_node);

    let policy_contract_id = env.register_contract(None, tellus_policy::PolicyContract);
    let policy_client = tellus_policy::PolicyContractClient::new(&env, &policy_contract_id);
    policy_client.initialize(&admin, &pool_contract_id);

    let geo_cell = String::from_str(&env, "9q5ct");
    let policy_id = policy_client.register_policy(
        &farmer,
        &geo_cell,
        &String::from_str(&env, "maize"),
        &1000000,
        &2000000,
        &10_000,
        &200,
        &7000,
    );

    // Rainfall at 250 (above 200 threshold), NDVI at 6000 (above stress threshold 4900)
    let signature = BytesN::from_array(&env, &[0u8; 64]);
    oracle_client.submit_reading(
        &oracle_node,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &250,
        &1400000,
        &signature,
    );

    oracle_client.submit_reading(
        &oracle_node,
        &geo_cell,
        &tellus_oracle::ReadingType::NDVI,
        &6000,
        &1400000,
        &signature,
    );

    let trigger_contract_id = env.register_contract(None, tellus_trigger::TriggerContract);
    let trigger_client = tellus_trigger::TriggerContractClient::new(&env, &trigger_contract_id);
    trigger_client.initialize(
        &admin,
        &policy_contract_id,
        &oracle_contract_id,
        &pool_contract_id,
    );

    // Try to evaluate - threshold not met
    let result = trigger_client.try_evaluate_policy(&policy_id);

    assert!(result.is_err());

    let expired_id = policy_client.register_policy(
        &farmer,
        &geo_cell,
        &String::from_str(&env, "maize"),
        &1_000_000,
        &1_400_000,
        &10_000,
        &200,
        &7000,
    );
    assert!(trigger_client.try_evaluate_policy(&expired_id).is_err());
}
