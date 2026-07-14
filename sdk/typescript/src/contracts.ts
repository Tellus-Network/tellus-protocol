import {
  Contract,
  SorobanRpc,
  TransactionBuilder,
  BASE_FEE,
  xdr,
  Address,
  nativeToScVal,
  scValToNative,
  Keypair,
} from '@stellar/stellar-sdk';
import { Policy, PoolStats, AggregatedReading } from './types';

export class ContractClient {
  protected contract: Contract;
  protected server: SorobanRpc.Server;
  protected networkPassphrase: string;

  constructor(contractId: string, rpcUrl: string, networkPassphrase: string) {
    this.contract = new Contract(contractId);
    this.server = new SorobanRpc.Server(rpcUrl);
    this.networkPassphrase = networkPassphrase;
  }

  protected async simulateTransaction(
    sourceAccount: string,
    operation: xdr.Operation
  ): Promise<SorobanRpc.Api.SimulateTransactionResponse> {
    const account = await this.server.getAccount(sourceAccount);
    const transaction = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(operation)
      .setTimeout(30)
      .build();

    return await this.server.simulateTransaction(transaction);
  }

  protected async submitTransaction(
    sourceKeypair: Keypair,
    operation: xdr.Operation
  ): Promise<SorobanRpc.Api.GetTransactionResponse> {
    const account = await this.server.getAccount(sourceKeypair.publicKey());
    const transaction = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(operation)
      .setTimeout(30)
      .build();

    const simulated = await this.server.simulateTransaction(transaction);

    if (SorobanRpc.Api.isSimulationError(simulated)) {
      throw new Error(`Simulation failed: ${simulated.error}`);
    }

    const prepared = SorobanRpc.assembleTransaction(transaction, simulated).build();
    prepared.sign(sourceKeypair);

    const sent = await this.server.sendTransaction(prepared);

    if (sent.status === 'ERROR') {
      throw new Error(`Transaction failed: ${sent.errorResult}`);
    }

    // Poll for result
    let result = await this.server.getTransaction(sent.hash);
    while (result.status === SorobanRpc.Api.GetTransactionStatus.NOT_FOUND) {
      await new Promise((resolve) => setTimeout(resolve, 500));
      result = await this.server.getTransaction(sent.hash);
    }

    return result;
  }
}

export class PoolContractClient extends ContractClient {
  async getProviderValue(provider: string): Promise<bigint> {
    const operation = this.contract.call(
      'get_provider_value',
      Address.fromString(provider).toScVal()
    );
    const result = await this.simulateTransaction(provider, operation);
    if (SorobanRpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result!.retval);
    }
    throw new Error('Failed to get provider value');
  }

  async getProviderShares(provider: string): Promise<bigint> {
    const operation = this.contract.call(
      'get_provider_shares',
      Address.fromString(provider).toScVal()
    );
    const result = await this.simulateTransaction(provider, operation);
    if (SorobanRpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result!.retval);
    }
    throw new Error('Failed to get provider shares');
  }

  async getPoolStats(): Promise<PoolStats> {
    const operation = this.contract.call('get_pool_stats');
    const result = await this.simulateTransaction(
      'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
      operation
    );

    if (SorobanRpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result!.retval);
    }

    throw new Error('Failed to get pool stats');
  }

  async deposit(sourceKeypair: Keypair, amount: bigint): Promise<bigint> {
    const operation = this.contract.call(
      'deposit',
      Address.fromString(sourceKeypair.publicKey()).toScVal(),
      nativeToScVal(amount, { type: 'i128' })
    );

    const result = await this.submitTransaction(sourceKeypair, operation);

    if (result.status === SorobanRpc.Api.GetTransactionStatus.SUCCESS) {
      const returnValue = result.returnValue;
      return scValToNative(returnValue!);
    }

    throw new Error('Deposit failed');
  }

  async withdraw(sourceKeypair: Keypair, shares: bigint): Promise<bigint> {
    const operation = this.contract.call(
      'withdraw',
      Address.fromString(sourceKeypair.publicKey()).toScVal(),
      nativeToScVal(shares, { type: 'i128' })
    );

    const result = await this.submitTransaction(sourceKeypair, operation);

    if (result.status === SorobanRpc.Api.GetTransactionStatus.SUCCESS) {
      const returnValue = result.returnValue;
      return scValToNative(returnValue!);
    }

    throw new Error('Withdraw failed');
  }
}

