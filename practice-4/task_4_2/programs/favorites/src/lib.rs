use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod constants;

pub use instructions::*;
pub use state::*;
pub use constants::*;

declare_id!("HngVG4VtZ83DshWrtQ556sez82HFZ4YmRcLCrDXN1pSv");

#[program]
pub mod favorites {
    use super::*;

    pub fn set_favorites(context: Context<SetFavorites>, number: u64, color: String) -> Result<()> {
        instructions::set_favorites::set(context, number, color)
    }

    pub fn update_favorites(context: Context<UpdateFavorites>, number: Option<u64>, color: Option<String>) -> Result<()> {
        instructions::update_favorites::update(context, number, color)
    }

    pub fn set_authority(context: Context<SetAuthority>, new_authority: Option<Pubkey>) -> Result<()> {
        instructions::set_authority::change_authority(context, new_authority)
    }

    pub fn set_favorites_v1(context: Context<SetLegacyFavorites>, number: u64, color: String) -> Result<()> {
        instructions::set_favorites_v1::set_v1(context, number, color)
    }

    pub fn update_favorites_multi_versions(context: Context<UpdateFavoritesMultiVersions>, number: Option<u64>, color: Option<String>) -> Result<()> {
        instructions::update_favorites_multi_versions::update_multi(context, number, color)
    }
}