use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct Favorites {
    pub number: u64,

    #[max_len(50)]
    pub color: String,

    pub authority: Option<Pubkey>
}

#[account]
#[derive(InitSpace, Debug)]
pub struct FavoritesV1 {
    pub number: u64,

    #[max_len(50)]
    pub color: String,
}