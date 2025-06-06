export * from './types';
export * from './core';
export * from './attestation';
export * from './encumbrance';
export * from './p2p';
export * from './utils';

// Main Shift SDK class
export class ShiftSDK {
  // Re-export all functionality for convenience
  static readonly Core = require('./core');
  static readonly Attestation = require('./attestation');
  static readonly Encumbrance = require('./encumbrance');
  static readonly P2P = require('./p2p');
  static readonly Utils = require('./utils');
}

// Version
export const VERSION = "1.0.0";

// Program IDs (these would be the actual deployed program IDs)
export const PROGRAM_IDS = {
  SHIFT_CORE: "SHiFT11111111111111111111111111111111111111",
  SHIFT_ATTESTATION: "ATT3ST111111111111111111111111111111111111",
  SHIFT_ENCUMBRANCE: "ENCUMB111111111111111111111111111111111111",
  SHIFT_P2P: "P2P111111111111111111111111111111111111111111",
};

// Constants
export const CONSTANTS = {
  DEFAULT_KEY_POOL_SIZE: 1000,
  ATTESTATION_VALIDITY_PERIOD: 30 * 24 * 60 * 60, // 30 days in seconds
  MAX_TRANSACTION_AMOUNT: 1_000_000_000_000, // 1 million tokens (6 decimals)
  HARDWARE_SIGNATURE_LENGTH: 64,
  DEVICE_ID_LENGTH: 32,
}; 