use anchor_lang::prelude::*;

use crate::{Favorites, ANCHOR_DISCRIMINATOR_SIZE, SEED};

#[derive(Accounts)]
pub struct SetFavorites<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = ANCHOR_DISCRIMINATOR_SIZE + Favorites::INIT_SPACE,
        seeds = [SEED, user.key().as_ref()],
        bump,
    )]
    pub favorites: Account<'info, Favorites>,

    pub system_program: Program<'info, System>
}

pub fn set(context: Context<SetFavorites>, number: u64, color: String) -> Result<()> {
    let user_public_key = context.accounts.user.key();
    msg!("Greetings from {}", context.program_id);
    msg!(
        "User {}'s favorite number is {} and favorite color is: {}",
        user_public_key,
        number,
        color
    );

    context
        .accounts
        .favorites
        .set_inner(Favorites { number, color});

    Ok(())
}