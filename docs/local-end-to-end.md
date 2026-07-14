# Local end-to-end workflow

This checklist exercises the complete Tellus flow without deploying to a public network.

## Prerequisites

- Rust and the `wasm32-unknown-unknown` target
- Stellar CLI with Soroban support
- Node.js 20 or newer for the TypeScript SDK

## Validate the workspace

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
```

Build each deployable contract for Soroban WASM:

```sh
cargo build --locked --target wasm32-unknown-unknown --release
```

Then validate the SDK:

```sh
cd sdk/typescript
npm ci
npm run typecheck
npm run build
```

## Protocol flow

The integration suite performs the same sequence an application should follow:

1. Initialize the pool, oracle, policy, and trigger contracts.
2. Deposit provider capital and record the resulting LP shares.
3. Whitelist an oracle node and submit fresh readings.
4. Register a policy; registration locks its coverage in the pool.
5. Evaluate the policy through the trigger contract.
6. Inspect the trigger event, policy state, farmer balance, and remaining pool capital.

Use unique contract instances for each local run. Timestamps must be non-zero, no later than
the current ledger time, and within the oracle's configured freshness window.
