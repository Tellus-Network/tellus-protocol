#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, vec, Address, Env, String, Vec};

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

#[contracttype]
pub enum DataKey {
    Config,
    LatestReading(String, ReadingType), // geo_cell, type -> latest value
    ReadingHistory(String, ReadingType), // geo_cell, type -> vector of historical readings
    HistoryIndex(String, ReadingType), // geo_cell, type -> next index for circular buffer
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub max_history_size: u32,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NoReadingsAvailable = 3,
    InvalidHistorySize = 4,
}

#[contract]
pub struct OracleContract;

#[contractimpl]
impl OracleContract {
    /// Initialize the oracle contract with configurable history size
    pub fn initialize(env: Env, admin: Address, max_history_size: u32) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        if max_history_size == 0 {
            return Err(Error::InvalidHistorySize);
        }

        let config = Config {
            admin,
            max_history_size,
        };
        env.storage().instance().set(&DataKey::Config, &config);

        Ok(())
    }

    /// Submit a reading with submitter tracking
    pub fn submit_reading(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
        value: u32,
        submitter: Address,
    ) -> Result<(), Error> {
        let config: Config = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let current_time = env.ledger().timestamp();

        // Store the latest reading
        let reading = LatestReading {
            geo_cell: geo_cell.clone(),
            reading_type,
            value,
            timestamp: current_time,
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
            timestamp: current_time,
            submitter,
        };

        // Maintain circular buffer behavior
        if history.len() >= config.max_history_size {
            history.remove(0);
        }

        history.push_back(historical_reading);
        env.storage().persistent().set(&history_key, &history);

        Ok(())
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

    /// Get the median of recent readings
    pub fn get_median(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
    ) -> Result<u32, Error> {
        let history = Self::get_history(env.clone(), geo_cell.clone(), reading_type)?;

        if history.is_empty() {
            return Err(Error::NoReadingsAvailable);
        }

        // Extract values into a vector
        let mut values: Vec<u32> = vec![&env];
        for reading in history.iter() {
            values.push_back(reading.value);
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

    /// Get the count of readings in history
    pub fn get_history_count(
        env: Env,
        geo_cell: String,
        reading_type: ReadingType,
    ) -> Result<u32, Error> {
        let history = Self::get_history(env, geo_cell, reading_type)?;
        Ok(history.len() as u32)
    }
}
