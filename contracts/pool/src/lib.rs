#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[derive(Clone)]
#[contracttype]
pub struct PoolConfig {
    pub admin: Address,
    pub stablecoin_asset: Address,
    pub min_collateral_ratio: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct PoolStats {
    pub total_capital: i128,
    pub locked_amount: i128,
    pub total_shares: i128,
    pub utilization_ratio: u32, // Basis points
}

#[derive(Clone)]
#[contracttype]
pub struct ProviderPosition {
    pub shares: i128,
}

#[contracttype]
pub enum DataKey {
    Config,
    TotalCapital,
    LockedAmount,
    TotalShares,
    Provider(Address),
    PolicyLock(u64), // policy_id -> locked amount
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InsufficientCapital = 3,
    InsufficientShares = 4,
    InvalidAmount = 5,
    PolicyNotLocked = 6,
    PolicyAlreadyLocked = 7,
}

#[contract]
pub struct PoolContract;

#[contractimpl]
impl PoolContract {
    /// Initialize the liquidity pool with admin, stablecoin asset, and minimum collateral ratio
    pub fn initialize(
        env: Env,
        admin: Address,
        stablecoin_asset: Address,
        min_collateral_ratio: u32,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        let config = PoolConfig {
            admin,
            stablecoin_asset,
            min_collateral_ratio,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::TotalCapital, &0i128);
        env.storage().instance().set(&DataKey::LockedAmount, &0i128);
        env.storage().instance().set(&DataKey::TotalShares, &0i128);

        Ok(())
    }

    /// Deposit capital into the pool and receive LP shares
    pub fn deposit(env: Env, provider: Address, amount: i128) -> Result<i128, Error> {
        provider.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        // Transfer stablecoin from provider to pool
        let token_client = soroban_sdk::token::Client::new(&env, &config.stablecoin_asset);
        token_client.transfer(&provider, &env.current_contract_address(), &amount);

        let total_capital: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalCapital)
            .unwrap_or(0);
        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0);

        // Calculate shares to mint
        let shares_to_mint = if total_shares == 0 {
            amount // First deposit: 1:1 ratio
        } else {
            (amount * total_shares) / total_capital
        };

        // Update provider position
        let provider_key = DataKey::Provider(provider.clone());
        let mut position: ProviderPosition = env
            .storage()
            .persistent()
            .get(&provider_key)
            .unwrap_or(ProviderPosition { shares: 0 });
        position.shares += shares_to_mint;
        env.storage().persistent().set(&provider_key, &position);

        // Update pool totals
        env.storage()
            .instance()
            .set(&DataKey::TotalCapital, &(total_capital + amount));
        env.storage()
            .instance()
            .set(&DataKey::TotalShares, &(total_shares + shares_to_mint));

