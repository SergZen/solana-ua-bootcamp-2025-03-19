use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod constants;

pub use instructions::*;
pub use state::*;
pub use constants::*;

declare_id!("EsL1PwU3Y7dDq8zaeiaUenZUq2NKqwgnq5ioNmqKfYLf");

#[program]
pub mod favorites {
    use super::*;

    pub fn set_favorites(context: Context<SetFavorites>, number: u64, color: String) -> Result<()> {
        instructions::set_favorites::set(context, number, color)
    }

    pub fn update_favorites(context: Context<UpdateFavorites>, number: Option<u64>, color: Option<String>) -> Result<()> {
        instructions::update_favorites::update(context, number, color)
    }
}