use anchor_lang::prelude::*;

use crate::Favorites;

#[derive(Accounts)]
pub struct UpdateFavorites<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub favorites: Account<'info, Favorites>,

    pub system_program: Program<'info, System>
}

pub fn update(context: Context<UpdateFavorites>, number: Option<u64>, color: Option<String>) -> Result<()> {
    let user_public_key = context.accounts.user.key();

    let current = &mut context.accounts.favorites;
    let new_number = number.unwrap_or(current.number);
    let new_color = color.unwrap_or(current.color.clone());
    
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
                color: new_color
            }
        );

    Ok(())
}