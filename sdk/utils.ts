import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

/**
 * Utility functions for the Shift Protocol SDK
 */

/**
 * Generate a random device ID
 */
export function generateDeviceId(): Uint8Array {
  const deviceId = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    deviceId[i] = Math.floor(Math.random() * 256);
  }
  return deviceId;
}

/**
 * Generate a random hardware signature (for testing)
 */
export function generateMockHardwareSignature(): Uint8Array {
  const signature = new Uint8Array(64);
  for (let i = 0; i < 64; i++) {
    signature[i] = Math.floor(Math.random() * 256);
  }
  return signature;
}

/**
 * Generate a mock key encumbrance proof
 */
export function generateMockKeyEncumbranceProof(): Uint8Array {
  const proof = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    proof[i] = Math.floor(Math.random() * 256);
  }
  return proof;
}

/**
 * Find PDA for protocol state
 */
export function findProtocolStatePDA(programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("protocol")],
    programId
  );
}

/**
 * Find PDA for device account
 */
export function findDeviceAccountPDA(
  deviceId: Uint8Array,
  programId: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("device"), Buffer.from(deviceId)],
    programId
  );
}

/**
 * Find PDA for transaction account
 */
export function findTransactionAccountPDA(
  sender: PublicKey,
  timestamp: BN,
  programId: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("transaction"),
      sender.toBuffer(),
      Buffer.from(timestamp.toArray("le", 8))
    ],
    programId
  );
}

/**
 * Format device ID for display
 */
export function formatDeviceId(deviceId: Uint8Array): string {
  const hex = Array.from(deviceId)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  return `${hex.slice(0, 8)}...${hex.slice(-8)}`;
}

/**
 * Format amount for display
 */
export function formatAmount(amount: BN, decimals: number = 6): string {
  const divisor = new BN(10).pow(new BN(decimals));
  const wholePart = amount.div(divisor);
  const fractionalPart = amount.mod(divisor);
  
  if (fractionalPart.isZero()) {
    return wholePart.toString();
  }
  
  const fractionalStr = fractionalPart.toString().padStart(decimals, '0');
  return `${wholePart.toString()}.${fractionalStr.replace(/0+$/, '')}`;
}

/**
 * Validate device ID format
 */
export function isValidDeviceId(deviceId: Uint8Array): boolean {
  return deviceId instanceof Uint8Array && deviceId.length === 32;
}

/**
 * Validate hardware signature format
 */
export function isValidHardwareSignature(signature: Uint8Array): boolean {
  return signature instanceof Uint8Array && signature.length === 64;
}

/**
 * Calculate simple hash (for demo purposes)
 */
export function calculateSimpleHash(data: Uint8Array): Uint8Array {
  const hash = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    let sum = 0;
    for (let j = 0; j < data.length; j++) {
      sum += data[j] * (i + 1) * (j + 1);
    }
    hash[i] = sum % 256;
  }
  return hash;
}

/**
 * Convert Uint8Array to hex string
 */
export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * Convert hex string to Uint8Array
 */
export function fromHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
  }
  return bytes;
}

/**
 * Sleep utility for testing
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Constants for the Shift protocol
 */
export const SHIFT_CONSTANTS = {
  DEVICE_ID_LENGTH: 32,
  HARDWARE_SIGNATURE_LENGTH: 64,
  KEY_ENCUMBRANCE_PROOF_LENGTH: 32,
  ATTESTATION_PROOF_LENGTH: 128,
  DEFAULT_DECIMALS: 6,
  MAX_DEVICES_PER_POOL: 1000,
} as const; 