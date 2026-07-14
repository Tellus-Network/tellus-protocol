#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, Address, Env, String,
};

#[derive(Clone)]
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
    pub rainfall_threshold: u32,
    pub ndvi_baseline: u32,
    pub state: PolicyState,
}

#[contractclient(name = "PolicyClient")]
pub trait PolicyInterface {
    fn get_policy(env: Env, policy_id: u64) -> Policy;
    fn update_policy_state(env: Env, policy_id: u64, new_state: PolicyState);
}

#[derive(Clone, Copy)]
#[contracttype]
pub enum ReadingType {
    Rainfall,
    NDVI,
    SoilMoisture,
}

#[derive(Clone)]
#[contracttype]
pub struct LatestReading {
    pub geo_cell: String,
    pub reading_type: ReadingType,
    pub value: u32,
    pub timestamp: u64,
}

#[contractclient(name = "OracleClient")]
pub trait OracleInterface {
    fn get_latest(env: Env, geo_cell: String, reading_type: ReadingType) -> LatestReading;
}

#[contractclient(name = "PoolClient")]
pub trait PoolInterface {
    fn release_payout(env: Env, policy_id: u64, farmer: Address, amount: i128);
}

#[derive(Clone)]
#[contracttype]
pub struct TriggerEvent {
    pub policy_id: u64,
    pub triggered_at: u64,
    pub rainfall_value: u32,
    pub payout_amount: i128,
    pub trigger_reason: String,
}

#[contracttype]
pub enum DataKey {
    Config,
    Triggered(u64), // policy_id -> TriggerEvent
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub policy_contract: Address,
    pub oracle_contract: Address,
    pub pool_contract: Address,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AlreadyTriggered = 3,
    ThresholdNotMet = 4,
    InvalidPolicyId = 5,
    InactivePolicy = 6,
}

#[contract]
pub struct TriggerContract;

#[contractimpl]
impl TriggerContract {
    /// Initialize the trigger contract
    pub fn initialize(
        env: Env,
        admin: Address,
        policy_contract: Address,
        oracle_contract: Address,
        pool_contract: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        let config = Config {
            admin,
            policy_contract,
            oracle_contract,
            pool_contract,
        };

        env.storage().instance().set(&DataKey::Config, &config);

        Ok(())
    }

    /// Return the configured contract dependencies.
    pub fn get_config(env: Env) -> Result<Config, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    /// Evaluate a policy
    pub fn evaluate_policy(env: Env, policy_id: u64) -> Result<(), Error> {
        if policy_id == 0 {
            return Err(Error::InvalidPolicyId);
        }
        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // Check if already triggered
        if env
            .storage()
            .persistent()
            .has(&DataKey::Triggered(policy_id))
        {
            return Err(Error::AlreadyTriggered);
        }

        let current_time = env.ledger().timestamp();

        // 1. Fetch policy details from Policy contract
        let policy_client = PolicyClient::new(&env, &config.policy_contract);
        let policy = policy_client.get_policy(&policy_id);

        if !matches!(policy.state, PolicyState::Active) {
            return Err(Error::InactivePolicy);
        }

        // Check if season has ended (policy expired)
        if current_time > policy.season_end {
            return Err(Error::ThresholdNotMet);
        }

        // 2. Fetch latest readings from Oracle contract
        let oracle_client = OracleClient::new(&env, &config.oracle_contract);

        // Check if rainfall is below threshold (drought)
        let rainfall_reading =
            oracle_client.get_latest(&policy.farm_geohash, &ReadingType::Rainfall);

        if rainfall_reading.value < policy.rainfall_threshold {
            let payout_amount = policy.coverage_amount;

            // Release payout from pool
            let pool_client = PoolClient::new(&env, &config.pool_contract);
            pool_client.release_payout(&policy_id, &policy.farmer, &payout_amount);

            // Update policy state
            policy_client.update_policy_state(&policy_id, &PolicyState::Triggered);

            let trigger_event = TriggerEvent {
                policy_id,
                triggered_at: current_time,
                rainfall_value: rainfall_reading.value,
                payout_amount,
                trigger_reason: String::from_str(&env, "drought_detected"),
            };
            // Persist trigger event for audit and replay

            env.storage()
                .persistent()
                .set(&DataKey::Triggered(policy_id), &trigger_event);

            return Ok(());
        }

        // Check NDVI if baseline is set (crop stress)
        if policy.ndvi_baseline > 0 {
            let ndvi_reading = oracle_client.get_latest(&policy.farm_geohash, &ReadingType::NDVI);

            let stress_threshold = (policy.ndvi_baseline * 7) / 10; // 70% of baseline
            if ndvi_reading.value < stress_threshold {
                let payout_amount = policy.coverage_amount / 2; // 50% partial payout

                // Release payout from pool
                let pool_client = PoolClient::new(&env, &config.pool_contract);
                pool_client.release_payout(&policy_id, &policy.farmer, &payout_amount);

                // Update policy state
                policy_client.update_policy_state(&policy_id, &PolicyState::Triggered);

                let trigger_event = TriggerEvent {
                    policy_id,
                    triggered_at: current_time,
                    rainfall_value: rainfall_reading.value,
                    payout_amount,
                    trigger_reason: String::from_str(&env, "crop_stress_detected"),
                };

                env.storage()
                    .persistent()
                    .set(&DataKey::Triggered(policy_id), &trigger_event);

                return Ok(());
            }
        }

        Err(Error::ThresholdNotMet)
    }

    /// Get trigger event details for a policy
    pub fn get_trigger_event(env: Env, policy_id: u64) -> Result<TriggerEvent, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Triggered(policy_id))
            .ok_or(Error::NotInitialized)
    }

    /// Check if a policy has been triggered
    pub fn is_triggered(env: Env, policy_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Triggered(policy_id))
    }
}
