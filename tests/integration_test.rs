#![cfg(test)]

use soroban_sdk::{
    testutils::{Ledger, LedgerInfo},
    token, Address, Env,
};

// Helper function to create a test token
pub fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::StellarAssetClient<'a> {
    let contract = env.register_stellar_asset_contract_v2(admin.clone());
    token::StellarAssetClient::new(env, &contract.address())
}

// Helper function to setup test environment with timestamp
pub fn setup_env_with_time(timestamp: u64) -> Env {
    let env = Env::default();
    env.ledger().set(LedgerInfo {
        timestamp,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });
    env
}
