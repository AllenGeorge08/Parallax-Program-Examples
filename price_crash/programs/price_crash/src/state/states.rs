use anchor_lang::prelude::*;

#[account]
#[derive(Debug)]
pub struct Vault {
    pub owner: Pubkey,
    pub total_deposits: u64,
    pub total_available: u64,
    pub seed: u64,
    pub bump: u8,
}

impl Vault {
    pub const SPACE: usize = 8 + 32 + 8 + 8 + 8 + 1; // discriminator + fields
}

#[account]
#[derive(Debug)]
pub struct User {
    pub owner: Pubkey,
    pub total_deposited: u64,
    pub bump: u8,
}

// in your anchor program's lib.rs
#[account]
#[derive(InitSpace)]
pub struct Oracle {
    pub authority: Pubkey,
    pub price_mantissa: i64,
    pub price_exponent: i32,
    pub confidence: u64,
    pub last_update_slot: u64,
    pub last_update_epoch: u64,
    pub is_valid: bool,
    pub bump: u8,
}