        Ok(shares_to_mint)
    }

    /// Withdraw capital by burning LP shares
    pub fn withdraw(env: Env, provider: Address, shares: i128) -> Result<i128, Error> {
        provider.require_auth();

        if shares <= 0 {
            return Err(Error::InvalidAmount);
        }

        let config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let total_capital: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalCapital)
            .unwrap_or(0);
        let locked_amount: i128 = env
            .storage()
            .instance()
            .get(&DataKey::LockedAmount)
            .unwrap_or(0);
        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0);

        let available_capital = total_capital - locked_amount;

        // Get provider position
        let provider_key = DataKey::Provider(provider.clone());
        let mut position: ProviderPosition = env
            .storage()
            .persistent()
            .get(&provider_key)
            .ok_or(Error::InsufficientShares)?;

        if position.shares < shares {
            return Err(Error::InsufficientShares);
        }

        // Calculate amount to return
        let amount_to_return = (shares * total_capital) / total_shares;

        if amount_to_return > available_capital {
            return Err(Error::InsufficientCapital);
        }

        // Update provider position
        position.shares -= shares;
        if position.shares == 0 {
            env.storage().persistent().remove(&provider_key);
        } else {
            env.storage().persistent().set(&provider_key, &position);
        }

        // Update pool totals
        env.storage()
            .instance()
            .set(&DataKey::TotalCapital, &(total_capital - amount_to_return));
        env.storage()
            .instance()
            .set(&DataKey::TotalShares, &(total_shares - shares));

        // Transfer stablecoin back to provider
        let token_client = soroban_sdk::token::Client::new(&env, &config.stablecoin_asset);
        token_client.transfer(
            &env.current_contract_address(),
            &provider,
            &amount_to_return,
        );

        Ok(amount_to_return)
    }

    /// Lock coverage amount for an active policy
    pub fn lock_coverage(env: Env, policy_id: u64, amount: i128) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::PolicyLock(policy_id))
        {
            return Err(Error::PolicyAlreadyLocked);
        }

        let total_capital: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalCapital)
            .unwrap_or(0);
        let locked_amount: i128 = env
            .storage()
            .instance()
            .get(&DataKey::LockedAmount)
            .unwrap_or(0);

        let new_locked = locked_amount
            .checked_add(amount)
            .ok_or(Error::InsufficientCapital)?;
        if new_locked > total_capital {
            return Err(Error::InsufficientCapital);
        }

        // min_collateral_ratio is scaled by 100 (500 means 5:1 free capital
        // to locked coverage). Keep enough capital unlocked after this policy.
        let required_free_capital = new_locked
            .checked_mul(i128::from(config.min_collateral_ratio))
            .ok_or(Error::InsufficientCapital)?
            / 100;
        if total_capital - new_locked < required_free_capital {
            return Err(Error::InsufficientCapital);
        }

        // Store the lock
        env.storage()
            .persistent()
            .set(&DataKey::PolicyLock(policy_id), &amount);
        env.storage()
            .instance()
            .set(&DataKey::LockedAmount, &new_locked);

        Ok(())
    }

    /// Release payout for a triggered policy
    pub fn release_payout(
        env: Env,
        policy_id: u64,
        farmer: Address,
        amount: i128,
    ) -> Result<(), Error> {
        let config: PoolConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)?;

        let locked_amount: i128 = env
            .storage()
            .instance()
            .get(&DataKey::LockedAmount)
            .unwrap_or(0);
        let total_capital: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalCapital)
            .unwrap_or(0);

        let policy_lock_key = DataKey::PolicyLock(policy_id);
        let policy_locked: i128 = env
            .storage()
            .persistent()
            .get(&policy_lock_key)
            .ok_or(Error::PolicyNotLocked)?;

        if amount > policy_locked {
            return Err(Error::InsufficientCapital);
        }

        // Release the lock and update totals
        env.storage().persistent().remove(&policy_lock_key);
        env.storage()
            .instance()
            .set(&DataKey::LockedAmount, &(locked_amount - policy_locked));
        env.storage()
            .instance()
            .set(&DataKey::TotalCapital, &(total_capital - amount));

        // Transfer stablecoin payout to the farmer
        let token_client = soroban_sdk::token::Client::new(&env, &config.stablecoin_asset);
        token_client.transfer(&env.current_contract_address(), &farmer, &amount);

        Ok(())
    }

    /// Get current pool statistics
    pub fn get_pool_stats(env: Env) -> Result<PoolStats, Error> {
        let total_capital: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalCapital)
            .unwrap_or(0);
        let locked_amount: i128 = env
            .storage()
            .instance()
            .get(&DataKey::LockedAmount)
            .unwrap_or(0);
        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0);

        let utilization_ratio = if total_capital > 0 {
            ((locked_amount * 10000) / total_capital) as u32 // basis points (1/10000)
        } else {
            0
        };

        Ok(PoolStats {
            total_capital,
            locked_amount,
            total_shares,
            utilization_ratio,
        })
    }

    /// Get provider's share balance
    pub fn get_provider_shares(env: Env, provider: Address) -> i128 {
        let provider_key = DataKey::Provider(provider);
        env.storage()
            .persistent()
            .get(&provider_key)
            .map(|p: ProviderPosition| p.shares)
            .unwrap_or(0)
    }

    /// Get provider's current capital value (shares redeemed at current pool ratio)
    pub fn get_provider_value(env: Env, provider: Address) -> i128 {
        let shares = Self::get_provider_shares(env.clone(), provider);
        if shares == 0 {
            return 0;
        }
        let total_capital: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalCapital)
            .unwrap_or(0);
        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0);
        if total_shares == 0 {
            return 0;
        }
        (shares * total_capital) / total_shares
    }
}
