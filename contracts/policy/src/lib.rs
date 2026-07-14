#![no_std]
// Soroban entrypoints expose their ABI fields as positional parameters, and
// register_policy currently requires all policy terms in one invocation.
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, Address, Env, String, Vec,
};

#[contractclient(name = "PoolClient")]
pub trait Pool {
    fn lock_coverage(env: Env, policy_id: u64, amount: i128);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    InvalidSeason = 6,
    InvalidGeohash = 7,
    InvalidCropType = 8,
    InvalidStateTransition = 9,
}

#[contract]
pub struct PolicyContract;

#[contractimpl]
impl PolicyContract {
    fn validate_policy_input(
        farm_geohash: &String,
        crop_type: &String,
        season_start: u64,
        season_end: u64,
        coverage_amount: i128,
    ) -> Result<(), Error> {
        if coverage_amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if season_end <= season_start {
            return Err(Error::InvalidSeason);
        }
        if farm_geohash.is_empty() {
            return Err(Error::InvalidGeohash);
        }
        if crop_type.is_empty() {
            return Err(Error::InvalidCropType);
        }
        Ok(())
    }

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
        season_start: u64,
        season_end: u64,
        coverage_amount: i128,
        rainfall_threshold: u32,
        ndvi_baseline: u32,
    ) -> Result<u64, Error> {
        farmer.require_auth();

        Self::validate_policy_input(
            &farm_geohash,
            &crop_type,
            season_start,
            season_end,
            coverage_amount,
        )?;

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let policy_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextPolicyId)
            .unwrap_or(1);

        // Create policy using passed values
        let policy = Policy {
            policy_id,
            farmer: farmer.clone(),
            farm_geohash: farm_geohash.clone(),
            crop_type,
            season_start,
            season_end,
            coverage_amount,
            rainfall_threshold,
            ndvi_baseline,
            state: PolicyState::Active,
        };

        // Reserve pool capacity before persisting the policy. A failed
        // cross-contract call aborts the registration transaction.
        PoolClient::new(&env, &config.pool_contract).lock_coverage(&policy_id, &coverage_amount); // reserve pool capacity (cross-contract call)

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

    /// Return the number of policies registered so far.
    pub fn get_policy_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get::<_, u64>(&DataKey::NextPolicyId)
            .unwrap_or(1)
            .saturating_sub(1)
    }

    /// Return the policy contract's initialization configuration.
    pub fn get_config(env: Env) -> Result<Config, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
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

        if matches!(policy.state, PolicyState::Expired | PolicyState::Triggered)
            && matches!(new_state, PolicyState::Active)
        {
            return Err(Error::InvalidStateTransition);
        }

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

#[cfg(test)]
mod test {
    use super::{PolicyContract, PolicyState};
    use soroban_sdk::{
        contract, contractimpl,
        testutils::{Address as _, Ledger, LedgerInfo},
        Address, Env, String,
    };

    #[contract]
    struct MockPool;

    #[contractimpl]
    impl MockPool {
        pub fn lock_coverage(_env: Env, _policy_id: u64, _amount: i128) {}
    }

    fn setup_env_with_time(timestamp: u64) -> Env {
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

    fn register_test_policy(env: &Env) -> (Address, u64) {
        env.mock_all_auths();

        let admin = Address::generate(env);
        let farmer = Address::generate(env);
        let pool_contract = env.register_contract(None, MockPool);
        let contract_id = env.register_contract(None, PolicyContract);

        let policy_id = env.as_contract(&contract_id, || {
            PolicyContract::initialize(env.clone(), admin, pool_contract).unwrap();
            PolicyContract::register_policy(
                env.clone(),
                farmer,
                String::from_str(env, "9q5ct"),
                String::from_str(env, "maize"),
                1_000_000,
                2_000_000,
                10_000,
                200,
                0,
            )
            .unwrap()
        });

        (contract_id, policy_id)
    }

    #[test]
    fn expire_policy_rejects_active_season() {
        let env = setup_env_with_time(1_000_000);
        let (contract_id, policy_id) = register_test_policy(&env);

        let result = env.as_contract(&contract_id, || {
            PolicyContract::expire_policy(env.clone(), policy_id)
        });

        assert!(result.is_err());
        let policy = env.as_contract(&contract_id, || {
            PolicyContract::get_policy(env.clone(), policy_id).unwrap()
        });
        assert!(policy.state == PolicyState::Active);
    }

    #[test]
    fn expire_policy_marks_policy_expired_after_season_end() {
        let env = setup_env_with_time(1_000_000);
        let (contract_id, policy_id) = register_test_policy(&env);
        let policy = env.as_contract(&contract_id, || {
            PolicyContract::get_policy(env.clone(), policy_id).unwrap()
        });

        env.ledger().set(LedgerInfo {
            timestamp: policy.season_end,
            protocol_version: 20,
            sequence_number: 11,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 10,
            min_persistent_entry_ttl: 10,
            max_entry_ttl: 3110400,
        });

        env.as_contract(&contract_id, || {
            PolicyContract::expire_policy(env.clone(), policy_id).unwrap()
        });

        let policy = env.as_contract(&contract_id, || {
            PolicyContract::get_policy(env.clone(), policy_id).unwrap()
        });
        assert!(policy.state == PolicyState::Expired);
    }
}
