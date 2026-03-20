use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // ── Oracle errors ─────────────────────────────────────────────────────────
    #[msg("Oracle account could not be deserialized")]
    InvalidOracle,

    #[msg("Oracle is marked as invalid")]
    OracleInvalid,

    #[msg("Oracle price is negative or zero")]
    InvalidOraclePrice,

    // ── Deposit errors ────────────────────────────────────────────────────────
    #[msg("Deposit amount must be greater than zero")]
    ZeroDepositAmount,

    #[msg("Insufficient token balance for deposit")]
    InsufficientBalance,

    // ── Withdraw errors ───────────────────────────────────────────────────────
    #[msg("Withdraw amount exceeds available vault balance")]
    InsufficientVaultBalance,

    #[msg("Withdraw amount must be greater than zero")]
    ZeroWithdrawAmount,

    #[msg("Actual withdrawal value is zero at current oracle price")]
    ZeroWithdrawValue,

    // ── Vault errors ──────────────────────────────────────────────────────────
    #[msg("Vault owner mismatch")]
    UnauthorizedVaultAccess,

    // ── Math errors ───────────────────────────────────────────────────────────
    #[msg("Math overflow occurred")]
    MathOverflow,

    #[msg("Math underflow occurred")]
    MathUnderflow,
}
