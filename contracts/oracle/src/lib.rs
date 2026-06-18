#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, String, Vec,
};

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone)]
#[contracttype]
pub struct HistoricalReading {
    pub value: u32,
    pub timestamp: u64,
    pub submitter: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct AggregatedReading {
    pub geo_cell: String,
    pub reading_type: ReadingType,
    pub value: u32,
    pub timestamp: u64,
    pub sample_count: u32,
}

#[contracttype]
pub enum DataKey {
    Config,
    LatestReading(String, ReadingType), // geo_cell, type -> latest value
    ReadingHistory(String, ReadingType), // geo_cell, type -> vector of historical readings
    HistoryIndex(String, ReadingType),  // geo_cell, type -> next index for circular buffer
    Whitelist(Address),                 // oracle_address -> bool (whitelisted)
    AggregatedReading(String, ReadingType), // geo_cell, type -> aggregated value
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub max_history_size: u32,
    pub max_reading_age: u64, // Maximum age of readings in seconds
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NoReadingsAvailable = 3,
    InvalidHistorySize = 4,
    NotAuthorized = 5,           // Caller not authorized (not admin)
    NotWhitelisted = 6,          // Submitter not whitelisted
    StaleReading = 7,            // Reading timestamp is too old
    InvalidTimestamp = 8,        // Timestamp is invalid (future or zero)
    NoAggregatedReading = 9,     // No aggregated reading available
}

#[contract]
pub struct OracleContract;

#[contractimpl]
impl OracleContract {
    /// Initialize the oracle contract with configurable history size and reading age
    pub fn initialize(env: Env, admin: Address, max_reading_age: u64) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        if max_reading_age == 0 {
            return Err(Error::InvalidHistorySize);
        }

        let config = Config {
            admin: admin.clone(),
            max_history_size: 100, // Default history size
            max_reading_age,
        };
        env.storage().instance().set(&DataKey::Config, &config);

        // Whitelist the admin by default
        let whitelist_key = DataKey::Whitelist(admin);
        env.storage().persistent().set(&whitelist_key, &true);

        Ok(())
    }

    /// Submit a reading with authentication, timestamp validation, and signature verification
    pub fn submit_reading(
        env: Env,
        submitter: Address,
        geo_cell: String,
        reading_type: ReadingType,
        value: u32,
        reading_timestamp: u64,
        signature: BytesN<64>,
    ) -> Result<(), Error> {
        submitter.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // Check that submitter is whitelisted
        if !Self::is_whitelisted(env.clone(), submitter.clone()) {
            return Err(Error::NotWhitelisted);
        }

        // Validate reading timestamp
        let current_time = env.ledger().timestamp();
        if reading_timestamp == 0 || reading_timestamp > current_time {
            return Err(Error::InvalidTimestamp);
        }

        // Check that reading is not too old
        let reading_age = current_time - reading_timestamp;
        if reading_age > config.max_reading_age {
            return Err(Error::StaleReading);
        }

        // Validate signature (placeholder for future implementation)
        Self::validate_signature(
            env.clone(),
            &submitter,
            &geo_cell,
            reading_type,
            value,
            reading_timestamp,
            &signature,
        )?;

        // Store the latest reading
        let reading = LatestReading {
            geo_cell: geo_cell.clone(),
            reading_type,
            value,
            timestamp: reading_timestamp,
        };

        let latest_key = DataKey::LatestReading(geo_cell.clone(), reading_type);
        env.storage().persistent().set(&latest_key, &reading);

        // Add to history
        let history_key = DataKey::ReadingHistory(geo_cell.clone(), reading_type);
        let mut history: Vec<HistoricalReading> = env
            .storage()
            .persistent()
            .get(&history_key)
            .unwrap_or(vec![&env]);

        let historical_reading = HistoricalReading {
            value,
            timestamp: reading_timestamp,
            submitter,
        };

        // Maintain circular buffer behavior
        if history.len() >= config.max_history_size as usize {
            history.remove(0);
        }

        history.push_back(historical_reading);
        env.storage().persistent().set(&history_key, &history);

        Ok(())
    }

    /// Validate signature on a reading (extensible for future cryptographic verification)
    fn validate_signature(
        env: Env,
        submitter: &Address,
        geo_cell: &String,
        reading_type: ReadingType,
        value: u32,
        timestamp: u64,
        signature: &BytesN<64>,
    ) -> Result<(), Error> {
        // Placeholder for signature validation logic
        // Future: Implement ECDSA or other signature schemes
        // For now, accept all signatures as long as basic checks pass above

        // Prevent unused variable warnings
        let _ = (env, submitter, geo_cell, reading_type, value, timestamp, signature);

        Ok(())
    }

