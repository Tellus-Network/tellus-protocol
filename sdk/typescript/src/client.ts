import { Keypair } from '@stellar/stellar-sdk';
import {
  PoolContractClient,
  PolicyContractClient,
  OracleContractClient,
  TriggerContractClient,
} from './contracts';
import {
  TellusConfig,
  RegisterPolicyParams,
  SubmitReadingParams,
  Policy,
  PoolStats,
  AggregatedReading,
} from './types';

/**
 * Main client for interacting with Tellus Protocol contracts
 */
export class TellusClient {
  private poolClient: PoolContractClient;
  private policyClient: PolicyContractClient;
  private oracleClient: OracleContractClient;
  private triggerClient: TriggerContractClient;

  constructor(config: TellusConfig) {
    this.poolClient = new PoolContractClient(
      config.poolContractId,
      config.rpcUrl,
      config.networkPassphrase
    );

    this.policyClient = new PolicyContractClient(
      config.policyContractId,
      config.rpcUrl,
      config.networkPassphrase
    );

    this.oracleClient = new OracleContractClient(
      config.oracleContractId,
      config.rpcUrl,
      config.networkPassphrase
    );

    this.triggerClient = new TriggerContractClient(
      config.triggerContractId,
      config.rpcUrl,
      config.networkPassphrase
    );
  }

  // Pool methods

  /**
   * Get current pool statistics
   */
  async getPoolStats(): Promise<PoolStats> {
    return await this.poolClient.getPoolStats();
  }

  /**
   * Deposit capital into the liquidity pool
   * @param keypair - Keypair of the liquidity provider
   * @param amount - Amount to deposit (in stroops, smallest unit)
   * @returns Number of LP shares minted
   */
  async deposit(keypair: Keypair, amount: bigint): Promise<bigint> {
    return await this.poolClient.deposit(keypair, amount);
  }

  /**
   * Withdraw capital from the liquidity pool
   * @param keypair - Keypair of the liquidity provider
   * @param shares - Number of LP shares to burn
   * @returns Amount of capital returned
   */
  async withdraw(keypair: Keypair, shares: bigint): Promise<bigint> {
    return await this.poolClient.withdraw(keypair, shares);
  }

  // Policy methods

  /**
   * Register a new parametric insurance policy
   * @param keypair - Keypair of the farmer
   * @param params - Policy parameters
   * @returns Policy ID
   */
  async registerPolicy(
    keypair: Keypair,
    params: RegisterPolicyParams
  ): Promise<bigint> {
    return await this.policyClient.registerPolicy(keypair, params);
  }

  /**
   * Get policy details by ID
   * @param policyId - Policy ID
   * @returns Policy details
   */
  async getPolicy(policyId: bigint): Promise<Policy> {
    return await this.policyClient.getPolicy(policyId);
  }

  /**
   * List all policies for a farmer
   * @param farmerAddress - Farmer's Stellar address
   * @returns Array of policy IDs
   */
  async listFarmerPolicies(farmerAddress: string): Promise<bigint[]> {
    return await this.policyClient.listPoliciesByFarmer(farmerAddress);
  }

  // Oracle methods

  /**
   * Submit an oracle reading
   * @param keypair - Keypair of the oracle node
   * @param params - Reading parameters
   */
  async submitOracleReading(
    keypair: Keypair,
    params: SubmitReadingParams
  ): Promise<void> {
    return await this.oracleClient.submitReading(keypair, {
      geoCell: params.geoCell,
      readingType: params.readingType,
      value: params.value,
      timestamp: params.timestamp,
      signature: params.signature,
    });
  }

  /**
   * Get aggregated oracle reading for a location
   * @param geoCell - Geohash cell identifier
   * @param readingType - Type of reading (Rainfall, NDVI, SoilMoisture)
   * @returns Aggregated reading data
   */
  async getAggregatedReading(
    geoCell: string,
    readingType: string
  ): Promise<AggregatedReading> {
    return await this.oracleClient.getAggregated(geoCell, readingType);
  }

  // Trigger methods

  /**
   * Evaluate a policy against oracle data and trigger payout if conditions are met
   * @param keypair - Keypair to sign the transaction (can be anyone)
   * @param policyId - Policy ID to evaluate
   */
  async evaluatePolicy(keypair: Keypair, policyId: bigint): Promise<void> {
    return await this.triggerClient.evaluatePolicy(keypair, policyId);
  }

  /**
   * Check if a policy has been triggered
   * @param policyId - Policy ID
   * @returns True if triggered, false otherwise
   */
  async isTriggered(policyId: bigint): Promise<boolean> {
    return await this.triggerClient.isTriggered(policyId);
  }
}
