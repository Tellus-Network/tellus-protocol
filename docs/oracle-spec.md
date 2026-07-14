# Oracle Contract Specification

This document describes the oracle contract implementation with authentication, whitelisting, and aggregation.

## Overview

The oracle contract stores weather and crop-health readings, maintains historical data, and provides aggregated values via median calculation. The contract enforces whitelist-based access control and validates reading timestamps for freshness.

## Data Types

### ReadingType Enum
Supported reading types:
- `Rainfall`
- `NDVI` (Normalized Difference Vegetation Index)
- `SoilMoisture`

### LatestReading Struct
Stores the most recent reading for a location and type:
- `geo_cell`: geohash-like location identifier (String)
- `reading_type`: ReadingType enum value
- `value`: unsigned integer value (u32)
- `timestamp`: ledger timestamp when reading was accepted (u64)

### HistoricalReading Struct
Individual reading entry with submitter tracking:
- `value`: unsigned integer value (u32)
- `timestamp`: ledger timestamp when reading was submitted (u64)
- `submitter`: Address of the oracle node that submitted the reading

### AggregatedReading Struct
Median aggregation result:
- `geo_cell`: geohash location identifier (String)
- `reading_type`: ReadingType enum value
- `value`: median value from aggregation (u32)
- `timestamp`: ledger timestamp when aggregation was performed (u64)
- `sample_count`: number of readings included in median (u32)

## Public Methods

### Administrative

#### `initialize(admin, max_reading_age)`
Initializes the contract with:
- `admin`: Administrative address with whitelist management privileges
- `max_reading_age`: Maximum age of readings in seconds (e.g., 172800 for 48 hours)

The admin is automatically whitelisted on initialization.

**Constraints:**
- Can only be called once (returns `AlreadyInitialized` if called again)
- `max_reading_age` must be greater than zero

#### `add_oracle_node(admin, oracle_address)`
Whitelists an oracle address to submit readings.

**Authorization:** Requires `admin` signature

**Constraints:**
- Caller must match the configured admin
- Returns `NotAuthorized` if caller is not admin

#### `remove_oracle_node(admin, oracle_address)`
Removes an oracle address from the whitelist.

**Authorization:** Requires `admin` signature

**Constraints:**
- Caller must match the configured admin
- Returns `NotAuthorized` if caller is not admin

### Data Submission

#### `submit_reading(submitter, geo_cell, reading_type, value, reading_timestamp, signature)`
Submits a reading from a whitelisted oracle node.

**Parameters:**
- `submitter`: Address submitting the reading (must be whitelisted)
- `geo_cell`: Location identifier (String)
- `reading_type`: Type of reading (ReadingType enum)
- `value`: Unsigned integer value (u32)
- `reading_timestamp`: Timestamp when reading was captured (u64)
- `signature`: 64-byte signature for future cryptographic verification (BytesN<64>)

**Authorization:** Requires `submitter` signature

**Validation:**
- `submitter` must be whitelisted (returns `NotWhitelisted` if not)
- `reading_timestamp` must be valid: not zero and not in the future (returns `InvalidTimestamp` if invalid)
- Reading must not be older than `max_reading_age` (returns `StaleReading` if too old)
- The signature is accepted as an opaque 64-byte value; cryptographic verification is not implemented yet

**Behavior:**
- Updates the latest reading for the `(geo_cell, reading_type)` pair
- Appends to historical readings (maintains circular buffer up to `max_history_size`)
- Returns error if submitter is not whitelisted or timestamp validation fails

### Data Retrieval

#### `get_latest(geo_cell, reading_type)`
Retrieves the most recent reading for a location and type.

**Returns:**
- `LatestReading` struct with the latest data
- `NoReadingsAvailable` if no readings have been submitted

#### `get_history(geo_cell, reading_type)`
Retrieves the historical readings for a location and type.

**Returns:**
- Vector of `HistoricalReading` entries (circular buffer)
- `NoReadingsAvailable` if no readings have been submitted
- Maximum count is `max_history_size` (default 100)

#### `get_history_count(geo_cell, reading_type)`
Retrieves the count of historical readings available.

**Returns:**
- Number of readings in history (u32)
- `NoReadingsAvailable` if no readings exist

### Aggregation

#### `aggregate_readings(geo_cell, reading_type, max_reading_age)`
Computes the median of recent readings within the specified age window.

