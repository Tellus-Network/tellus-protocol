# Farmer Workflow Model

This document describes the farmer-facing workflow that the current contracts are shaped around. It is not an end-user onboarding guide for a live application, because this repository does not currently include a web app, production deployment, support channel, or audited payout system.

## What Exists Today

The current contracts can model these steps in tests or development deployments:

1. A farmer address registers a policy.
2. The policy stores a farm geohash, crop type, coverage amount, and rainfall threshold.
3. Oracle readings can be submitted for a geohash and reading type.
4. A trigger event can be recorded when a simulated rainfall value is below a simulated threshold.
5. Pool accounting can record deposits, shares, coverage locks, and simulated payout releases.

## What Does Not Exist Yet

The current implementation does not provide:

- A farmer web application.
- Wallet-guided registration screens.
- Premium collection.
- Real USDC payouts.
- Automatic monitoring workers.
- Cross-contract trigger evaluation.
- Oracle aggregation or signer verification.
- Farmer notifications.
- Customer support or local agents.

## Current Policy Data

The policy contract stores:

- Farmer address.
- Farm geohash.
- Crop type.
- Season start and end timestamps.
- Coverage amount.
- Rainfall threshold.
- NDVI baseline, currently set to `0` during registration.
- Policy state.

Policy registration currently sets the season start from the ledger timestamp and the season end to 90 days later.

## Current Trigger Behavior

The trigger contract does not fetch policy or oracle data. It accepts:

- `policy_id`
- `simulated_rainfall`
- `simulated_threshold`

If `simulated_rainfall < simulated_threshold`, the contract records a trigger event with reason `drought_detected` and a hardcoded payout amount. No token transfer occurs (simulated accounting only).

## Development Use

For now, this document is useful as a product model for contributors. Any real farmer-facing guide should wait until the project has:

- A working application flow.
- Real premium and payout transfers.
- Verified oracle data.
- Clear deployment addresses.
- Security review.
