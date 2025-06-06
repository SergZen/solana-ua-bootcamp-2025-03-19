use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Not Authorized")]
    NotAuthorized,
    #[msg("Insufficient Delegated Amount")]
    InsufficientDelegatedAmount
}