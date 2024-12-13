use anchor_lang::prelude::*;
use instructions::*;

declare_id!("6ebivbQFHXnU7TqinCBugwnQWNduvQ3q34Xrug8kTkc2");

#[program]
pub mod moonpool {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.handler()
    }

    pub fn initialize_raydium_lp(
        ctx: Context<InitializeRaydiumLp>,
        init_amount_0: u64,
        init_amount_1: u64,
        open_time: u64,
    ) -> Result<()> {
        ctx.accounts
            .handler(init_amount_0, init_amount_1, open_time)
    }

    pub fn create_pool(
        ctx: Context<CreatePool>,
        pool_name: String,
        symbol: String,
        raise_goal: u64,
    ) -> Result<()> {
        ctx.accounts
            .handler(pool_name, symbol, raise_goal, ctx.bumps.pool)
    }

    pub fn create_pool_mint(ctx: Context<CreatePoolMint>, metadata_uri: String) -> Result<()> {
        ctx.accounts.handler(metadata_uri)
    }

    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }

    pub fn add_asset(ctx: Context<AddAsset>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }

    pub fn buy_droplets(ctx: Context<BuyDroplets>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }

    pub fn sell_droplets(ctx: Context<SellDroplets>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }
}

mod errors;
mod instructions;
mod state;
