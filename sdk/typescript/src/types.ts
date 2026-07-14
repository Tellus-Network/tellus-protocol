export enum PolicyState {
  Active = 'Active',
  Triggered = 'Triggered',
  Expired = 'Expired',
}

export enum ReadingType {
  Rainfall = 'Rainfall',
  NDVI = 'NDVI',
  SoilMoisture = 'SoilMoisture',
}

export interface Policy {
  policyId: bigint;
  farmer: string;
  farmGeohash: string;
  cropType: string;
  seasonStart: bigint;
  seasonEnd: bigint;
  coverageAmount: bigint;
  rainfallThreshold: number;
  ndviBaseline: number;
  state: PolicyState;
}

// Pool statistics (totalCapital and other values are in smallest token units)
export interface PoolStats {
  totalCapital: bigint;
  lockedAmount: bigint;
  totalShares: bigint;
  utilizationRatio: number;
}

export interface AggregatedReading {
  geoCell: string;
  readingType: ReadingType;
  value: number;
  lastUpdated: bigint;
  sampleCount: number;
}

export interface TriggerEvent {
  policyId: bigint;
  triggeredAt: bigint;
  rainfallValue: number;
  ndviValue: number;
  payoutAmount: bigint;
  triggerReason: string;
}

export interface RegisterPolicyParams {
  farmer: string;
  farmGeohash: string;
  cropType: string;
  seasonStart: bigint;
  seasonEnd: bigint;
  coverageAmount: bigint;
  rainfallThreshold: number;
  ndviBaseline: number;
}

export interface SubmitReadingParams {
  oracleNode: string;
  geoCell: string;
  readingType: ReadingType;
  value: number;
  timestamp: bigint;
  signature: Buffer;
}

export interface TellusConfig {
  networkPassphrase: string;
  rpcUrl: string;
  poolContractId: string;
  policyContractId: string;
  oracleContractId: string;
  triggerContractId: string;
}
