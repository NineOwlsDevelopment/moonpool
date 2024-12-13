use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{burn, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct SellDroplets<'info> {
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

    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        seeds = [POOL_WSOL_VAULT_SEED, pool.key().as_ref()],
        bump,
    )]
    pub pool_wsol_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [DROPLET_MINT_SEED, pool.key().as_ref(), pool.name.as_bytes()],
        bump,
    )]
    pub droplet_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = pool.owner == pool_owner.key(),
    )]
    pub pool_owner: SystemAccount<'info>,

    #[account(
        mut,
        associated_token::mint = droplet_mint,
        associated_token::authority = payer,
    )]
    pub seller_droplet_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> SellDroplets<'info> {
    pub fn handler(&mut self, amount: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        if current_time > self.pool.maturity_date {
            return Err(ErrorCode::PoolMatured.into());
        }

        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let droplets_value = self.pool.get_sell_price(amount)?;
        let pool_owner_fee = droplets_value * POOL_OWNER_FEE / 100;
        let program_fee = droplets_value * PROGRAM_FEE / 100;

        // Transfer pool owner fee from payer to pool owner
        let pool_owner_ix = solana_program::system_instruction::transfer(
            &self.payer.key(),
            &self.pool.owner,
            pool_owner_fee,
        );
        solana_program::program::invoke(
            &pool_owner_ix,
            &[
                self.payer.to_account_info(),
                self.pool_owner.to_account_info(),
            ],
        )?;

        // Transfer the program fee in SOL from the payer to the fee vault
        let fee_vault_ix = solana_program::system_instruction::transfer(
            &self.payer.key(),
            &self.fee_vault.key(),
            program_fee,
        );
        solana_program::program::invoke(
            &fee_vault_ix,
            &[
                self.payer.to_account_info(),
                self.fee_vault.to_account_info(),
            ],
        )?;

        // Burn the droplets
        let cpi_context = CpiContext::new(
            self.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: self.droplet_mint.to_account_info(),
                from: self.seller_droplet_token_account.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        );
        burn(cpi_context, amount)?;
        self.pool.droplet_supply -= amount;

        // Transfer droplets value from pool vault to payer
        **self
            .pool_wsol_vault
            .to_account_info()
            .try_borrow_mut_lamports()? = self
            .pool_wsol_vault
            .to_account_info()
            .lamports()
            .checked_sub(droplets_value)
            .ok_or(ErrorCode::InvalidCalculation)?;

        **self.payer.to_account_info().try_borrow_mut_lamports()? = self
            .payer
            .to_account_info()
            .lamports()
            .checked_add(droplets_value)
            .ok_or(ErrorCode::InvalidCalculation)?;

        Ok(())
    }
}
