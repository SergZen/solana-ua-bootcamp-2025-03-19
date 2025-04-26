use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken, token_2022::{revoke, Revoke}, token_interface::{
        Mint, TokenAccount, TokenInterface
    }
};

use crate::Offer;

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub token_mint_a: InterfaceAccount<'info, Mint>,
    
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        has_one = token_mint_b,
    )]
    pub offer: Account<'info, Offer>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn return_tokens_to_maker(ctx: &Context<CancelOffer>) -> Result<()> {
    let revoke_accounts = Revoke {
        source: ctx.accounts.maker_token_account_a.to_account_info(),
        authority: ctx.accounts.maker.to_account_info(),
    };

    let cpi_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        revoke_accounts,
    );

    revoke(
        cpi_context,
    )
}