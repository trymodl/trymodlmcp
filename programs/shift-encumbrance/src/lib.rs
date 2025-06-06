use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256, Sha512};

declare_id!("ENCUMB111111111111111111111111111111111111");

#[program]
pub mod shift_encumbrance {
    use super::*;

    /// Initialize the key encumbrance system
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let encumbrance_authority = &mut ctx.accounts.encumbrance_authority;
        encumbrance_authority.authority = ctx.accounts.authority.key();
        encumbrance_authority.total_encumbered_keys = 0;
        encumbrance_authority.total_devices = 0;
        encumbrance_authority.bump = ctx.bumps.encumbrance_authority;
        
        msg!("Shift Key Encumbrance system initialized - one-time keys enabled");
        Ok(())
    }

    /// Initialize key pool for a hardware device
    pub fn initialize_key_pool(
        ctx: Context<InitializeKeyPool>,
        device_id: [u8; 32],
        initial_pool_size: u32,
        public_keys: Vec<[u8; 32]>, // Initial set of public keys
    ) -> Result<()> {
        let key_pool = &mut ctx.accounts.key_pool;
        let encumbrance_authority = &mut ctx.accounts.encumbrance_authority;

        require!(
            public_keys.len() <= initial_pool_size as usize,
            EncumbranceError::InvalidPoolSize
        );

        key_pool.device_id = device_id;
        key_pool.owner = ctx.accounts.owner.key();
        key_pool.total_keys = initial_pool_size;
        key_pool.available_keys = public_keys.len() as u32;
        key_pool.used_keys = 0;
        key_pool.public_keys = public_keys;
        key_pool.encumbered_keys = Vec::new();
        key_pool.created_at = Clock::get()?.unix_timestamp;
        key_pool.bump = ctx.bumps.key_pool;

        encumbrance_authority.total_devices += 1;

        msg!("Key pool initialized for device: {:?} with {} keys", device_id, initial_pool_size);
        Ok(())
    }

    /// Register a key usage and create encumbrance proof
    pub fn encumber_key(
        ctx: Context<EncumberKey>,
        device_id: [u8; 32],
        key_index: u32,
        public_key: [u8; 32],
        destruction_proof: KeyDestructionProof,
        transaction_hash: [u8; 32],
    ) -> Result<()> {
        let key_pool = &mut ctx.accounts.key_pool;
        let encumbrance_record = &mut ctx.accounts.encumbrance_record;
        let encumbrance_authority = &mut ctx.accounts.encumbrance_authority;

        // Verify device ID matches
        require!(
            key_pool.device_id == device_id,
            EncumbranceError::DeviceIdMismatch
        );

        // Verify key exists in pool and hasn't been used
        require!(
            key_index < key_pool.public_keys.len() as u32,
            EncumbranceError::InvalidKeyIndex
        );

        require!(
            key_pool.public_keys[key_index as usize] == public_key,
            EncumbranceError::KeyMismatch
        );

        // Check if key is already encumbered
        require!(
            !key_pool.encumbered_keys.contains(&key_index),
            EncumbranceError::KeyAlreadyEncumbered
        );

        // Verify destruction proof
        require!(
            verify_destruction_proof(&destruction_proof, &public_key, &transaction_hash),
            EncumbranceError::InvalidDestructionProof
        );

        // Create encumbrance record
        encumbrance_record.device_id = device_id;
        encumbrance_record.key_index = key_index;
        encumbrance_record.public_key = public_key;
        encumbrance_record.transaction_hash = transaction_hash;
        encumbrance_record.destruction_proof = destruction_proof;
        encumbrance_record.encumbered_at = Clock::get()?.unix_timestamp;
        encumbrance_record.status = EncumbranceStatus::Encumbered;
        encumbrance_record.bump = ctx.bumps.encumbrance_record;

        // Update key pool
        key_pool.encumbered_keys.push(key_index);
        key_pool.used_keys += 1;
        key_pool.available_keys -= 1;

        encumbrance_authority.total_encumbered_keys += 1;

        msg!("Key encumbered: device {:?}, key index {}", device_id, key_index);
        Ok(())
    }

    /// Verify a key encumbrance (used by other programs)
    pub fn verify_encumbrance(
        ctx: Context<VerifyEncumbrance>,
        device_id: [u8; 32],
        key_index: u32,
        transaction_hash: [u8; 32],
    ) -> Result<bool> {
        let encumbrance_record = &ctx.accounts.encumbrance_record;

        // Verify record matches the query
        require!(
            encumbrance_record.device_id == device_id,
            EncumbranceError::DeviceIdMismatch
        );

        require!(
            encumbrance_record.key_index == key_index,
            EncumbranceError::InvalidKeyIndex
        );

        require!(
            encumbrance_record.transaction_hash == transaction_hash,
            EncumbranceError::TransactionHashMismatch
        );

        require!(
            encumbrance_record.status == EncumbranceStatus::Encumbered,
            EncumbranceError::KeyNotEncumbered
        );

        msg!("Key encumbrance verified: device {:?}, key {}", device_id, key_index);
        Ok(true)
    }

    /// Add new keys to an existing key pool
    pub fn replenish_key_pool(
        ctx: Context<ReplenishKeyPool>,
        device_id: [u8; 32],
        new_public_keys: Vec<[u8; 32]>,
    ) -> Result<()> {
        let key_pool = &mut ctx.accounts.key_pool;

        require!(
            key_pool.device_id == device_id,
            EncumbranceError::DeviceIdMismatch
        );

        require!(
            key_pool.owner == ctx.accounts.owner.key(),
            EncumbranceError::UnauthorizedOwner
        );

        // Add new keys to the pool
        for new_key in new_public_keys.iter() {
            key_pool.public_keys.push(*new_key);
        }

        key_pool.total_keys += new_public_keys.len() as u32;
        key_pool.available_keys += new_public_keys.len() as u32;

        msg!("Key pool replenished: {} new keys added", new_public_keys.len());
        Ok(())
    }

    /// Generate a zero-knowledge proof of key destruction
    pub fn create_destruction_proof(
        ctx: Context<CreateDestructionProof>,
        device_id: [u8; 32],
        private_key_hash: [u8; 32], // Hash of the private key (never store the actual key)
        public_key: [u8; 32],
        nonce: [u8; 32],
    ) -> Result<KeyDestructionProof> {
        // In a real implementation, this would generate a ZK-SNARK proof
        // For demo purposes, we'll create a simplified proof structure
        let proof_data = create_destruction_proof_data(
            &device_id,
            &private_key_hash,
            &public_key,
            &nonce,
        )?;

        let destruction_proof = KeyDestructionProof {
            proof_type: ProofType::ZeroKnowledge,
            proof_data,
            timestamp: Clock::get()?.unix_timestamp,
            nonce,
            hardware_signature: [0u8; 64], // Would be signed by hardware device
        };

        msg!("Destruction proof created for key: {:?}", public_key);
        Ok(destruction_proof)
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
        space = 8 + EncumbranceAuthority::LEN,
        seeds = [b"encumbrance_authority"],
        bump
    )]
    pub encumbrance_authority: Account<'info, EncumbranceAuthority>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(device_id: [u8; 32])]
