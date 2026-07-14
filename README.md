# Tellus Protocol

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Tellus Protocol is an open-source prototype for parametric crop insurance on Stellar using Soroban smart contracts.

The repository currently contains four contracts:

- `pool`: tracks liquidity provider deposits, shares, coverage locks, and simulated payout releases.
- `policy`: stores farmer policy records with crop, geohash, coverage amount, and rainfall threshold data.
- `oracle`: stores the latest submitted weather or crop-health reading for a geohash and reading type.
- `trigger`: evaluates simulated rainfall values against a threshold and records trigger events.

This project is not deployed, audited, or production-ready. It is a development-stage codebase for experimenting with on-chain crop insurance mechanics.

## Current Status

Implemented:

- Soroban contracts for pool, policy, oracle, and trigger modules.
- Rust unit and integration tests for the contract modules.
- Basic TypeScript SDK scaffolding. (work in progress)
- Deployment and seed scripts for local/testnet experimentation.

Not implemented yet:

- Token transfers for premiums or payouts.
- Cross-contract payout orchestration from trigger to policy/pool.
- Oracle authentication, whitelisting, signatures, history, or median aggregation.
- A hosted application, hosted documentation site, or support channel.
- Mainnet deployment or security audit.

## Repository Layout

```text
contracts/
  oracle/      Latest-reading oracle contract
  policy/      Policy registry contract
  pool/        Liquidity pool accounting contract
  trigger/     Simulated trigger evaluation contract
docs/
  architecture.md
  farmer-onboarding.md
  oracle-spec.md
scripts/
  deploy.sh
  seed.ts
sdk/typescript/
tests/
```

## Prerequisites

- Rust 1.88.0, pinned by `rust-toolchain.toml`
- Stellar CLI, for contract deployment
- Node.js 20+, for TypeScript SDK work

## Build And Test

```bash
cargo build --target wasm32-unknown-unknown --release
cargo test
```

Format and lint Rust code:

```bash
cargo fmt
cargo clippy -- -D warnings
```

Build the TypeScript SDK:

```bash
cd sdk/typescript
npm install
npm run build
```

## Contract Notes

### Pool

The pool contract records provider shares and pool balances. `release_payout` reduces recorded capital and releases a policy lock, but it does not transfer a Stellar asset.

### Policy

The policy contract lets a farmer register a policy. Registration currently sets the season to 90 days from the current ledger timestamp and stores a rainfall threshold.

### Oracle

The oracle contract stores the latest reading by geohash and reading type. It does not authenticate submitters or aggregate multiple readings.

### Trigger

The trigger contract accepts simulated rainfall and threshold values. If rainfall is below the threshold, it stores a trigger event with a hardcoded payout amount. It does not read the policy/oracle contracts or call the pool contract.

## Documentation

- [Architecture](docs/architecture.md)
- [Oracle Contract Notes](docs/oracle-spec.md)
- [Farmer Workflow Model](docs/farmer-onboarding.md)

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for local setup, testing, and pull request guidance.

## Security

This project has not been audited. Do not use it to protect real funds or real insurance policies.

## License

MIT License. See [LICENSE](LICENSE).
