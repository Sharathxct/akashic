use anchor_lang::prelude::*;

#[error_code]
pub enum AkashicErrors {
    #[msg("Vow is already initialised")]
    VowAlreadyInitialised,

    #[msg("Deadline has passed")]
    DeadlinePassed,

    #[msg("Vow is not yet resolved")]
    VowNotResolved,

    #[msg("Vow is already resolved")]
    VowAlreadyResolved,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Insufficient short tokens in vault")]
    InsufficientShortTokens,

    #[msg("Cannot claim - wrong outcome")]
    InvalidClaim,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