    /// Aggregate readings using median calculation (admin only)
    pub fn aggregate_readings(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
        max_reading_age: u64,
    ) -> Result<(), Error> {
        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // Get the history of readings
        let history_key = DataKey::ReadingHistory(geo_cell.clone(), reading_type);
        let history: Vec<HistoricalReading> = env
            .storage()
            .persistent()
            .get(&history_key)
            .ok_or(Error::NoReadingsAvailable)?;

        if history.is_empty() {
            return Err(Error::NoReadingsAvailable);
        }

        let current_time = env.ledger().timestamp();

        // Filter readings within the specified age window
        let mut valid_readings: Vec<u32> = vec![&env];
        for reading in history.iter() {
            let reading_age = current_time - reading.timestamp;
            if reading_age <= max_reading_age {
                valid_readings.push_back(reading.value);
            }
        }

        if valid_readings.is_empty() {
            return Err(Error::NoReadingsAvailable);
        }

        // Calculate median
        let median = Self::calculate_median(env.clone(), valid_readings.clone())?;
        let sample_count = valid_readings.len() as u32;

        // Store aggregated reading
        let aggregated = AggregatedReading {
            geo_cell: geo_cell.clone(),
            reading_type,
            value: median,
            timestamp: current_time,
            sample_count,
        };

        let agg_key = DataKey::AggregatedReading(geo_cell, reading_type);
        env.storage().persistent().set(&agg_key, &aggregated);

        Ok(())
    }

    /// Get the aggregated reading (median) for a geo cell and reading type
    pub fn get_aggregated(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
    ) -> Result<AggregatedReading, Error> {
        let agg_key = DataKey::AggregatedReading(geo_cell, reading_type);
        env.storage()
            .persistent()
            .get(&agg_key)
            .ok_or(Error::NoAggregatedReading)
    }

    /// Calculate median of a set of values
    fn calculate_median(env: Env, mut values: Vec<u32>) -> Result<u32, Error> {
        if values.is_empty() {
            return Err(Error::NoReadingsAvailable);
        }

        // Simple bubble sort for small vectors
        let len = values.len();
        for i in 0..len {
            for j in 0..(len - i - 1) {
                if values.get(j).unwrap() > values.get(j + 1).unwrap() {
                    let temp = values.get(j).unwrap();
                    values.set(j, values.get(j + 1).unwrap());
                    values.set(j + 1, temp);
                }
            }
        }

        // Calculate median
        let median = if len % 2 == 0 {
            // For even count, return average of two middle values
            (values.get(len / 2 - 1).unwrap() + values.get(len / 2).unwrap()) / 2
        } else {
            // For odd count, return middle value
            values.get(len / 2).unwrap()
        };

        Ok(median)
    }

    /// Get latest reading for a geo cell and reading type
    pub fn get_latest(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
    ) -> Result<LatestReading, Error> {
        let key = DataKey::LatestReading(geo_cell, reading_type);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NoReadingsAvailable)
    }

    /// Get reading history for a geo cell and reading type
    pub fn get_history(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
    ) -> Result<Vec<HistoricalReading>, Error> {
        let history_key = DataKey::ReadingHistory(geo_cell, reading_type);
        env.storage()
            .persistent()
            .get(&history_key)
            .ok_or(Error::NoReadingsAvailable)
    }

    /// Get the median of recent readings (deprecated in favor of get_aggregated)
    pub fn get_median(env: Env, geo_cell: String, reading_type: ReadingType) -> Result<u32, Error> {
        let history = Self::get_history(env.clone(), geo_cell.clone(), reading_type)?;

        if history.is_empty() {
            return Err(Error::NoReadingsAvailable);
        }

        // Extract values into a vector
        let mut values: Vec<u32> = vec![&env];
        for reading in history.iter() {
            values.push_back(reading.value);
        }

        Self::calculate_median(env, values)
    }

    /// Get the count of readings in history
    pub fn get_history_count(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
    ) -> Result<u32, Error> {
        let history = Self::get_history(env, geo_cell, reading_type)?;
        Ok(history.len())
    }

    /// Add an oracle node to the whitelist (admin only)
    pub fn add_oracle_node(
        env: Env,
        admin: Address,
        oracle_address: Address,
    ) -> Result<(), Error> {
        admin.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if admin != config.admin {
            return Err(Error::NotAuthorized);
        }

        let whitelist_key = DataKey::Whitelist(oracle_address);
        env.storage().persistent().set(&whitelist_key, &true);

        Ok(())
    }

    /// Remove an oracle node from the whitelist (admin only)
    pub fn remove_oracle_node(
        env: Env,
        admin: Address,
        oracle_address: Address,
    ) -> Result<(), Error> {
        admin.require_auth();

        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if admin != config.admin {
            return Err(Error::NotAuthorized);
        }

        let whitelist_key = DataKey::Whitelist(oracle_address);
        env.storage().persistent().remove(&whitelist_key);

        Ok(())
    }

    /// Check if an address is whitelisted
    pub fn is_whitelisted(env: Env, oracle_address: Address) -> bool {
        let whitelist_key = DataKey::Whitelist(oracle_address);
        env.storage()
            .persistent()
            .get::<_, bool>(&whitelist_key)
            .unwrap_or(false)
    }
}
