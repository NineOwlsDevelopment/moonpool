use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct AddAsset<'info> {
    #[account(
        mut,
        seeds = [MOONPOOL_SEED],
        bump,
    )]
    pub moonpool: Box<Account<'info, Moonpool>>,

    #[account(
        mut,
        constraint = pool.owner == payer.key(),
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init,
        seeds = [ASSET_SEED, pool.key().as_ref(), mint.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<Asset>(),
    )]
    pub asset: Box<Account<'info, Asset>>,

    #[account(
        init,
        seeds = [ASSET_VAULT_SEED, pool.key().as_ref(), mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = asset,
        payer = payer,
    )]
    pub asset_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub payer_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddAsset<'info> {
    pub fn handler(&mut self, amount: u64) -> Result<()> {
        if self.pool.maturity_date < Clock::get()?.unix_timestamp {
            return Err(ErrorCode::PoolMatured.into());
        }

        // transfer from payer to asset_vault
        let deposit_context = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.payer_token_account.to_account_info(),
                to: self.asset_vault.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        );
        transfer(deposit_context, amount)?;

        self.asset.pool = self.pool.key();
        self.asset.mint = self.mint.key();
        self.asset.vault = self.asset_vault.key();
        self.asset.amount = amount;
        Ok(())
    }
}
