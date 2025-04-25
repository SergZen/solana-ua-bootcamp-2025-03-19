use anchor_lang::prelude::*;

use crate::{Favorites, FavoritesV1, ANCHOR_DISCRIMINATOR_SIZE};

#[derive(Accounts)]
pub struct UpdateFavoritesMultiVersions<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Only for key()
    pub owner: AccountInfo<'info>,

    /// CHECK: Check favorites versions
    #[account(mut)]
    pub favorites: AccountInfo<'info>,

    pub system_program: Program<'info, System>
}

pub fn update_multi(context: Context<UpdateFavoritesMultiVersions>, number: Option<u64>, color: Option<String>) -> Result<()> {
    let user_public_key = context.accounts.user.key();

    let account_data = &mut context.accounts.favorites.try_borrow_mut_data()?[..];
    let discriminator = &account_data[..ANCHOR_DISCRIMINATOR_SIZE];

    if discriminator == Favorites::DISCRIMINATOR {
        let mut favorites: Favorites = Favorites::try_deserialize_unchecked(&mut &account_data[..])?;

        let is_owner = context.accounts.owner.key() == user_public_key;
        let is_authority = favorites.authority.map_or(false, |auth| auth == user_public_key);
        require!(is_owner || is_authority, crate::errors::ErrorCode::Unauthorized);

        let new_number = number.unwrap_or(favorites.number);
        let new_color = color.unwrap_or(favorites.color.clone());

        msg!(
            "New user {}'s favorite number is {} and favorite color is: {}",
            user_public_key,
            new_number,
            new_color
        );

        favorites.color = new_color;
        favorites.number = new_number;

        let dst = &mut account_data[..];
        favorites.try_serialize(&mut &mut *dst)?;

    } else if discriminator == FavoritesV1::DISCRIMINATOR {
        let mut favorites: FavoritesV1 = FavoritesV1::try_deserialize_unchecked(&mut &account_data[..])?;

        let is_owner = context.accounts.owner.key() == user_public_key;
        require!(is_owner, crate::errors::ErrorCode::Unauthorized);

        let new_number = number.unwrap_or(favorites.number);
        let new_color = color.unwrap_or(favorites.color.clone());

        msg!(
            "New user {}'s favorite number is {} and favorite color is: {}",
            user_public_key,
            new_number,
            new_color
        );

        favorites.color = new_color;
        favorites.number = new_number;       

        let dst = &mut account_data[..];
        favorites.try_serialize(&mut &mut *dst)?;
    } else {
        return Err(crate::errors::ErrorCode::UnknownFavoritesVersion.into());
    }

    Ok(())
}