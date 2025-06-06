use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};

declare_id!("SHiFT11111111111111111111111111111111111111");

#[program]
pub mod shift_core {
    use super::*;

    /// Initialize the Shift protocol system
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let protocol_state = &mut ctx.accounts.protocol_state;
        protocol_state.authority = ctx.accounts.authority.key();
        protocol_state.total_devices = 0;
        protocol_state.total_transactions = 0;
        protocol_state.protocol_fee = 0; // No fees in Shift!
        protocol_state.bump = ctx.bumps.protocol_state;
        
        msg!("Shift Protocol initialized - validator-less P2P transactions enabled!");
        Ok(())
    }

    /// Register a hardware device with attestation
    pub fn register_device(
        ctx: Context<RegisterDevice>,
        device_id: [u8; 32],
        attestation_data: AttestationData,
    ) -> Result<()> {
        let device_account = &mut ctx.accounts.device_account;
        let protocol_state = &mut ctx.accounts.protocol_state;

        // Verify attestation signature
        require!(
            verify_attestation(&device_id, &attestation_data),
            ShiftError::InvalidAttestation
        );

        device_account.device_id = device_id;
        device_account.owner = ctx.accounts.owner.key();
        device_account.attestation = attestation_data;
        device_account.is_active = true;
        device_account.key_pool_size = 1000; // Initial key pool
        device_account.used_keys = 0;
        device_account.created_at = Clock::get()?.unix_timestamp;
        device_account.bump = ctx.bumps.device_account;

        protocol_state.total_devices += 1;

        msg!("Hardware device registered: {:?}", device_id);
        Ok(())
    }

    /// Prepare a P2P transaction (receiver generates attested address)
    pub fn prepare_transaction(
        ctx: Context<PrepareTransaction>,
        amount: u64,
        recipient_device_id: [u8; 32],
    ) -> Result<()> {
        let tx_account = &mut ctx.accounts.transaction_account;
        let device_account = &ctx.accounts.device_account;

        // Verify device is active and has available keys
        require!(device_account.is_active, ShiftError::DeviceInactive);
        require!(
            device_account.used_keys < device_account.key_pool_size,
            ShiftError::InsufficientKeyPool
        );

        tx_account.sender = ctx.accounts.sender.key();
        tx_account.amount = amount;
        tx_account.recipient_device_id = recipient_device_id;
        tx_account.status = TransactionStatus::Prepared;
        tx_account.created_at = Clock::get()?.unix_timestamp;
        tx_account.bump = ctx.bumps.transaction_account;

        msg!("P2P transaction prepared: {} tokens", amount);
        Ok(())
    }

    /// Execute P2P transaction with hardware signature
    pub fn execute_transaction(
        ctx: Context<ExecuteTransaction>,
        hardware_signature: [u8; 64],
        key_encumbrance_proof: [u8; 32],
    ) -> Result<()> {
        let tx_account = &mut ctx.accounts.transaction_account;
        let sender_device = &mut ctx.accounts.sender_device;
        let protocol_state = &mut ctx.accounts.protocol_state;

        // Verify transaction is in prepared state
        require!(
            tx_account.status == TransactionStatus::Prepared,
            ShiftError::InvalidTransactionState
        );

        // Verify hardware signature
        let tx_hash = calculate_transaction_hash(tx_account)?;
        require!(
            verify_hardware_signature(&tx_hash, &hardware_signature, sender_device),
            ShiftError::InvalidHardwareSignature
        );

        // Verify key encumbrance (proves key is now destroyed)
        require!(
            verify_key_encumbrance(&key_encumbrance_proof, sender_device),
            ShiftError::InvalidKeyEncumbrance
        );

        // Execute the token transfer
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.sender_token_account.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.sender.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, tx_account.amount)?;

        // Update transaction status and device state
        tx_account.status = TransactionStatus::Completed;
        tx_account.completed_at = Some(Clock::get()?.unix_timestamp);
        tx_account.hardware_signature = Some(hardware_signature);
        
        sender_device.used_keys += 1;
        protocol_state.total_transactions += 1;

        msg!("P2P transaction executed successfully - no network consensus needed!");
        Ok(())
    }

    /// Verify a completed transaction (for record keeping)
    pub fn verify_transaction(
        ctx: Context<VerifyTransaction>,
        transaction_hash: [u8; 32],
    ) -> Result<()> {
        let tx_account = &ctx.accounts.transaction_account;
        
        require!(
            tx_account.status == TransactionStatus::Completed,
            ShiftError::TransactionNotCompleted
        );

        let computed_hash = calculate_transaction_hash(tx_account)?;
        require!(
            computed_hash == transaction_hash,
            ShiftError::HashMismatch
        );

        msg!("Transaction verified: {:?}", transaction_hash);
        Ok(())
    }
}

// Context structs
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + ProtocolState::LEN,
        seeds = [b"protocol"],
        bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(device_id: [u8; 32])]
