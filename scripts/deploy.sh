#!/bin/bash
set -euo pipefail

echo "🚀 Deploying Tellus Protocol contracts to Stellar Testnet"

# Check if stellar CLI is installed
if ! command -v stellar &> /dev/null; then
    echo "❌ Stellar CLI not found. Install from: https://developers.stellar.org/docs/tools/developer-tools"
    exit 1
fi

# Build contracts
echo "📦 Building contracts..."
cargo build --target wasm32-unknown-unknown --release

# Optimize WASM files
echo "🔧 Optimizing WASM files..."
stellar contract optimize \
    --wasm target/wasm32-unknown-unknown/release/tellus_pool.wasm \
    --wasm-out target/wasm32-unknown-unknown/release/tellus_pool_optimized.wasm

stellar contract optimize \
    --wasm target/wasm32-unknown-unknown/release/tellus_policy.wasm \
    --wasm-out target/wasm32-unknown-unknown/release/tellus_policy_optimized.wasm

stellar contract optimize \
    --wasm target/wasm32-unknown-unknown/release/tellus_oracle.wasm \
    --wasm-out target/wasm32-unknown-unknown/release/tellus_oracle_optimized.wasm

stellar contract optimize \
    --wasm target/wasm32-unknown-unknown/release/tellus_trigger.wasm \
    --wasm-out target/wasm32-unknown-unknown/release/tellus_trigger_optimized.wasm

# Set network
NETWORK="testnet"
echo "🌐 Using network: $NETWORK"

# Deploy pool contract
echo "📤 Deploying pool contract..."
POOL_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/tellus_pool_optimized.wasm \
    --source admin \
    --network $NETWORK)
echo "✅ Pool contract deployed: $POOL_ID"

# Deploy oracle contract
echo "📤 Deploying oracle contract..."
ORACLE_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/tellus_oracle_optimized.wasm \
    --source admin \
    --network $NETWORK)
echo "✅ Oracle contract deployed: $ORACLE_ID"

# Deploy policy contract
echo "📤 Deploying policy contract..."
POLICY_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/tellus_policy_optimized.wasm \
    --source admin \
    --network $NETWORK)
echo "✅ Policy contract deployed: $POLICY_ID"

# Deploy trigger contract
echo "📤 Deploying trigger contract..."
TRIGGER_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/tellus_trigger_optimized.wasm \
    --source admin \
    --network $NETWORK)
echo "✅ Trigger contract deployed: $TRIGGER_ID"

# Save contract addresses
cat > deployed_contracts.json <<EOF
{
  "network": "$NETWORK",
  "contracts": {
    "pool": "$POOL_ID",
    "oracle": "$ORACLE_ID",
    "policy": "$POLICY_ID",
    "trigger": "$TRIGGER_ID"
  },
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo ""
echo "✨ Deployment complete!"
echo "📝 Contract addresses saved to deployed_contracts.json"
echo ""
echo "Next steps:"
echo "1. Initialize contracts with: npm run seed"
echo "2. Use the saved contract IDs in local scripts or SDK experiments"
echo ""
echo "Contract IDs:"
echo "  Pool:    $POOL_ID"
echo "  Oracle:  $ORACLE_ID"
echo "  Policy:  $POLICY_ID"
echo "  Trigger: $TRIGGER_ID"
