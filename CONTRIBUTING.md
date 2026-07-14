# Contributing To Tellus Protocol

Thank you for helping improve Tellus Protocol. This repository is an open-source Soroban prototype for parametric crop insurance.

## Ways To Contribute

- Fix contract bugs.
- Add focused tests for contract behavior.
- Improve TypeScript SDK accuracy.
- Keep documentation aligned with implemented code.
- Review security-sensitive logic.

## Local Setup

Prerequisites:

- Rust 1.88.0, installed automatically when using `rustup` with `rust-toolchain.toml`.
- Stellar CLI for deployment work.
- Node.js 20+ for TypeScript SDK development.
- Git.

Clone your fork or local copy, then install dependencies:

```bash
cargo build

cd sdk/typescript
npm install # install SDK dependencies
cd ../..
```

## Running Tests

Rust contracts:

```bash
cargo test
cargo test pool_tests
cargo test -- --nocapture
```

Rust formatting and linting:

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

TypeScript SDK:

```bash
cd sdk/typescript
npm run build
npm run typecheck
```

## Pull Requests

Before opening a pull request:

1. Keep the change focused.
2. Add or update tests when behavior changes.
3. Update docs when public behavior changes.
4. Run the relevant Rust and TypeScript checks.
5. Write a clear summary of what changed and why.

Suggested PR title format:

- `feat: add rainfall validation`
- `fix: reject invalid coverage locks`
- `docs: update oracle contract notes`
- `test: cover triggered policy state`
- `chore: update sdk package metadata`

## Development Guidelines

Rust smart contracts:

- Prefer typed errors over raw numeric failures.
- Use typed storage keys.
- Avoid `unwrap()` in contract logic.
- Validate inputs before changing state.
- Keep tests isolated and deterministic.

TypeScript SDK:

- Keep exported types in sync with contract APIs.
- Use strict TypeScript types.
- Keep examples limited to implemented behavior.
- Avoid documenting methods that do not work against the current contracts.

Documentation:

- Describe current behavior plainly.
- Do not add links to project channels or hosted resources that do not exist.
- Mark limitations directly instead of presenting planned behavior as available behavior.

## Security

Never commit private keys, seed phrases, API keys, or deployment secrets.

If you find a vulnerability and the repository does not yet have private vulnerability reporting enabled, open a minimal public issue asking maintainers to establish a private reporting channel. Do not include exploit details in the issue.

## Code Of Conduct

Contributors are expected to be respectful, constructive, and focused on the work. Harassment, personal attacks, and publishing another person's private information are not acceptable.

## Questions

Open an issue in the repository for bugs, implementation questions, or documentation gaps.
