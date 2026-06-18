# Tellus Protocol Architecture

Tellus Protocol is a Soroban prototype made of four contracts: pool, policy, oracle, and trigger. The contracts are currently tested together in Rust, but full production payout orchestration is not implemented.

## Contract Overview

```text
Farmer address ---- registers policy ----> Policy contract
LP address -------- deposits capital ----> Pool contract
Oracle submitter --- submits reading ----> Oracle contract
Caller ------------ simulated trigger ---> Trigger contract
```

## Pool Contract

Purpose: record liquidity provider capital, shares, coverage locks, and simulated payout releases.

Implemented methods:

- `initialize(admin, stablecoin_asset, min_collateral_ratio)`
- `deposit(provider, amount)`
- `withdraw(provider, shares)`
- `lock_coverage(policy_id, amount)`
- `release_payout(policy_id, farmer, amount)`
- `get_pool_stats()`
- `get_provider_shares(provider)`
- `get_provider_value(provider)`

Important limitation: the pool contract updates accounting values only. It does not transfer a Stellar token when deposits, withdrawals, or payouts occur.

## Policy Contract

Purpose: store farmer policy records.

Implemented methods:

- `initialize(admin, pool_contract)`
- `register_policy(farmer, farm_geohash, crop_type, coverage_amount, rainfall_threshold)`
- `get_policy(policy_id)`
- `list_policies_by_farmer(farmer)`
- `update_policy_state(policy_id, new_state)`

Current registration behavior:

- Requires farmer authorization.
- Rejects non-positive coverage amounts.
- Sets `season_start` to the current ledger timestamp.
- Sets `season_end` to 90 days after `season_start`.
- Sets `ndvi_baseline` to `0`.
- Sets state to `Active`.

## Oracle Contract

Purpose: store weather and crop-health readings with authentication, history, and aggregation.

Implemented methods:

- `initialize(admin, max_reading_age)` - Initialize with admin and max age for readings
- `add_oracle_node(admin, oracle_address)` - Whitelist an oracle node (admin only)
- `remove_oracle_node(admin, oracle_address)` - Remove oracle node from whitelist (admin only)
- `is_whitelisted(oracle_address)` - Query whitelist status
- `submit_reading(submitter, geo_cell, reading_type, value, reading_timestamp, signature)` - Submit authenticated reading
- `get_latest(geo_cell, reading_type)` - Get most recent reading
- `get_history(geo_cell, reading_type)` - Get historical readings (circular buffer)
- `get_history_count(geo_cell, reading_type)` - Get count of readings in history
- `aggregate_readings(geo_cell, reading_type, max_reading_age)` - Compute median of recent readings
- `get_aggregated(geo_cell, reading_type)` - Retrieve aggregated (median) reading
- `get_median(geo_cell, reading_type)` - Get median of all historical readings (deprecated)

New features implemented:

- **Whitelist enforcement**: Only whitelisted oracle nodes can submit readings
- **Authentication**: Requires submitter signature via `require_auth()`
- **Timestamp validation**: Rejects readings that are too old (> max_reading_age)
- **Future timestamp rejection**: Prevents readings with invalid timestamps
- **Reading history**: Maintains circular buffer of historical submissions
- **Aggregation**: Computes median from readings within specified time window
- **Submitter tracking**: Records which oracle node submitted each reading

Important limitation: the oracle contract does not yet perform cryptographic signature verification (placeholder for future implementation).

## Trigger Contract

Purpose: record a trigger event when simulated rainfall is below a simulated threshold.

Implemented methods:

- `initialize(admin, policy_contract, oracle_contract, pool_contract)`
- `evaluate_policy(policy_id, simulated_rainfall, simulated_threshold)`
- `get_trigger_event(policy_id)`
- `is_triggered(policy_id)`

Current trigger behavior:

- Rejects a policy that has already been triggered.
- Compares the supplied rainfall value with the supplied threshold.
- Stores a trigger event when rainfall is below threshold.
- Uses a hardcoded payout amount.

Important limitation: the trigger contract does not read policy data, read oracle data, update policy state, call the pool, or transfer funds.

## Storage Model

The contracts use Soroban typed storage keys.

Instance storage:

- Contract configuration.
- Pool totals.
- Policy ID counter.

Persistent storage:

- Provider positions.
- Policy records.
- Farmer policy lists.
- Latest oracle readings.
- Trigger events.
- Coverage locks.

## Security Notes

Implemented safeguards:

- Farmer authorization is required to register a policy.
- Liquidity providers authorize deposits and withdrawals.
- Contract initialization can only happen once.
- Invalid or non-positive amounts are rejected in core pool and policy paths.
- Duplicate trigger events for the same policy are rejected.

Known gaps:

- No token transfer integration.
- No cryptographic signature verification for oracle readings (placeholder implemented).
- No cross-contract authorization for pool payout release.
- No premium calculation or premium collection.
- No security audit.

## Development Guidance

Keep architecture documentation aligned with code. When adding cross-contract calls, token transfers, oracle validation, or payout automation, update this document in the same change.
