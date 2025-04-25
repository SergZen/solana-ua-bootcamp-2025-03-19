use anchor_lang::prelude::*;

use crate::{FavoritesV1, ANCHOR_DISCRIMINATOR_SIZE, SEED_V1};

#[derive(Accounts)]
pub struct SetLegacyFavorites<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = ANCHOR_DISCRIMINATOR_SIZE + FavoritesV1::INIT_SPACE,
        seeds = [SEED_V1, user.key().as_ref()],
        bump,
    )]
    pub favorites: Account<'info, FavoritesV1>,

    pub system_program: Program<'info, System>
}

pub fn set_v1(context: Context<SetLegacyFavorites>, number: u64, color: String) -> Result<()> {
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
        .set_inner(FavoritesV1 { 
            number, 
            color
        });

    Ok(())
}