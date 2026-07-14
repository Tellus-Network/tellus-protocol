import { Keypair, Networks } from '@stellar/stellar-sdk';
import { TellusClient } from '../sdk/typescript/src';
import * as fs from 'fs';

/**
 * Seed script to populate testnet with sample data
 * Creates 3 farmer policies, 2 LP deposits, and 5 oracle readings
 */

async function main() {
  console.log('🌱 Seeding Tellus Protocol testnet...\n');
  console.debug('seed script starting');

  // Load deployed contract addresses
  const deployed = JSON.parse(fs.readFileSync('deployed_contracts.json', 'utf-8'));

  // Initialize client
  const client = new TellusClient({
    networkPassphrase: Networks.TESTNET,
    rpcUrl: 'https://soroban-testnet.stellar.org',
    poolContractId: deployed.contracts.pool,
    policyContractId: deployed.contracts.policy,
    oracleContractId: deployed.contracts.oracle,
    triggerContractId: deployed.contracts.trigger,
  });

  // Generate test keypairs
  const admin = Keypair.random();
  const provider1 = Keypair.random();
  const provider2 = Keypair.random();
  const farmer1 = Keypair.random();
  const farmer2 = Keypair.random();
  const farmer3 = Keypair.random();
  const oracleNode = Keypair.random();

  console.log('Generated test accounts:');
  console.log(`  Admin:     ${admin.publicKey()}`);
  console.log(`  Provider1: ${provider1.publicKey()}`);
  console.log(`  Provider2: ${provider2.publicKey()}`);
  console.log(`  Farmer1:   ${farmer1.publicKey()}`);
  console.log(`  Farmer2:   ${farmer2.publicKey()}`);
  console.log(`  Farmer3:   ${farmer3.publicKey()}`);
  console.log(`  Oracle:    ${oracleNode.publicKey()}\n`);

  // Fund accounts (in production, users would fund their own accounts)
  console.log('💰 Funding accounts from Friendbot...');
  await Promise.all([
    fundAccount(admin.publicKey()),
    fundAccount(provider1.publicKey()),
    fundAccount(provider2.publicKey()),
    fundAccount(farmer1.publicKey()),
    fundAccount(farmer2.publicKey()),
    fundAccount(farmer3.publicKey()),
    fundAccount(oracleNode.publicKey()),
  ]);
  console.log('✅ Accounts funded\n');

  // 1. LP Deposits
  console.log('💵 Creating LP deposits...');
  try {
    const shares1 = await client.deposit(provider1, BigInt(50_000_0000000)); // 50,000 USDC
    console.log(`  Provider1 deposited 50,000 USDC, received ${shares1} shares`);

    const shares2 = await client.deposit(provider2, BigInt(30_000_0000000)); // 30,000 USDC
    console.log(`  Provider2 deposited 30,000 USDC, received ${shares2} shares`);
  } catch (error) {
    console.error('  ⚠️  LP deposits failed (may need to initialize pool first)');
  }
  console.log('');

  // 2. Register Policies
  console.log('📋 Registering farmer policies...');
  try {
    const now = Math.floor(Date.now() / 1000);
    const seasonDuration = 90 * 24 * 60 * 60; // 90 days

    const policy1 = await client.registerPolicy(farmer1, {
      farmer: farmer1.publicKey(),
      farmGeohash: '9q5ct', // San Francisco area (example)
      cropType: 'maize',
      seasonStart: BigInt(now),
      seasonEnd: BigInt(now + seasonDuration),
      coverageAmount: BigInt(5_000_0000000), // 5,000 USDC
      rainfallThreshold: 200, // 200mm
      ndviBaseline: 7000, // 0.7 scaled by 10000
    });
    console.log(`  Farmer1 registered policy #${policy1} for maize`);

    const policy2 = await client.registerPolicy(farmer2, {
      farmer: farmer2.publicKey(),
      farmGeohash: '9q5cu',
      cropType: 'wheat',
      seasonStart: BigInt(now),
      seasonEnd: BigInt(now + seasonDuration),
      coverageAmount: BigInt(3_000_0000000), // 3,000 USDC
      rainfallThreshold: 180,
      ndviBaseline: 6500,
    });
    console.log(`  Farmer2 registered policy #${policy2} for wheat`);

    const policy3 = await client.registerPolicy(farmer3, {
      farmer: farmer3.publicKey(),
      farmGeohash: '9q5cv',
      cropType: 'sorghum',
      seasonStart: BigInt(now),
      seasonEnd: BigInt(now + seasonDuration),
      coverageAmount: BigInt(4_000_0000000), // 4,000 USDC
      rainfallThreshold: 220,
      ndviBaseline: 6800,
    });
    console.log(`  Farmer3 registered policy #${policy3} for sorghum`);
  } catch (error) {
    console.error('  ⚠️  Policy registration failed:', error);
  }
  console.log('');

  // 3. Submit Oracle Readings
  console.log('🛰️  Submitting oracle readings...');
  try {
    const now = Math.floor(Date.now() / 1000);
    const signature = Buffer.alloc(64); // Dummy signature for testing

    // Rainfall readings for different locations
    await client.submitOracleReading(oracleNode, {
      oracleNode: oracleNode.publicKey(),
      geoCell: '9q5ct',
      readingType: 'Rainfall',
      value: 185, // Below threshold - potential trigger
      timestamp: BigInt(now - 3600),
      signature,
    });
    console.log('  Submitted rainfall reading for 9q5ct: 185mm');

    await client.submitOracleReading(oracleNode, {
      oracleNode: oracleNode.publicKey(),
      geoCell: '9q5cu',
      readingType: 'Rainfall',
      value: 210,
      timestamp: BigInt(now - 3600),
      signature,
    });
    console.log('  Submitted rainfall reading for 9q5cu: 210mm');

    // NDVI readings
    await client.submitOracleReading(oracleNode, {
      oracleNode: oracleNode.publicKey(),
      geoCell: '9q5ct',
      readingType: 'NDVI',
      value: 7200,
      timestamp: BigInt(now - 3600),
      signature,
    });
    console.log('  Submitted NDVI reading for 9q5ct: 0.72');

    await client.submitOracleReading(oracleNode, {
      oracleNode: oracleNode.publicKey(),
      geoCell: '9q5cu',
      readingType: 'NDVI',
      value: 4500, // Below 70% of baseline - crop stress
      timestamp: BigInt(now - 3600),
      signature,
    });
    console.log('  Submitted NDVI reading for 9q5cu: 0.45 (stress)');

    // Soil moisture reading
    await client.submitOracleReading(oracleNode, {
      oracleNode: oracleNode.publicKey(),
      geoCell: '9q5cv',
      readingType: 'SoilMoisture',
      value: 35,
      timestamp: BigInt(now - 3600),
      signature,
    });
    console.log('  Submitted soil moisture reading for 9q5cv: 35%');
  } catch (error) {
    console.error('  ⚠️  Oracle readings failed:', error);
  }
  console.log('');

  // 4. Get Pool Stats
  console.log('📊 Pool Statistics:');
  try {
    const stats = await client.getPoolStats();
    console.log(`  Total Capital: ${stats.totalCapital}`);
    console.log(`  Locked Amount: ${stats.lockedAmount}`);
    console.log(`  Utilization:   ${stats.utilizationRatio / 100}%`);
  } catch (error) {
    console.error('  ⚠️  Could not fetch pool stats');
  }
  console.log('');

  console.log('✨ Seeding complete!');
  console.log('\nYou can now:');
  console.log('  - Query policies with the SDK or contract CLI');
  console.log('  - Evaluate policies for payouts');
  console.log('  - Submit additional oracle readings');
}

async function fundAccount(address: string): Promise<void> {
  const response = await fetch(
    `https://friendbot.stellar.org?addr=${encodeURIComponent(address)}`
  );
  if (!response.ok) {
    throw new Error(`Failed to fund account ${address}`);
  }
}

main().catch((error) => {
  console.error('❌ Seeding failed:', error);
  process.exit(1);
});
