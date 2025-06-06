use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};

declare_id!("P2P111111111111111111111111111111111111111111");

#[program]
pub mod shift_p2p {
    use super::*;

    /// Initialize the P2P system
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let p2p_authority = &mut ctx.accounts.p2p_authority;
        p2p_authority.authority = ctx.accounts.authority.key();
        p2p_authority.total_channels = 0;
        p2p_authority.total_transactions = 0;
        p2p_authority.total_volume = 0;
        p2p_authority.bump = ctx.bumps.p2p_authority;
        
        msg!("Shift P2P system initialized - direct transfers enabled!");
        Ok(())
    }

    /// Create a P2P payment channel between two parties
    pub fn create_channel(
        ctx: Context<CreateChannel>,
        channel_id: [u8; 32],
        counterparty: Pubkey,
        initial_deposit: u64,
        channel_config: ChannelConfig,
    ) -> Result<()> {
        let channel = &mut ctx.accounts.channel;
        let p2p_authority = &mut ctx.accounts.p2p_authority;

        channel.channel_id = channel_id;
        channel.party_a = ctx.accounts.creator.key();
        channel.party_b = counterparty;
        channel.balance_a = initial_deposit;
        channel.balance_b = 0;
        channel.config = channel_config;
        channel.status = ChannelStatus::Active;
        channel.created_at = Clock::get()?.unix_timestamp;
        channel.last_update = Clock::get()?.unix_timestamp;
        channel.transaction_count = 0;
        channel.bump = ctx.bumps.channel;

        // Transfer initial deposit to channel
        if initial_deposit > 0 {
            let transfer_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.creator_token_account.to_account_info(),
                    to: ctx.accounts.channel_token_account.to_account_info(),
                    authority: ctx.accounts.creator.to_account_info(),
                },
            );
            token::transfer(transfer_ctx, initial_deposit)?;
        }

        p2p_authority.total_channels += 1;

        msg!("P2P channel created: {:?}", channel_id);
        Ok(())
    }

    /// Execute a direct P2P transaction
    pub fn execute_p2p_transaction(
        ctx: Context<ExecuteP2PTransaction>,
        channel_id: [u8; 32],
        amount: u64,
        recipient_address: [u8; 32], // Hardware-attested address
        hardware_signature: [u8; 64],
        attestation_proof: [u8; 128],
    ) -> Result<()> {
        let channel = &mut ctx.accounts.channel;
        let transaction_record = &mut ctx.accounts.transaction_record;
        let p2p_authority = &mut ctx.accounts.p2p_authority;

        // Verify channel is active
        require!(
            channel.status == ChannelStatus::Active,
            P2PError::ChannelInactive
        );

        require!(
            channel.channel_id == channel_id,
            P2PError::ChannelIdMismatch
        );

        // Verify sender is a channel participant
        let is_party_a = channel.party_a == ctx.accounts.sender.key();
        let is_party_b = channel.party_b == ctx.accounts.sender.key();
        require!(
            is_party_a || is_party_b,
            P2PError::UnauthorizedSender
        );

        // Verify sufficient balance
        let sender_balance = if is_party_a { channel.balance_a } else { channel.balance_b };
        require!(
            sender_balance >= amount,
            P2PError::InsufficientBalance
        );

        // Verify hardware attestation
        require!(
            verify_hardware_attestation(&recipient_address, &attestation_proof),
            P2PError::InvalidAttestation
        );

        // Verify hardware signature
        let tx_hash = calculate_p2p_transaction_hash(
            &channel_id,
            &ctx.accounts.sender.key(),
            amount,
            &recipient_address,
        )?;
        require!(
            verify_p2p_signature(&tx_hash, &hardware_signature),
            P2PError::InvalidSignature
        );

        // Create transaction record
        transaction_record.channel_id = channel_id;
        transaction_record.sender = ctx.accounts.sender.key();
        transaction_record.recipient_address = recipient_address;
        transaction_record.amount = amount;
        transaction_record.hardware_signature = hardware_signature;
        transaction_record.attestation_proof = attestation_proof;
        transaction_record.timestamp = Clock::get()?.unix_timestamp;
        transaction_record.status = TransactionStatus::Completed;
        transaction_record.bump = ctx.bumps.transaction_record;

        // Update channel balances
        if is_party_a {
            channel.balance_a -= amount;
        } else {
            channel.balance_b -= amount;
        }

        channel.transaction_count += 1;
        channel.last_update = Clock::get()?.unix_timestamp;

        p2p_authority.total_transactions += 1;
        p2p_authority.total_volume += amount;

        msg!("P2P transaction executed: {} tokens sent directly", amount);
        Ok(())
    }

    /// Close a P2P channel and settle balances
    pub fn close_channel(
        ctx: Context<CloseChannel>,
        channel_id: [u8; 32],
        final_balance_a: u64,
        final_balance_b: u64,
        closing_signatures: [u8; 128], // Both parties must sign
    ) -> Result<()> {
        let channel = &mut ctx.accounts.channel;

        require!(
            channel.channel_id == channel_id,
            P2PError::ChannelIdMismatch
        );

        require!(
            channel.status == ChannelStatus::Active,
            P2PError::ChannelInactive
        );

        // Verify closing signatures from both parties
        require!(
            verify_closing_signatures(&channel_id, &closing_signatures, channel),
            P2PError::InvalidClosingSignatures
        );

        // Verify final balances sum correctly
        require!(
            final_balance_a + final_balance_b <= channel.balance_a + channel.balance_b,
            P2PError::InvalidFinalBalances
        );

        // Transfer final balances back to parties
        if final_balance_a > 0 {
            let transfer_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.channel_token_account.to_account_info(),
                    to: ctx.accounts.party_a_token_account.to_account_info(),
                    authority: ctx.accounts.channel.to_account_info(),
                },
            );
            token::transfer(transfer_ctx, final_balance_a)?;
        }

        if final_balance_b > 0 {
            let transfer_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.channel_token_account.to_account_info(),
                    to: ctx.accounts.party_b_token_account.to_account_info(),
                    authority: ctx.accounts.channel.to_account_info(),
                },
            );
            token::transfer(transfer_ctx, final_balance_b)?;
        }

        channel.status = ChannelStatus::Closed;
        channel.last_update = Clock::get()?.unix_timestamp;

        msg!("P2P channel closed: {:?}", channel_id);
        Ok(())
    }

    /// Dispute a transaction (for dispute resolution)
    pub fn dispute_transaction(
        ctx: Context<DisputeTransaction>,
        transaction_id: [u8; 32],
        dispute_evidence: [u8; 256],
        dispute_reason: DisputeReason,
    ) -> Result<()> {
        let transaction_record = &mut ctx.accounts.transaction_record;
        let dispute_record = &mut ctx.accounts.dispute_record;

        require!(
            transaction_record.status == TransactionStatus::Completed,
            P2PError::TransactionNotCompleted
        );

        dispute_record.transaction_id = transaction_id;
        dispute_record.disputer = ctx.accounts.disputer.key();
        dispute_record.reason = dispute_reason;
        dispute_record.evidence = dispute_evidence;
        dispute_record.status = DisputeStatus::Open;
        dispute_record.created_at = Clock::get()?.unix_timestamp;
        dispute_record.bump = ctx.bumps.dispute_record;

        transaction_record.status = TransactionStatus::Disputed;

        msg!("Transaction disputed: {:?}", transaction_id);
        Ok(())
    }

    /// Verify a completed P2P transaction
    pub fn verify_p2p_transaction(
        ctx: Context<VerifyP2PTransaction>,
        transaction_id: [u8; 32],
        expected_hash: [u8; 32],
    ) -> Result<bool> {
        let transaction_record = &ctx.accounts.transaction_record;

        // Calculate actual transaction hash
        let computed_hash = calculate_p2p_transaction_hash(
            &transaction_record.channel_id,
            &transaction_record.sender,
            transaction_record.amount,
            &transaction_record.recipient_address,
        )?;

        require!(
            computed_hash == expected_hash,
            P2PError::HashMismatch
        );

        require!(
            transaction_record.status == TransactionStatus::Completed,
            P2PError::TransactionNotCompleted
        );

        msg!("P2P transaction verified: {:?}", transaction_id);
        Ok(true)
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
        space = 8 + P2PAuthority::LEN,
        seeds = [b"p2p_authority"],
        bump
    )]
    pub p2p_authority: Account<'info, P2PAuthority>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(channel_id: [u8; 32])]
