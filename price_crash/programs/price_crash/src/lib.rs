pub mod instructions;
pub mod state;

pub use instructions::*;
pub use state::*;
use anchor_lang::prelude::*;

declare_id!("5wpEfF27ApCwAiyxedbD1fUVWqh15JJ6mwYLwca5cV23");

#[program]
pub mod price_crash {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>,amount: u64) -> Result<()>{
        ctx.accounts.deposit(amount, &ctx.bumps)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>,requested_units: u64) -> Result<()>{
        ctx.accounts.withdraw(requested_units)?;
        Ok(())
    }
}