export class PolicyContractClient extends ContractClient {
  async registerPolicy(
    sourceKeypair: Keypair,
    params: {
      farmer: string;
      farmGeohash: string;
      cropType: string;
      seasonStart: bigint;
      seasonEnd: bigint;
      coverageAmount: bigint;
      rainfallThreshold: number;
      ndviBaseline: number;
    }
  ): Promise<bigint> {
    const operation = this.contract.call(
      'register_policy',
      Address.fromString(params.farmer).toScVal(),
      nativeToScVal(params.farmGeohash, { type: 'string' }),
      nativeToScVal(params.cropType, { type: 'string' }),
      nativeToScVal(params.seasonStart, { type: 'u64' }),
      nativeToScVal(params.seasonEnd, { type: 'u64' }),
      nativeToScVal(params.coverageAmount, { type: 'i128' }),
      nativeToScVal(params.rainfallThreshold, { type: 'u32' }),
      nativeToScVal(params.ndviBaseline, { type: 'u32' })
    );

    const result = await this.submitTransaction(sourceKeypair, operation);

    if (result.status === SorobanRpc.Api.GetTransactionStatus.SUCCESS) {
      return scValToNative(result.returnValue!);
    }

    throw new Error('Policy registration failed');
  }

  async getPolicy(policyId: bigint): Promise<Policy> {
    const operation = this.contract.call(
      'get_policy',
      nativeToScVal(policyId, { type: 'u64' })
    );

    const result = await this.simulateTransaction(
      'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
      operation
    );

    if (SorobanRpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result!.retval);
    }

    throw new Error('Failed to get policy');
  }

  async listPoliciesByFarmer(farmer: string): Promise<bigint[]> {
    const operation = this.contract.call(
      'list_policies_by_farmer',
      Address.fromString(farmer).toScVal()
    );

    const result = await this.simulateTransaction(
      'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
      operation
    );

    if (SorobanRpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result!.retval);
    }

    throw new Error('Failed to list policies');
  }
}

export class OracleContractClient extends ContractClient {
  async submitReading(
    sourceKeypair: Keypair,
    params: {
      geoCell: string;
      readingType: string;
      value: number;
      timestamp: bigint;
      signature: Buffer;
    }
  ): Promise<void> {
    const operation = this.contract.call(
      'submit_reading',
      Address.fromString(sourceKeypair.publicKey()).toScVal(),
      nativeToScVal(params.geoCell, { type: 'string' }),
      nativeToScVal(params.readingType, { type: 'symbol' }),
      nativeToScVal(params.value, { type: 'u32' }),
      nativeToScVal(params.timestamp, { type: 'u64' }),
      nativeToScVal(params.signature, { type: 'bytes' })
    );

    const result = await this.submitTransaction(sourceKeypair, operation);

    if (result.status !== SorobanRpc.Api.GetTransactionStatus.SUCCESS) {
      throw new Error('Submit reading failed');
    }
  }

  async getAggregated(geoCell: string, readingType: string): Promise<AggregatedReading> {
    const operation = this.contract.call(
      'get_aggregated',
      nativeToScVal(geoCell, { type: 'string' }),
      nativeToScVal(readingType, { type: 'symbol' })
    );

    const result = await this.simulateTransaction(
      'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
      operation
    );

    if (SorobanRpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result!.retval);
    }

    throw new Error('Failed to get aggregated reading');
  }
}

export class TriggerContractClient extends ContractClient {
  async evaluatePolicy(sourceKeypair: Keypair, policyId: bigint): Promise<void> {
    const operation = this.contract.call(
      'evaluate_policy',
      nativeToScVal(policyId, { type: 'u64' })
    );

    const result = await this.submitTransaction(sourceKeypair, operation);

    if (result.status !== SorobanRpc.Api.GetTransactionStatus.SUCCESS) {
      throw new Error('Policy evaluation failed');
    }
  }

  async isTriggered(policyId: bigint): Promise<boolean> {
    const operation = this.contract.call(
      'is_triggered',
      nativeToScVal(policyId, { type: 'u64' })
    );

    const result = await this.simulateTransaction(
      'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF',
      operation
    );

    if (SorobanRpc.Api.isSimulationSuccess(result)) {
      return scValToNative(result.result!.retval);
    }

    throw new Error('Failed to check trigger status');
  }
}
