use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};

declare_id!("ATT3ST111111111111111111111111111111111111");

#[program]
pub mod shift_attestation {
    use super::*;

    /// Initialize the attestation system
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let attestation_authority = &mut ctx.accounts.attestation_authority;
        attestation_authority.authority = ctx.accounts.authority.key();
        attestation_authority.trusted_manufacturers = Vec::new();
        attestation_authority.total_attestations = 0;
        attestation_authority.bump = ctx.bumps.attestation_authority;
        
        msg!("Shift Attestation system initialized");
        Ok(())
    }

    /// Add a trusted hardware manufacturer
    pub fn add_trusted_manufacturer(
        ctx: Context<AddTrustedManufacturer>,
        manufacturer_id: [u8; 32],
        name: String,
        public_key: [u8; 32],
    ) -> Result<()> {
        let manufacturer_account = &mut ctx.accounts.manufacturer_account;
        let attestation_authority = &mut ctx.accounts.attestation_authority;

        manufacturer_account.manufacturer_id = manufacturer_id;
        manufacturer_account.name = name;
        manufacturer_account.public_key = public_key;
        manufacturer_account.is_active = true;
        manufacturer_account.devices_attested = 0;
        manufacturer_account.created_at = Clock::get()?.unix_timestamp;
        manufacturer_account.bump = ctx.bumps.manufacturer_account;

        attestation_authority.trusted_manufacturers.push(manufacturer_id);

        msg!("Trusted manufacturer added: {:?}", manufacturer_id);
        Ok(())
    }

    /// Create a remote attestation for a hardware device
    pub fn create_attestation(
        ctx: Context<CreateAttestation>,
        device_id: [u8; 32],
        manufacturer_id: [u8; 32],
        attestation_quote: AttestationQuote,
        device_certificate: [u8; 1024], // Device certificate from manufacturer
    ) -> Result<()> {
        let attestation_record = &mut ctx.accounts.attestation_record;
        let manufacturer = &mut ctx.accounts.manufacturer_account;
        let attestation_authority = &mut ctx.accounts.attestation_authority;

        // Verify manufacturer is trusted
        require!(manufacturer.is_active, AttestationError::UntrustedManufacturer);
        require!(
            manufacturer.manufacturer_id == manufacturer_id,
            AttestationError::ManufacturerMismatch
        );

        // Verify attestation quote signature
        require!(
            verify_attestation_quote(&device_id, &attestation_quote, &manufacturer.public_key),
            AttestationError::InvalidAttestationQuote
        );

        // Verify device certificate
        require!(
            verify_device_certificate(&device_certificate, &manufacturer.public_key),
            AttestationError::InvalidDeviceCertificate
        );

        // Create attestation record
        attestation_record.device_id = device_id;
        attestation_record.manufacturer_id = manufacturer_id;
        attestation_record.attestation_quote = attestation_quote;
        attestation_record.device_certificate = device_certificate;
        attestation_record.status = AttestationStatus::Valid;
        attestation_record.created_at = Clock::get()?.unix_timestamp;
        attestation_record.expires_at = Clock::get()?.unix_timestamp + 86400 * 30; // 30 days
        attestation_record.bump = ctx.bumps.attestation_record;

        manufacturer.devices_attested += 1;
        attestation_authority.total_attestations += 1;

        msg!("Device attestation created: {:?}", device_id);
        Ok(())
    }

    /// Verify an existing attestation
    pub fn verify_attestation(
        ctx: Context<VerifyAttestation>,
        device_id: [u8; 32],
    ) -> Result<bool> {
        let attestation_record = &ctx.accounts.attestation_record;
        let current_time = Clock::get()?.unix_timestamp;

        // Check if attestation exists and is valid
        require!(
            attestation_record.device_id == device_id,
            AttestationError::DeviceIdMismatch
        );
        
        require!(
            attestation_record.status == AttestationStatus::Valid,
            AttestationError::InvalidAttestation
        );

        require!(
            current_time < attestation_record.expires_at,
            AttestationError::AttestationExpired
        );

        msg!("Device attestation verified: {:?}", device_id);
        Ok(true)
    }

    /// Revoke an attestation (in case of compromise)
    pub fn revoke_attestation(
        ctx: Context<RevokeAttestation>,
        device_id: [u8; 32],
        reason: RevocationReason,
    ) -> Result<()> {
        let attestation_record = &mut ctx.accounts.attestation_record;

        require!(
            attestation_record.device_id == device_id,
            AttestationError::DeviceIdMismatch
        );

        attestation_record.status = AttestationStatus::Revoked;
        attestation_record.revocation_reason = Some(reason);
        attestation_record.revoked_at = Some(Clock::get()?.unix_timestamp);

        msg!("Device attestation revoked: {:?}", device_id);
        Ok(())
    }

    /// Update attestation (refresh before expiry)
    pub fn refresh_attestation(
        ctx: Context<RefreshAttestation>,
        device_id: [u8; 32],
        new_attestation_quote: AttestationQuote,
    ) -> Result<()> {
        let attestation_record = &mut ctx.accounts.attestation_record;
        let manufacturer = &ctx.accounts.manufacturer_account;

        require!(
            attestation_record.device_id == device_id,
            AttestationError::DeviceIdMismatch
        );

        require!(
            attestation_record.status == AttestationStatus::Valid,
            AttestationError::InvalidAttestation
        );

        // Verify new attestation quote
        require!(
            verify_attestation_quote(&device_id, &new_attestation_quote, &manufacturer.public_key),
            AttestationError::InvalidAttestationQuote
        );

        attestation_record.attestation_quote = new_attestation_quote;
        attestation_record.expires_at = Clock::get()?.unix_timestamp + 86400 * 30; // Extend 30 days

        msg!("Device attestation refreshed: {:?}", device_id);
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
        space = 8 + AttestationAuthority::LEN,
        seeds = [b"attestation_authority"],
        bump
    )]
    pub attestation_authority: Account<'info, AttestationAuthority>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(manufacturer_id: [u8; 32])]
