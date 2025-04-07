//! process_context_module module for model registry
//!
//! This module provides functionality for implementing model registry functionality.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};

/// ProcessContextModule state account
#[account]
pub struct ProcessContextModule {
    /// The authority that can update this account
    pub authority: Pubkey,
    
    /// Status of the account
    pub status: u8,
    
    /// Additional data
    pub data: [u8; 32],
    
    /// Creation time
    pub created_at: i64,
}

/// Initialize a new ProcessContextModule
pub fn initialize_process_context_module(ctx: Context<InitializeProcessContextModule>) -> Result<()> {
    let account = &mut ctx.accounts.process_context_module;
    account.authority = ctx.accounts.authority.key();
    account.status = 1; // Active
    account.created_at = Clock::get()?.unix_timestamp;
    
    Ok(())
}

/// Update ProcessContextModule data
pub fn update_process_context_module(ctx: Context<UpdateProcessContextModule>, data: [u8; 32]) -> Result<()> {
    let account = &mut ctx.accounts.process_context_module;
    account.data = data;
    
    Ok(())
}

/// Account validation
#[derive(Accounts)]
pub struct InitializeProcessContextModule<'info> {
    /// The authority that can update this account
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The account to initialize
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 32 + 8,
    )]
    pub process_context_module: Account<'info, ProcessContextModule>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

/// Account validation for update
#[derive(Accounts)]
pub struct UpdateProcessContextModule<'info> {
    /// The authority that can update this account
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The account to update
    #[account(
        mut,
        constraint = process_context_module.authority == authority.key()
    )]
    pub process_context_module: Account<'info, ProcessContextModule>,
}