pub struct CreateChannel<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        init,
        payer = creator,
        space = 8 + Channel::LEN,
        seeds = [b"channel", channel_id.as_ref()],
        bump
    )]
    pub channel: Account<'info, Channel>,
    
    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub channel_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"p2p_authority"],
        bump = p2p_authority.bump
    )]
    pub p2p_authority: Account<'info, P2PAuthority>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(channel_id: [u8; 32])]
pub struct ExecuteP2PTransaction<'info> {
    #[account(mut)]
    pub sender: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"channel", channel_id.as_ref()],
        bump = channel.bump
    )]
    pub channel: Account<'info, Channel>,
    
    #[account(
        init,
        payer = sender,
        space = 8 + TransactionRecord::LEN,
        seeds = [b"transaction", channel_id.as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub transaction_record: Account<'info, TransactionRecord>,
    
    #[account(
        mut,
        seeds = [b"p2p_authority"],
        bump = p2p_authority.bump
    )]
    pub p2p_authority: Account<'info, P2PAuthority>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(channel_id: [u8; 32])]
pub struct CloseChannel<'info> {
    #[account(mut)]
    pub closer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"channel", channel_id.as_ref()],
        bump = channel.bump
    )]
    pub channel: Account<'info, Channel>,
    
    #[account(mut)]
    pub channel_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub party_a_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub party_b_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(transaction_id: [u8; 32])]
