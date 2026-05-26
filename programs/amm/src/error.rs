use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The pool is currently locked")]
    PoolLocked,
    #[msg("Invalid amount, must be greater than 0")]
    InvalidAmount,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("No authority set for this pool")]
    NoAuthority,
}
