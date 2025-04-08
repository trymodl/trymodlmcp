//! register_model_module module for governance
//!
//! This module provides functionality for implementing governance functionality.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};

/// RegisterModelModule state account
#[account]
pub struct RegisterModelModule {
    /// The authority that can update this account
    pub authority: Pubkey,
    
    /// Status of the account
    pub status: u8,
    
    /// Additional data
    pub data: [u8; 32],
    
    /// Creation time
    pub created_at: i64,
}

/// Initialize a new RegisterModelModule
pub fn initialize_register_model_module(ctx: Context<InitializeRegisterModelModule>) -> Result<()> {
    let account = &mut ctx.accounts.register_model_module;
    account.authority = ctx.accounts.authority.key();
    account.status = 1; // Active
    account.created_at = Clock::get()?.unix_timestamp;
    
    Ok(())
}

/// Update RegisterModelModule data
pub fn update_register_model_module(ctx: Context<UpdateRegisterModelModule>, data: [u8; 32]) -> Result<()> {
    let account = &mut ctx.accounts.register_model_module;
    account.data = data;
    
    Ok(())
}

/// Account validation
#[derive(Accounts)]
pub struct InitializeRegisterModelModule<'info> {
    /// The authority that can update this account
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The account to initialize
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 32 + 8,
    )]
    pub register_model_module: Account<'info, RegisterModelModule>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

/// Account validation for update
#[derive(Accounts)]
pub struct UpdateRegisterModelModule<'info> {
    /// The authority that can update this account
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The account to update
    #[account(
        mut,
        constraint = register_model_module.authority == authority.key()
    )]
    pub register_model_module: Account<'info, RegisterModelModule>,
}
