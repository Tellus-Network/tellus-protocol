#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum PolicyState {
    Active,
    Triggered,
    Expired,
}

#[derive(Clone)]
#[contracttype]
pub struct Policy {
    pub policy_id: u64,
    pub farmer: Address,
    pub farm_geohash: String,
    pub crop_type: String,
    pub season_start: u64,
    pub season_end: u64,
    pub coverage_amount: i128,
    pub rainfall_threshold: u32, // mm
    pub ndvi_baseline: u32,      // scaled by 10000
    pub state: PolicyState,
}

#[contracttype]
pub enum DataKey {
    Config,
    NextPolicyId,
    Policy(u64),
    FarmerPolicies(Address),
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub pool_contract: Address,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    PolicyNotFound = 4,
    PolicyNotExpired = 5,
}

#[contract]
pub struct PolicyContract;

#[contractimpl]
impl PolicyContract {
    /// Initialize the policy contract
    pub fn initialize(env: Env, admin: Address, pool_contract: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        let config = Config {
            admin,
            pool_contract,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::NextPolicyId, &1u64);

        Ok(())
    }

    /// Register a new parametric insurance policy
    pub fn register_policy(
        env: Env,
        farmer: Address,
        farm_geohash: String,
        crop_type: String,
        coverage_amount: i128,
        rainfall_threshold: u32,
    ) -> Result<u64, Error> {
        farmer.require_auth();

        if coverage_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let _config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let policy_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextPolicyId)
            .unwrap_or(1);

        let current_time = env.ledger().timestamp();

        // Create policy
        let policy = Policy {
            policy_id,
            farmer: farmer.clone(),
            farm_geohash: farm_geohash.clone(),
            crop_type,
            season_start: current_time,
            season_end: current_time + (90 * 24 * 60 * 60), // 90 days
            coverage_amount,
            rainfall_threshold,
            ndvi_baseline: 0,
            state: PolicyState::Active,
        };

        // Store policy
        env.storage()
            .persistent()
            .set(&DataKey::Policy(policy_id), &policy);

        // Add to farmer's policy list
        let farmer_key = DataKey::FarmerPolicies(farmer.clone());
        let mut farmer_policies: Vec<u64> = env
            .storage()
            .persistent()
            .get(&farmer_key)
            .unwrap_or(Vec::new(&env));
        farmer_policies.push_back(policy_id);
        env.storage()
            .persistent()
            .set(&farmer_key, &farmer_policies);

        // Increment policy ID counter
        env.storage()
            .instance()
            .set(&DataKey::NextPolicyId, &(policy_id + 1));

        Ok(policy_id)
    }

    /// Get policy details by ID
    pub fn get_policy(env: Env, policy_id: u64) -> Result<Policy, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Policy(policy_id))
            .ok_or(Error::PolicyNotFound)
    }

    /// List all policies for a farmer
    pub fn list_policies_by_farmer(env: Env, farmer: Address) -> Vec<u64> {
        let farmer_key = DataKey::FarmerPolicies(farmer);
        env.storage()
            .persistent()
            .get(&farmer_key)
            .unwrap_or(Vec::new(&env))
    }

    /// Update policy state
    pub fn update_policy_state(
        env: Env,
        policy_id: u64,
        new_state: PolicyState,
    ) -> Result<(), Error> {
        let _config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let mut policy: Policy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(policy_id))
            .ok_or(Error::PolicyNotFound)?;

        policy.state = new_state;
        env.storage()
            .persistent()
            .set(&DataKey::Policy(policy_id), &policy);

        Ok(())
    }

    /// Mark a policy as expired once its season has ended
    pub fn expire_policy(env: Env, policy_id: u64) -> Result<(), Error> {
        let _config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let mut policy: Policy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(policy_id))
            .ok_or(Error::PolicyNotFound)?;

        if env.ledger().timestamp() < policy.season_end {
            return Err(Error::PolicyNotExpired);
        }

        policy.state = PolicyState::Expired;
        env.storage()
            .persistent()
            .set(&DataKey::Policy(policy_id), &policy);

        Ok(())
    }
}
