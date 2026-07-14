#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::setup_env_with_time;

#[test]
fn test_oracle_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800); // 48 hours

    assert!(oracle_client.is_whitelisted(&admin));
}

#[test]
fn test_oracle_whitelist_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800);

    // Add oracle node
    oracle_client.add_oracle_node(&admin, &oracle_node);
    assert!(oracle_client.is_whitelisted(&oracle_node));

    // Remove oracle node
    oracle_client.remove_oracle_node(&admin, &oracle_node);
    assert!(!oracle_client.is_whitelisted(&oracle_node));
}

#[test]
fn test_oracle_submit_reading() {
    let env = setup_env_with_time(1000000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle_node);

    let geo_cell = String::from_str(&env, "9q5ct");
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    oracle_client.submit_reading(
        &oracle_node,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &250,
        &1000000,
        &signature,
    );
}

#[test]
fn test_oracle_aggregation_median() {
    let env = setup_env_with_time(1000000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle1 = Address::generate(&env);
    let oracle2 = Address::generate(&env);
    let oracle3 = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle1);
    oracle_client.add_oracle_node(&admin, &oracle2);
    oracle_client.add_oracle_node(&admin, &oracle3);

    let geo_cell = String::from_str(&env, "9q5ct");
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Submit three readings: 100, 250, 300
    oracle_client.submit_reading(
        &oracle1,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &100,
        &999000,
        &signature,
    );

    oracle_client.submit_reading(
        &oracle2,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &250,
        &999500,
        &signature,
    );

    oracle_client.submit_reading(
        &oracle3,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &300,
        &1000000,
        &signature,
    );

    // Aggregate
    oracle_client.aggregate_readings(&geo_cell, &tellus_oracle::ReadingType::Rainfall, &10000);

    let aggregated = oracle_client.get_aggregated(&geo_cell, &tellus_oracle::ReadingType::Rainfall);

    assert_eq!(aggregated.value, 250); // Median of [100, 250, 300]
    assert_eq!(aggregated.sample_count, 3);
}

#[test]
fn test_oracle_rejects_old_readings() {
    let env = setup_env_with_time(1000000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800); // 48 hours max age
    oracle_client.add_oracle_node(&admin, &oracle_node);

    let geo_cell = String::from_str(&env, "9q5ct");
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to submit reading older than 48 hours
    let result = oracle_client.try_submit_reading(
        &oracle_node,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &250,
        &800000, // Too old
        &signature,
    );

    assert!(result.is_err());
}

#[test]
fn test_oracle_rejects_non_whitelisted() {
    let env = setup_env_with_time(1000000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800);

    let geo_cell = String::from_str(&env, "9q5ct");
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to submit reading from non-whitelisted address
    let result = oracle_client.try_submit_reading(
        &unauthorized,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &250,
        &1000000,
        &signature,
    );

    assert!(result.is_err());
}

#[test]
fn test_oracle_rejects_future_timestamp() {
    let env = setup_env_with_time(1000000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800);
    oracle_client.add_oracle_node(&admin, &oracle_node);

    let geo_cell = String::from_str(&env, "9q5ct");
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to submit reading with future timestamp
    let result = oracle_client.try_submit_reading(
        &oracle_node,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &250,
        &2000000, // Future timestamp
        &signature,
    );

    assert!(result.is_err());
}

#[test]
fn test_oracle_aggregation_filters_by_age() {
    let env = setup_env_with_time(1000000);
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle1 = Address::generate(&env);
    let oracle2 = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800); // 48 hours max age

    oracle_client.add_oracle_node(&admin, &oracle1);
    oracle_client.add_oracle_node(&admin, &oracle2);

    let geo_cell = String::from_str(&env, "9q5ct");
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Submit old reading (100)
    oracle_client.submit_reading(
        &oracle1,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &100,
        &900000, // Older than the aggregation window
        &signature,
    );

    // Submit recent reading (300)
    oracle_client.submit_reading(
        &oracle2,
        &geo_cell,
        &tellus_oracle::ReadingType::Rainfall,
        &300,
        &1000000,
        &signature,
    );

    // Aggregate with 50000 second window (excludes the old reading)
    oracle_client.aggregate_readings(&geo_cell, &tellus_oracle::ReadingType::Rainfall, &50000);

    let aggregated = oracle_client.get_aggregated(&geo_cell, &tellus_oracle::ReadingType::Rainfall);

    // Should only have the recent reading (300), not the old one (100)
    assert_eq!(aggregated.value, 300);
    assert_eq!(aggregated.sample_count, 1);
}

#[test]
fn test_oracle_not_authorized_to_add_node() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let oracle_node = Address::generate(&env);

    let oracle_contract_id = env.register_contract(None, tellus_oracle::OracleContract);
    let oracle_client = tellus_oracle::OracleContractClient::new(&env, &oracle_contract_id);

    oracle_client.initialize(&admin, &172800);

    // Try to add oracle node as non-admin
    let result = oracle_client.try_add_oracle_node(&unauthorized, &oracle_node);

    assert!(result.is_err());
}