pub struct RegisterDevice<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + DeviceAccount::LEN,
        seeds = [b"device", device_id.as_ref()],
        bump
    )]
    pub device_account: Account<'info, DeviceAccount>,
    
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PrepareTransaction<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    
    #[account(
        init,
        payer = sender,
        space = 8 + TransactionAccount::LEN,
        seeds = [b"transaction", sender.key().as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub transaction_account: Account<'info, TransactionAccount>,
    
    #[account(
        seeds = [b"device", device_account.device_id.as_ref()],
        bump = device_account.bump,
        constraint = device_account.owner == sender.key()
    )]
    pub device_account: Account<'info, DeviceAccount>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteTransaction<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"transaction", sender.key().as_ref(), &transaction_account.created_at.to_le_bytes()],
        bump = transaction_account.bump
    )]
    pub transaction_account: Account<'info, TransactionAccount>,
    
    #[account(
        mut,
        seeds = [b"device", sender_device.device_id.as_ref()],
        bump = sender_device.bump,
        constraint = sender_device.owner == sender.key()
    )]
    pub sender_device: Account<'info, DeviceAccount>,
    
    #[account(
        mut,
        seeds = [b"protocol"],
        bump = protocol_state.bump
    )]
    pub protocol_state: Account<'info, ProtocolState>,
    
    #[account(mut)]
    pub sender_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct VerifyTransaction<'info> {
    pub transaction_account: Account<'info, TransactionAccount>,
}

// Account structures
#[account]
pub struct ProtocolState {
    pub authority: Pubkey,
    pub total_devices: u64,
    pub total_transactions: u64,
    pub protocol_fee: u64, // Always 0 for Shift!
    pub bump: u8,
}

impl ProtocolState {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct DeviceAccount {
    pub device_id: [u8; 32],
    pub owner: Pubkey,
    pub attestation: AttestationData,
    pub is_active: bool,
    pub key_pool_size: u32,
    pub used_keys: u32,
    pub created_at: i64,
    pub bump: u8,
}

impl DeviceAccount {
    pub const LEN: usize = 32 + 32 + AttestationData::LEN + 1 + 4 + 4 + 8 + 1;
}

#[account]
pub struct TransactionAccount {
    pub sender: Pubkey,
    pub amount: u64,
    pub recipient_device_id: [u8; 32],
    pub status: TransactionStatus,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub hardware_signature: Option<[u8; 64]>,
    pub bump: u8,
}

impl TransactionAccount {
    pub const LEN: usize = 32 + 8 + 32 + 1 + 8 + 9 + 65 + 1;
}

// Data structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AttestationData {
    pub attestation_key: [u8; 32],
    pub signature: [u8; 64],
    pub timestamp: i64,
    pub hardware_type: HardwareType,
}

impl AttestationData {
    pub const LEN: usize = 32 + 64 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum HardwareType {
    ShiftDevice,
    LedgerV2,
    TrustedExecutionEnvironment,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TransactionStatus {
    Prepared,
    Completed,
    Failed,
}

// Helper functions
fn verify_attestation(device_id: &[u8; 32], attestation: &AttestationData) -> bool {
    // In production, this would verify the hardware attestation signature
    // For demo purposes, we'll do basic validation
    attestation.timestamp > 0 && attestation.signature != [0u8; 64]
}

fn verify_hardware_signature(
    tx_hash: &[u8; 32],
    signature: &[u8; 64],
    device: &DeviceAccount,
) -> bool {
    // In production, this would verify the transaction signature using the device's public key
    // For demo purposes, we'll do basic validation
    signature != &[0u8; 64] && device.is_active
}

fn verify_key_encumbrance(proof: &[u8; 32], device: &DeviceAccount) -> bool {
    // In production, this would verify cryptographic proof that the signing key was destroyed
    // For demo purposes, we'll do basic validation
    proof != &[0u8; 32] && device.used_keys < device.key_pool_size
}

fn calculate_transaction_hash(tx: &TransactionAccount) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(tx.sender.as_ref());
    hasher.update(tx.amount.to_le_bytes());
    hasher.update(tx.recipient_device_id);
    hasher.update(tx.created_at.to_le_bytes());
    
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    Ok(hash)
}

// Error handling
#[error_code]
pub enum ShiftError {
    #[msg("Invalid hardware attestation")]
    InvalidAttestation,
    #[msg("Device is inactive")]
    DeviceInactive,
    #[msg("Insufficient key pool")]
    InsufficientKeyPool,
    #[msg("Invalid transaction state")]
    InvalidTransactionState,
    #[msg("Invalid hardware signature")]
    InvalidHardwareSignature,
    #[msg("Invalid key encumbrance proof")]
    InvalidKeyEncumbrance,
    #[msg("Transaction not completed")]
    TransactionNotCompleted,
    #[msg("Hash mismatch")]
    HashMismatch,
} 