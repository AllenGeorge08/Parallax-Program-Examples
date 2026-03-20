use crate::state::errors::ErrorCode;
use crate::state::Oracle;
use crate::state::Vault;
use anchor_lang::prelude::*;
use anchor_spl::token::TransferChecked;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint_account,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub mint_account: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [b"vault", user.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        associated_token::mint = mint_account,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    /// CHECK: deserialized and validated manually
    #[account(mut)]
    pub oracle: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, requested_units: u64) -> Result<()> {
        require_eq!(
            self.vault.owner,
            self.user.key(),
            ErrorCode::UnauthorizedVaultAccess
        );

        // ── Validate oracle ───────────────────────────────────────────────────
        let oracle = Oracle::try_deserialize(&mut self.oracle.data.borrow().as_ref())?;

        require!(oracle.is_valid, ErrorCode::OracleInvalid);

        let price_mantissa = oracle.price_mantissa;
        let price_exponent = oracle.price_exponent;

        require!(price_mantissa > 0, ErrorCode::InvalidOraclePrice);
        require!(requested_units > 0, ErrorCode::ZeroWithdrawAmount);

        // ── Validate vault ────────────────────────────────────────────────────
        let vault = &mut self.vault;
        require!(
            vault.total_available >= requested_units,
            ErrorCode::InsufficientVaultBalance
        );

        // ── Compute actual current value ──────────────────────────────────────
        // The user deposited `requested_units` of an asset.
        // The actual token amount to return is based on current oracle price.
        //
        // actual_tokens = requested_units * price_mantissa * 10^price_exponent
        //
        // Since price_exponent is typically negative (e.g. -3),
        // we divide by 10^abs(exponent) to normalize.
        let actual_tokens = if price_exponent >= 0 {
            (requested_units as u128)
                .checked_mul(price_mantissa as u128)
                .ok_or(ErrorCode::MathOverflow)?
                .checked_mul(10u128.pow(price_exponent as u32))
                .ok_or(ErrorCode::MathOverflow)? as u64
        } else {
            let divisor = 10u128.pow(price_exponent.unsigned_abs());
            (requested_units as u128)
                .checked_mul(price_mantissa as u128)
                .ok_or(ErrorCode::MathOverflow)?
                .checked_div(divisor)
                .ok_or(ErrorCode::MathUnderflow)? as u64
        };

        require!(actual_tokens > 0, ErrorCode::ZeroWithdrawValue);

        // ── Transfer actual current value from vault → user ───────────────────
        let seeds = &[b"vault", self.user.key.as_ref(), &[vault.bump]];
        let signer_seeds = &[&seeds[..]];

        let accounts = TransferChecked {
            from: self.vault_ata.to_account_info(),
            to: self.user_ata.to_account_info(),
            mint: self.mint_account.to_account_info(),
            authority: vault.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        transfer_checked(ctx, actual_tokens, self.mint_account.decimals)?;

        // ── Update vault state ────────────────────────────────────────────────
        vault.total_available = vault
            .total_available
            .checked_sub(requested_units)
            .ok_or(ErrorCode::MathUnderflow)?;

        vault.total_deposits = vault
            .total_deposits
            .checked_sub(requested_units)
            .ok_or(ErrorCode::MathUnderflow)?;

        msg!(
            "Withdrew {} units → {} tokens at price {}e{}",
            requested_units,
            actual_tokens,
            price_mantissa,
            price_exponent
        );

        Ok(())
    }
}
