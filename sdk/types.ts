import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

// Core Protocol Types
export interface ProtocolState {
  authority: PublicKey;
  totalDevices: BN;
  totalTransactions: BN;
  protocolFee: BN;
  bump: number;
}

export interface DeviceAccount {
  deviceId: Uint8Array;
  owner: PublicKey;
  attestation: AttestationData;
  isActive: boolean;
  keyPoolSize: number;
  usedKeys: number;
  createdAt: BN;
  bump: number;
}

export interface TransactionAccount {
  sender: PublicKey;
  amount: BN;
  recipientDeviceId: Uint8Array;
  status: TransactionStatus;
  createdAt: BN;
  completedAt?: BN;
  hardwareSignature?: Uint8Array;
  bump: number;
}

// Attestation Types
export interface AttestationData {
  attestationKey: Uint8Array;
  signature: Uint8Array;
  timestamp: BN;
  hardwareType: HardwareType;
}

export interface AttestationRecord {
  deviceId: Uint8Array;
  manufacturerId: Uint8Array;
  attestationQuote: AttestationQuote;
  deviceCertificate: Uint8Array;
  status: AttestationStatus;
  createdAt: BN;
  expiresAt: BN;
  revokedAt?: BN;
  revocationReason?: RevocationReason;
  bump: number;
}

export interface AttestationQuote {
  version: number;
  signature: Uint8Array;
  publicKey: Uint8Array;
  nonce: Uint8Array;
  timestamp: BN;
  measurements: Uint8Array[];
}

// Encumbrance Types
export interface KeyPool {
  deviceId: Uint8Array;
  owner: PublicKey;
  totalKeys: number;
  availableKeys: number;
  usedKeys: number;
  publicKeys: Uint8Array[];
  encumberedKeys: number[];
  createdAt: BN;
  bump: number;
}

export interface EncumbranceRecord {
  deviceId: Uint8Array;
  keyIndex: number;
  publicKey: Uint8Array;
  transactionHash: Uint8Array;
  destructionProof: KeyDestructionProof;
  encumberedAt: BN;
  status: EncumbranceStatus;
  bump: number;
}

export interface KeyDestructionProof {
  proofType: ProofType;
  proofData: Uint8Array;
  timestamp: BN;
  nonce: Uint8Array;
  hardwareSignature: Uint8Array;
}

// P2P Types
export interface Channel {
  channelId: Uint8Array;
  partyA: PublicKey;
  partyB: PublicKey;
  balanceA: BN;
  balanceB: BN;
  config: ChannelConfig;
  status: ChannelStatus;
  createdAt: BN;
  lastUpdate: BN;
  transactionCount: number;
  bump: number;
}

export interface ChannelConfig {
  disputeTimeout: BN;
  autoCloseTimeout: BN;
  maxTransactionAmount: BN;
  requireDualSignatures: boolean;
}

export interface TransactionRecord {
  channelId: Uint8Array;
  sender: PublicKey;
  recipientAddress: Uint8Array;
  amount: BN;
  hardwareSignature: Uint8Array;
  attestationProof: Uint8Array;
  timestamp: BN;
  status: TransactionStatus;
  bump: number;
}

// Enums
export enum HardwareType {
  ShiftDevice = "ShiftDevice",
  LedgerV2 = "LedgerV2",
  TrustedExecutionEnvironment = "TrustedExecutionEnvironment",
}

export enum TransactionStatus {
  Prepared = "Prepared",
  Completed = "Completed",
  Failed = "Failed",
  Disputed = "Disputed",
}

export enum AttestationStatus {
  Valid = "Valid",
  Expired = "Expired",
  Revoked = "Revoked",
  Pending = "Pending",
}

export enum RevocationReason {
  Compromised = "Compromised",
  Expired = "Expired",
  ManufacturerRevoked = "ManufacturerRevoked",
  UserRequested = "UserRequested",
  Other = "Other",
}

export enum ProofType {
  ZeroKnowledge = "ZeroKnowledge",
  HardwareAttestation = "HardwareAttestation",
  CryptographicCommitment = "CryptographicCommitment",
}

export enum EncumbranceStatus {
  Encumbered = "Encumbered",
  Verified = "Verified",
  Disputed = "Disputed",
}

export enum ChannelStatus {
  Active = "Active",
  Closing = "Closing",
  Closed = "Closed",
  Disputed = "Disputed",
}

export enum DisputeReason {
  InvalidSignature = "InvalidSignature",
  InvalidAttestation = "InvalidAttestation",
  DoubleSpending = "DoubleSpending",
  FraudulentTransaction = "FraudulentTransaction",
  Other = "Other",
}

export enum DisputeStatus {
  Open = "Open",
  UnderReview = "UnderReview",
  Resolved = "Resolved",
  Dismissed = "Dismissed",
}

// Utility Types
export interface ShiftConfig {
  connection: any; // Solana Connection
  wallet: any; // Wallet adapter
  programIds: {
    core: string;
    attestation: string;
    encumbrance: string;
    p2p: string;
  };
}

export interface HardwareDevice {
  deviceId: Uint8Array;
  publicKey: PublicKey;
  attestationData: AttestationData;
  keyPool: KeyPool;
}

export interface P2PTransactionParams {
  channelId: Uint8Array;
  amount: BN;
  recipientAddress: Uint8Array;
  hardwareSignature: Uint8Array;
  attestationProof: Uint8Array;
} 