pub struct AddTrustedManufacturer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + ManufacturerAccount::LEN,
        seeds = [b"manufacturer", manufacturer_id.as_ref()],
        bump
    )]
    pub manufacturer_account: Account<'info, ManufacturerAccount>,
    
    #[account(
        mut,
        seeds = [b"attestation_authority"],
        bump = attestation_authority.bump,
        constraint = attestation_authority.authority == authority.key()
    )]
    pub attestation_authority: Account<'info, AttestationAuthority>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(device_id: [u8; 32], manufacturer_id: [u8; 32])]
pub struct CreateAttestation<'info> {
    #[account(mut)]
    pub attester: Signer<'info>,
    
    #[account(
        init,
        payer = attester,
        space = 8 + AttestationRecord::LEN,
        seeds = [b"attestation", device_id.as_ref()],
        bump
    )]
    pub attestation_record: Account<'info, AttestationRecord>,
    
    #[account(
        mut,
        seeds = [b"manufacturer", manufacturer_id.as_ref()],
        bump = manufacturer_account.bump
    )]
    pub manufacturer_account: Account<'info, ManufacturerAccount>,
    
    #[account(
        mut,
        seeds = [b"attestation_authority"],
        bump = attestation_authority.bump
    )]
    pub attestation_authority: Account<'info, AttestationAuthority>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(device_id: [u8; 32])]
pub struct VerifyAttestation<'info> {
    #[account(
        seeds = [b"attestation", device_id.as_ref()],
        bump = attestation_record.bump
    )]
    pub attestation_record: Account<'info, AttestationRecord>,
}

#[derive(Accounts)]
#[instruction(device_id: [u8; 32])]
pub struct RevokeAttestation<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"attestation", device_id.as_ref()],
        bump = attestation_record.bump
    )]
    pub attestation_record: Account<'info, AttestationRecord>,
    
    #[account(
        seeds = [b"attestation_authority"],
        bump = attestation_authority.bump,
        constraint = attestation_authority.authority == authority.key()
    )]
    pub attestation_authority: Account<'info, AttestationAuthority>,
}

