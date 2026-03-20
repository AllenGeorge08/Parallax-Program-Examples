use crate::state::errors::ErrorCode;
use crate::state::Oracle;
use crate::state::Vault;
use anchor_lang::prelude::*;
use anchor_spl::token::TransferChecked;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer_checked, Mint, MintTo, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_account,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint_account: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        space = 8 + Vault::SPACE,
        payer = user,
        seeds = [b"vault", user.key().as_ref()],
        bump,
    )]
    pub vault: Box<Account<'info, Vault>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_account,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_ata: Account<'info, TokenAccount>,
    ///CHECKED:
    #[account(mut)]
    pub oracle: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64, bumps: &DepositBumps) -> Result<()> {
        let oracle = Oracle::try_deserialize(&mut self.oracle.data.borrow().as_ref())?;

        require!(oracle.is_valid, ErrorCode::OracleInvalid);

        let accounts = TransferChecked {
            from: self.user_ata.to_account_info(),
            to: self.vault_ata.to_account_info(),
            mint: self.mint_account.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), accounts);
        transfer_checked(ctx, amount, self.mint_account.decimals)?;

        let vault = &mut self.vault;
        vault.owner = self.user.key();
        vault.bump = bumps.vault;
        vault.total_deposits += amount;
        vault.total_available += amount;

        msg!(
            "Deposited {} tokens at oracle price {} (exponent {})",
            amount,
            oracle.price_mantissa,
            oracle.price_exponent
        );

        Ok(())
    }
}