pub struct InitializeKeyPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + KeyPool::LEN,
        seeds = [b"key_pool", device_id.as_ref()],
        bump
    )]
    pub key_pool: Account<'info, KeyPool>,
    
    #[account(
        mut,
        seeds = [b"encumbrance_authority"],
        bump = encumbrance_authority.bump
    )]
    pub encumbrance_authority: Account<'info, EncumbranceAuthority>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(device_id: [u8; 32], key_index: u32)]
pub struct EncumberKey<'info> {
    #[account(mut)]
    pub device_owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"key_pool", device_id.as_ref()],
        bump = key_pool.bump,
        constraint = key_pool.owner == device_owner.key()
    )]
    pub key_pool: Account<'info, KeyPool>,
    
    #[account(
        init,
        payer = device_owner,
        space = 8 + EncumbranceRecord::LEN,
        seeds = [b"encumbrance", device_id.as_ref(), &key_index.to_le_bytes()],
        bump
    )]
    pub encumbrance_record: Account<'info, EncumbranceRecord>,
    
    #[account(
        mut,
        seeds = [b"encumbrance_authority"],
        bump = encumbrance_authority.bump
    )]
    pub encumbrance_authority: Account<'info, EncumbranceAuthority>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(device_id: [u8; 32], key_index: u32)]
pub struct VerifyEncumbrance<'info> {
    #[account(
        seeds = [b"encumbrance", device_id.as_ref(), &key_index.to_le_bytes()],
        bump = encumbrance_record.bump
    )]
    pub encumbrance_record: Account<'info, EncumbranceRecord>,
}

#[derive(Accounts)]
#[instruction(device_id: [u8; 32])]
pub struct ReplenishKeyPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"key_pool", device_id.as_ref()],
        bump = key_pool.bump,
        constraint = key_pool.owner == owner.key()
    )]
    pub key_pool: Account<'info, KeyPool>,
}