#[derive(Accounts)]
#[instruction(device_id: [u8; 32])]
pub struct RefreshAttestation<'info> {
    #[account(mut)]
    pub device_owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"attestation", device_id.as_ref()],
        bump = attestation_record.bump
    )]
    pub attestation_record: Account<'info, AttestationRecord>,
    
    #[account(
        seeds = [b"manufacturer", attestation_record.manufacturer_id.as_ref()],
        bump = manufacturer_account.bump
    )]
    pub manufacturer_account: Account<'info, ManufacturerAccount>,
}

// Account structures
#[account]
pub struct AttestationAuthority {
    pub authority: Pubkey,
    pub trusted_manufacturers: Vec<[u8; 32]>,
    pub total_attestations: u64,
    pub bump: u8,
}

impl AttestationAuthority {
    pub const LEN: usize = 32 + 4 + (32 * 10) + 8 + 1; // Allow for up to 10 trusted manufacturers
}

#[account]
pub struct ManufacturerAccount {
    pub manufacturer_id: [u8; 32],
    pub name: String,
    pub public_key: [u8; 32],
    pub is_active: bool,
    pub devices_attested: u64,
    pub created_at: i64,
    pub bump: u8,
}

impl ManufacturerAccount {
    pub const LEN: usize = 32 + 4 + 50 + 32 + 1 + 8 + 8 + 1; // 50 chars for name
}

#[account]
pub struct AttestationRecord {
    pub device_id: [u8; 32],
    pub manufacturer_id: [u8; 32],
    pub attestation_quote: AttestationQuote,
    pub device_certificate: [u8; 1024],
    pub status: AttestationStatus,
    pub created_at: i64,
    pub expires_at: i64,
    pub revoked_at: Option<i64>,
    pub revocation_reason: Option<RevocationReason>,
    pub bump: u8,
}

impl AttestationRecord {
    pub const LEN: usize = 32 + 32 + AttestationQuote::LEN + 1024 + 1 + 8 + 8 + 9 + 2 + 1;
}

// Data structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AttestationQuote {
    pub version: u32,
    pub signature: [u8; 64],
    pub public_key: [u8; 32],
    pub nonce: [u8; 32],
    pub timestamp: i64,
    pub measurements: Vec<[u8; 32]>, // PCR measurements
}

impl AttestationQuote {
    pub const LEN: usize = 4 + 64 + 32 + 32 + 8 + 4 + (32 * 8); // Allow for up to 8 measurements
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AttestationStatus {
    Valid,
    Expired,
    Revoked,
    Pending,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum RevocationReason {
    Compromised,
    Expired,
    ManufacturerRevoked,
    UserRequested,
    Other,
}

// Helper functions
fn verify_attestation_quote(
    device_id: &[u8; 32],
    quote: &AttestationQuote,
    manufacturer_key: &[u8; 32],
) -> bool {
    // In production, this would verify the TPM/TEE attestation quote
    // For demo purposes, we'll do basic validation
    quote.version > 0 
        && quote.signature != [0u8; 64] 
        && quote.timestamp > 0
        && !quote.measurements.is_empty()
}

fn verify_device_certificate(certificate: &[u8; 1024], manufacturer_key: &[u8; 32]) -> bool {
    // In production, this would verify the X.509 certificate chain
    // For demo purposes, we'll do basic validation
    certificate != &[0u8; 1024] && manufacturer_key != &[0u8; 32]
}

// Error handling
#[error_code]
pub enum AttestationError {
    #[msg("Untrusted manufacturer")]
    UntrustedManufacturer,
    #[msg("Manufacturer ID mismatch")]
    ManufacturerMismatch,
    #[msg("Invalid attestation quote")]
    InvalidAttestationQuote,
    #[msg("Invalid device certificate")]
    InvalidDeviceCertificate,
    #[msg("Device ID mismatch")]
    DeviceIdMismatch,
    #[msg("Invalid attestation")]
    InvalidAttestation,
    #[msg("Attestation expired")]
    AttestationExpired,
} 