use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(pool_name: String)]
pub struct CreatePool<'info> {
    #[account(
        mut,
        seeds = [MOONPOOL_SEED],
        bump,
    )]
    pub moonpool: Box<Account<'info, Moonpool>>,

    #[account(
        mut,
        seeds = [FEE_VAULT_SEED],
        bump,
    )]
    pub fee_vault: Box<Account<'info, FeeVault>>,

    #[account(
        init,
        seeds = [POOL_SEED, payer.key().as_ref(), pool_name.as_ref()],
        bump,
        payer = payer,
        space = 64 + 24 + 10 + 8 +  std::mem::size_of::<Pool>(),
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init,
        seeds = [POOL_WSOL_VAULT_SEED, pool.key().as_ref()],
        bump,
        payer = payer,
        token::mint = wsol_mint,
        token::authority = pool,
    )]
    pub pool_wsol_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = wsol_mint.key() == NATIVE_SOL_SPL_MINT,
    )]
    pub wsol_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreatePool<'info> {
    pub fn handler(
        &mut self,
        pool_name: String,
        symbol: String,
        raise_goal: u64,
        pool_bump: u8,
    ) -> Result<()> {
        if pool_name.is_empty() || pool_name.len() > 24 {
            return Err(ErrorCode::InvalidPoolName.into());
        }

        if raise_goal == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        // Transfer the pool creation fee in SOL from the payer to the fee vault
        let fee_vault_ix = solana_program::system_instruction::transfer(
            &self.payer.key(),
            &self.fee_vault.key(),
            POOL_CREATION_FEE,
        );
        solana_program::program::invoke(
            &fee_vault_ix,
            &[
                self.payer.to_account_info(),
                self.fee_vault.to_account_info(),
            ],
        )?;

        self.pool.owner = self.payer.key();
        self.pool.uri = "".to_string();
        self.pool.name = pool_name;
        self.pool.symbol = symbol;
        self.pool.droplet_mint = Pubkey::default();
        self.pool.droplet_supply = 0;
        self.pool.droplet_liquidity = 0;
        self.pool.raise_goal = raise_goal;
        self.pool.total_raised = 0;
        self.pool.raise_period_end = Clock::get()?.unix_timestamp + 72 * 60 * 60; // 3 days
        self.pool.maturity_date = Clock::get()?.unix_timestamp + 365 * 24 * 60 * 60; // 1 year
        self.pool.is_initialized = false;
        self.pool.bump = pool_bump;
        self.moonpool.pools += 1;

        Ok(())
    }
}
