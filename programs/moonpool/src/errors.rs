use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid calculation.")]
    InvalidCalculation,

    #[msg("This amount is not enough.")]
    AmountNotEnough,

    #[msg("Maturity date is invalid.")]
    InvalidMaturityDate,

    #[msg("Pool is not activated.")]
    PoolNotActivated,

    #[msg("Pool is already activated.")]
    PoolAlreadyActivated,

    #[msg("Lock period is not over.")]
    LockPeriodNotOver,

    #[msg("Pool is already matured.")]
    PoolMatured,

    #[msg("Raise period is not ended.")]
    RaisePeriodNotEnded,

    #[msg("This amount is invalid.")]
    InvalidAmount,

    #[msg("This stake is already running.")]
    AlreadyInitialized,

    #[msg("Unauthorized.")]
    Unauthorized,

    #[msg("Invalid account info")]
    InvalidAccount,

    #[msg("Invalid mint.")]
    InvalidMint,

    #[msg("Vault is not initialized.")]
    VaultNotInitialized,

    #[msg("No rewards to claim.")]
    NoRewardsToClaim,

    #[msg("Pool name is too long.")]
    InvalidPoolName,

    #[msg("Pool is not in raise period.")]
    PoolNotInRaisePeriod,

    #[msg("Exceeds maximum supply.")]
    ExceedsMaximumSupply,
}
