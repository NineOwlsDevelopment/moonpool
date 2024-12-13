use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{mint_to, sync_native, transfer, Mint, SyncNative, Token, TokenAccount};

#[derive(Accounts)]
pub struct Contribute<'info> {
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
        mut,
        seeds = [POOL_SEED, pool_owner.key().as_ref(), pool.name.as_ref()],
        bump,
        constraint = pool.droplet_mint != Pubkey::default(),
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        constraint = pool.owner == pool_owner.key(),
    )]
    pub pool_owner: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [POOL_WSOL_VAULT_SEED, pool.key().as_ref()],
        bump,
        token::mint = wsol_mint,
        token::authority = pool,
    )]
    pub pool_wsol_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = wsol_mint,
        associated_token::authority = payer,
    )]
    pub payer_wsol_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = droplet_mint,
        associated_token::authority = payer,
    )]
    pub payer_droplet_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = droplet_mint.key() == pool.droplet_mint,
    )]
    pub droplet_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub wsol_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Contribute<'info> {
    pub fn handler(&mut self, amount: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        if current_time > self.pool.raise_period_end {
            return Err(ErrorCode::PoolNotInRaisePeriod.into());
        }

        if amount == 0 {
            return Err(ErrorCode::InvalidAmount.into());
        }

        let amount_to_mint = self.pool.calculate_sol_to_droplets(amount)?;
        self.pool.validate(amount_to_mint)?;

        // Transfer the program fee in SOL from the payer to the fee vault
        let program_fee = amount * PROGRAM_FEE.checked_div(100).unwrap();
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

        // Convert the user's contribution amount from SOL to WSOL
        let wrap_ix = solana_program::system_instruction::transfer(
            &self.payer.key(),
            &self.payer_wsol_token_account.key(),
            amount,
        );
        solana_program::program::invoke(
            &wrap_ix,
            &[
                self.payer.to_account_info(),
                self.payer_wsol_token_account.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;
        let _sync_native_ix = sync_native(CpiContext::new(
            self.token_program.to_account_info(),
            SyncNative {
                account: self.payer_wsol_token_account.to_account_info(),
            },
        ))?;

        // Transfer the converted amount of WSOL to the pool vault
        let transfer_ix = CpiContext::new(
            self.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: self.payer_wsol_token_account.to_account_info(),
                to: self.pool_wsol_vault.to_account_info(),
                authority: self.payer.to_account_info(),
            },
        );
        transfer(transfer_ix, amount)?;

        // Mint proportionate droplets to payer's token account
        let pool_owner_key = self.pool.owner.key();
        let pool_seeds = &[
            POOL_SEED,
            pool_owner_key.as_ref(),
            self.pool.name.as_bytes(),
            &[self.pool.bump],
        ];

        let pool_signer = &[&pool_seeds[..]];
        let cpi_context = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: self.droplet_mint.to_account_info(),
                to: self.payer_droplet_token_account.to_account_info(),
                authority: self.pool.to_account_info(),
            },
            pool_signer,
        );
        mint_to(cpi_context, amount_to_mint)?;

        self.pool.droplet_supply = self
            .pool
            .droplet_supply
            .checked_add(amount_to_mint)
            .ok_or_else(|| ErrorCode::InvalidAmount)?;

        self.pool.total_raised = self
            .pool
            .total_raised
            .checked_add(amount)
            .ok_or_else(|| ErrorCode::InvalidAmount)?;

        Ok(())
    }
}
