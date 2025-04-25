use anchor_lang::prelude::*;

use crate::{Favorites, SEED};

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED, user.key().as_ref()],
        bump,
    )]
    pub favorites: Account<'info, Favorites>,

    pub system_program: Program<'info, System>
}

pub fn change_authority(context: Context<SetAuthority>, new_authority: Option<Pubkey>) -> Result<()> {
    let favorites = &mut context.accounts.favorites;
    favorites.authority = new_authority;
    
    if let Some(auth) = new_authority {
        msg!("New authority set to: {} for account owned by {}", auth, context.accounts.user.key());
    } else {
        msg!("Authority removed for account owned by {}", context.accounts.user.key());
    }
    
    Ok(())
}