pub struct DisputeTransaction<'info> {
    #[account(mut)]
    pub disputer: Signer<'info>,
    
    #[account(mut)]
    pub transaction_record: Account<'info, TransactionRecord>,
    
    #[account(
        init,
        payer = disputer,
        space = 8 + DisputeRecord::LEN,
        seeds = [b"dispute", transaction_id.as_ref()],
        bump
    )]
    pub dispute_record: Account<'info, DisputeRecord>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyP2PTransaction<'info> {
    pub transaction_record: Account<'info, TransactionRecord>,
}

// Account structures
#[account]
pub struct P2PAuthority {
    pub authority: Pubkey,
    pub total_channels: u64,
    pub total_transactions: u64,
    pub total_volume: u64,
    pub bump: u8,
}

impl P2PAuthority {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct Channel {
    pub channel_id: [u8; 32],
    pub party_a: Pubkey,
    pub party_b: Pubkey,
    pub balance_a: u64,
    pub balance_b: u64,
    pub config: ChannelConfig,
    pub status: ChannelStatus,
    pub created_at: i64,
    pub last_update: i64,
    pub transaction_count: u32,
    pub bump: u8,
}

impl Channel {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + ChannelConfig::LEN + 1 + 8 + 8 + 4 + 1;
}

#[account]
pub struct TransactionRecord {
    pub channel_id: [u8; 32],
    pub sender: Pubkey,
    pub recipient_address: [u8; 32],
    pub amount: u64,
    pub hardware_signature: [u8; 64],
    pub attestation_proof: [u8; 128],
    pub timestamp: i64,
    pub status: TransactionStatus,
    pub bump: u8,
}

impl TransactionRecord {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 64 + 128 + 8 + 1 + 1;
}

#[account]
pub struct DisputeRecord {
    pub transaction_id: [u8; 32],
    pub disputer: Pubkey,
    pub reason: DisputeReason,
    pub evidence: [u8; 256],
    pub status: DisputeStatus,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
    pub bump: u8,
}

impl DisputeRecord {
    pub const LEN: usize = 32 + 32 + 1 + 256 + 1 + 8 + 9 + 1;
}

// Data structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ChannelConfig {
    pub dispute_timeout: i64,
    pub auto_close_timeout: i64,
    pub max_transaction_amount: u64,
    pub require_dual_signatures: bool,
}

impl ChannelConfig {
    pub const LEN: usize = 8 + 8 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ChannelStatus {
    Active,
    Closing,
    Closed,
    Disputed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Disputed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum DisputeReason {
    InvalidSignature,
    InvalidAttestation,
    DoubleSpending,
    FraudulentTransaction,
    Other,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DisputeStatus {
    Open,
    UnderReview,
    Resolved,
    Dismissed,
}

// Helper functions
fn verify_hardware_attestation(address: &[u8; 32], proof: &[u8; 128]) -> bool {
    // In production, this would verify the hardware attestation
    // For demo purposes, we'll do basic validation
    address != &[0u8; 32] && proof != &[0u8; 128]
}

fn verify_p2p_signature(tx_hash: &[u8; 32], signature: &[u8; 64]) -> bool {
    // In production, this would verify the hardware signature
    // For demo purposes, we'll do basic validation
    signature != &[0u8; 64] && tx_hash != &[0u8; 32]
}

fn verify_closing_signatures(channel_id: &[u8; 32], signatures: &[u8; 128], channel: &Channel) -> bool {
    // In production, this would verify both parties signed the closing transaction
    // For demo purposes, we'll do basic validation
    signatures != &[0u8; 128] && channel_id != &[0u8; 32]
}

fn calculate_p2p_transaction_hash(
    channel_id: &[u8; 32],
    sender: &Pubkey,
    amount: u64,
    recipient: &[u8; 32],
) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(channel_id);
    hasher.update(sender.as_ref());
    hasher.update(amount.to_le_bytes());
    hasher.update(recipient);
    hasher.update(b"SHIFT_P2P_TRANSACTION");
    
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    Ok(hash)
}

// Error handling
#[error_code]
pub enum P2PError {
    #[msg("Channel is inactive")]
    ChannelInactive,
    #[msg("Channel ID mismatch")]
    ChannelIdMismatch,
    #[msg("Unauthorized sender")]
    UnauthorizedSender,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Invalid attestation")]
    InvalidAttestation,
    #[msg("Invalid signature")]
    InvalidSignature,
    #[msg("Invalid closing signatures")]
    InvalidClosingSignatures,
    #[msg("Invalid final balances")]
    InvalidFinalBalances,
    #[msg("Transaction not completed")]
    TransactionNotCompleted,
    #[msg("Hash mismatch")]
    HashMismatch,
} 