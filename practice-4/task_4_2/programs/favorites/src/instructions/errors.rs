use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("Invalid account")]
    InvalidAccount,
    #[msg("Unknown favorites version")]
    UnknownFavoritesVersion,
}