**Parameters:**
- `geo_cell`: Location identifier
- `reading_type`: Type of reading to aggregate
- `max_reading_age`: Maximum age of readings to include in the window (seconds)

**Behavior:**
- Filters historical readings to include only those within the age window
- Calculates median using bubble sort
- Stores aggregated result for future retrieval
- Returns `NoReadingsAvailable` if no readings exist or all are outside the window

#### `get_aggregated(geo_cell, reading_type)`
Retrieves the most recent aggregation result.

**Returns:**
- `AggregatedReading` struct with median value and sample count
- `NoAggregatedReading` if no aggregation has been performed

#### `get_median(geo_cell, reading_type)` [Deprecated]
Calculates the median of all available readings (uses full history).

**Returns:**
- Median value (u32) of all readings in history
- `NoReadingsAvailable` if no readings exist

**Note:** Deprecated in favor of `aggregate_readings()` with explicit time windows.

### Utility

#### `is_whitelisted(oracle_address)`
Checks if an address is authorized to submit readings.

**Parameters:**
- `oracle_address`: Address to check

**Returns:**
- `true` if whitelisted
- `false` if not whitelisted

## Storage Model

### Instance Storage
- `Config`: Admin address, max_history_size, max_reading_age

### Persistent Storage
- `LatestReading(geo_cell, reading_type)`: Most recent reading
- `ReadingHistory(geo_cell, reading_type)`: Vector of historical readings
- `AggregatedReading(geo_cell, reading_type)`: Aggregated median value
- `Whitelist(oracle_address)`: Boolean flag for whitelisted addresses

## Error Types

| Error | Code | Description |
|-------|------|-------------|
| `AlreadyInitialized` | 1 | Contract already initialized |
| `NotInitialized` | 2 | Contract not yet initialized |
| `NoReadingsAvailable` | 3 | No readings exist for the query |
| `InvalidHistorySize` | 4 | History size parameter invalid |
| `NotAuthorized` | 5 | Caller is not authorized (e.g., not admin) |
| `NotWhitelisted` | 6 | Submitter not whitelisted |
| `StaleReading` | 7 | Reading timestamp is too old |
| `InvalidTimestamp` | 8 | Reading timestamp is invalid (zero or future) |
| `NoAggregatedReading` | 9 | No aggregated reading available |

## Data Units

The contract stores raw unsigned integer values and does not enforce units. Integrations should document the units they use:
- **Rainfall**: millimeters or decimeters (scaled integer)
- **NDVI**: scaled integer (e.g., 0-10000 for 0.0-1.0)
- **SoilMoisture**: percentage or scaled integer

## Security Considerations

### Implemented
- **Whitelist enforcement**: Only whitelisted oracle nodes can submit readings
- **Timestamp validation**: Readings must not be stale (older than max_reading_age)
- **Future timestamp rejection**: Readings cannot have future timestamps
- **Admin authorization**: Whitelist management requires admin signature
- **Circular buffer**: History is bounded to prevent unbounded storage growth

### Future Enhancements
- **Signature verification**: Implement ECDSA or EdDSA signature validation
- **Oracle diversity**: Require minimum number of distinct submitters for aggregation
- **Rate limiting**: Prevent oracle spam or excessive submissions
- **Deviation detection**: Flag readings that deviate significantly from median
- **Timestamp sources**: Allow submitter's signed timestamp (currently uses ledger time)

Until signature verification is implemented, consumers must treat whitelist membership and
Soroban address authentication—not the `signature` argument—as the reading's trust boundary.

## Usage Example

```
1. Admin initializes: initialize(admin, 172800)  // 48-hour window
2. Admin whitelists: add_oracle_node(admin, oracle1), add_oracle_node(admin, oracle2)
3. Oracle 1 submits: submit_reading(oracle1, "9q5ct", Rainfall, 250, 1000000, sig1)
4. Oracle 2 submits: submit_reading(oracle2, "9q5ct", Rainfall, 280, 1000005, sig2)
5. Contract aggregates: aggregate_readings("9q5ct", Rainfall, 10000)
6. Caller retrieves: get_aggregated("9q5ct", Rainfall) -> AggregatedReading(265, 2)
```

## Implementation Notes

- Median calculation uses bubble sort for simplicity; suitable for small sample sizes — consider optimizing for large samples
- Circular buffer maintains only the most recent `max_history_size` readings
- Aggregation is performed on-demand; results are cached
- Signature parameter is reserved for future cryptographic verification
- Reading age is checked against ledger timestamp, not provided timestamp