#[derive(Accounts)]
pub struct CreateDestructionProof<'info> {
    pub signer: Signer<'info>,
}

// Account structures
#[account]
pub struct EncumbranceAuthority {
    pub authority: Pubkey,
    pub total_encumbered_keys: u64,
    pub total_devices: u64,
    pub bump: u8,
}

impl EncumbranceAuthority {
    pub const LEN: usize = 32 + 8 + 8 + 1;
}

#[account]
pub struct KeyPool {
    pub device_id: [u8; 32],
    pub owner: Pubkey,
    pub total_keys: u32,
    pub available_keys: u32,
    pub used_keys: u32,
    pub public_keys: Vec<[u8; 32]>,      // Available public keys
    pub encumbered_keys: Vec<u32>,       // Indices of encumbered keys
    pub created_at: i64,
    pub bump: u8,
}

impl KeyPool {
    pub const LEN: usize = 32 + 32 + 4 + 4 + 4 + 4 + (32 * 1000) + 4 + (4 * 1000) + 8 + 1; // Support up to 1000 keys
}

#[account]
pub struct EncumbranceRecord {
    pub device_id: [u8; 32],
    pub key_index: u32,
    pub public_key: [u8; 32],
    pub transaction_hash: [u8; 32],
    pub destruction_proof: KeyDestructionProof,
    pub encumbered_at: i64,
    pub status: EncumbranceStatus,
    pub bump: u8,
}

impl EncumbranceRecord {
    pub const LEN: usize = 32 + 4 + 32 + 32 + KeyDestructionProof::LEN + 8 + 1 + 1;
}

// Data structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct KeyDestructionProof {
    pub proof_type: ProofType,
    pub proof_data: [u8; 256], // ZK proof data
    pub timestamp: i64,
    pub nonce: [u8; 32],
    pub hardware_signature: [u8; 64], // Signed by hardware device
}

impl KeyDestructionProof {
    pub const LEN: usize = 1 + 256 + 8 + 32 + 64;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ProofType {
    ZeroKnowledge,
    HardwareAttestation,
    CryptographicCommitment,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EncumbranceStatus {
    Encumbered,
    Verified,
    Disputed,
}

// Helper functions
fn verify_destruction_proof(
    proof: &KeyDestructionProof,
    public_key: &[u8; 32],
    transaction_hash: &[u8; 32],
) -> bool {
    // In production, this would verify the ZK-SNARK proof
    // For demo purposes, we'll do basic validation
    proof.timestamp > 0 
        && proof.nonce != [0u8; 32]
        && proof.proof_data != [0u8; 256]
        && proof.hardware_signature != [0u8; 64]
}

fn create_destruction_proof_data(
    device_id: &[u8; 32],
    private_key_hash: &[u8; 32],
    public_key: &[u8; 32],
    nonce: &[u8; 32],
) -> Result<[u8; 256]> {
    // In production, this would generate a ZK-SNARK proof
    // For demo purposes, we'll create a hash-based proof
    let mut hasher = Sha256::new();
    hasher.update(device_id);
    hasher.update(private_key_hash);
    hasher.update(public_key);
    hasher.update(nonce);
    hasher.update(b"SHIFT_KEY_DESTRUCTION_PROOF");
    
    let hash = hasher.finalize();
    let mut proof_data = [0u8; 256];
    
    // Fill the proof data with multiple hashes for demonstration
    for i in 0..8 {
        let mut hasher = Sha256::new();
        hasher.update(&hash);
        hasher.update(&(i as u64).to_le_bytes());
        let iteration_hash = hasher.finalize();
        proof_data[i * 32..(i + 1) * 32].copy_from_slice(&iteration_hash);
    }
    
    Ok(proof_data)
}

// Error handling
#[error_code]
pub enum EncumbranceError {
    #[msg("Invalid pool size")]
    InvalidPoolSize,
    #[msg("Device ID mismatch")]
    DeviceIdMismatch,
    #[msg("Invalid key index")]
    InvalidKeyIndex,
    #[msg("Key mismatch")]
    KeyMismatch,
    #[msg("Key already encumbered")]
    KeyAlreadyEncumbered,
    #[msg("Invalid destruction proof")]
    InvalidDestructionProof,
    #[msg("Transaction hash mismatch")]
    TransactionHashMismatch,
    #[msg("Key not encumbered")]
    KeyNotEncumbered,
    #[msg("Unauthorized owner")]
    UnauthorizedOwner,
} 