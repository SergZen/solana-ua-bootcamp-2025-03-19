use anchor_lang::prelude::*;

use crate::{Favorites, SEED};

#[derive(Accounts)]
pub struct UpdateFavorites<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Only for key()
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [SEED, owner.key().as_ref()],
        bump,
    )]
    pub favorites: Account<'info, Favorites>,

    pub system_program: Program<'info, System>
}

pub fn update(context: Context<UpdateFavorites>, number: Option<u64>, color: Option<String>) -> Result<()> {
    let user_public_key = context.accounts.user.key();

    let favorites = &mut context.accounts.favorites;
    let is_owner = context.accounts.owner.key() == user_public_key;
    let is_authority = favorites.authority.map_or(false, |auth| auth == user_public_key);
    
    require!(is_owner || is_authority, crate::errors::ErrorCode::Unauthorized);    

    let current_number = context.accounts.favorites.number;
    let current_color = context.accounts.favorites.color.clone();
    let current_authority = context.accounts.favorites.authority;

    let new_number = number.unwrap_or(current_number);
    let new_color = color.unwrap_or(current_color);
    
    msg!("Greetings from {}", context.program_id);
    msg!(
        "New user {}'s favorite number is {} and favorite color is: {}",
        user_public_key,
        new_number,
        new_color
    );

    context
        .accounts
        .favorites
        .set_inner(
            Favorites { 
                number: new_number, 
                color: new_color,
                authority: current_authority
            }
        );

    Ok(())
}