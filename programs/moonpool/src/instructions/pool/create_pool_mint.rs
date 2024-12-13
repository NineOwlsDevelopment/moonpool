use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::instructions::CreateMetadataAccountV3Builder;
use mpl_token_metadata::types::DataV2;

#[derive(Accounts)]
pub struct CreatePoolMint<'info> {
    #[account(
        mut,
        seeds = [MOONPOOL_SEED],
        bump,
    )]
    pub moonpool: Box<Account<'info, Moonpool>>,

    #[account(
        mut,
        seeds = [POOL_SEED, payer.key().as_ref(), pool.name.as_ref()],
        bump,
        constraint = pool.owner == payer.key(),
        constraint = pool.is_initialized == false,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init,
        seeds = [DROPLET_MINT_SEED, pool.key().as_ref()],
        bump,
        payer = payer,
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub droplet_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [POOL_WSOL_VAULT_SEED, pool.key().as_ref()],
        bump,
        token::mint = wsol_mint,
        token::authority = pool,
    )]
    pub pool_wsol_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        seeds = [POOL_DROPLET_VAULT_SEED, pool.key().as_ref()],
        bump,
        payer = payer,
        token::mint = droplet_mint,
        token::authority = pool,
    )]
    pub pool_droplet_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: Metadata account that will be created by the token metadata program
    #[account(
        mut,
        seeds = [
            METADATA_SEED,
            token_metadata_program.key().as_ref(),
            droplet_mint.key().as_ref()
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        constraint = wsol_mint.key() == NATIVE_SOL_SPL_MINT,
    )]
    pub wsol_mint: Box<Account<'info, Mint>>,

    /// CHECK: Metaplex program
    pub token_metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreatePoolMint<'info> {
    pub fn handler(&mut self, metadata_uri: String) -> Result<()> {
        if self.token_metadata_program.key() != mpl_token_metadata::ID {
            return Err(ErrorCode::InvalidAccount.into());
        }

        // Create metadata for the droplet mint
        let payer_key = self.payer.key();
        let pool_seeds = &[
            POOL_SEED,
            payer_key.as_ref(),
            self.pool.name.as_ref(),
            &[self.pool.bump],
        ];
        let pool_signer = &[&pool_seeds[..]];

        let mut binding = CreateMetadataAccountV3Builder::new();
        let metadata_ix = binding
            .metadata(self.metadata.key())
            .mint(self.droplet_mint.key())
            .mint_authority(self.pool.key())
            .payer(self.payer.key())
            .update_authority(self.pool.key(), true)
            .data(DataV2 {
                name: self.pool.name.clone(),
                symbol: self.pool.symbol.clone(),
                uri: format!(
                    "https://lavender-far-hyena-367.mypinata.cloud/ipfs/{}",
                    metadata_uri.clone(),
                ),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            })
            .is_mutable(false);

        anchor_lang::solana_program::program::invoke_signed(
            &metadata_ix.instruction(),
            &[
                self.metadata.to_account_info(),
                self.droplet_mint.to_account_info(),
                self.pool.to_account_info(),
                self.payer.to_account_info(),
                self.system_program.to_account_info(),
                self.rent.to_account_info(),
            ],
            pool_signer,
        )?;

        // // Each pool starts with 1 Billion droplets
        // // Mint droplets to the pool

        // let amount = 1_000_000_000u64
        //     .checked_mul(DROPLET_MINT_DECIMALS)
        //     .ok_or_else(|| ErrorCode::InvalidAmount)?;

        // let cpi_context = CpiContext::new_with_signer(
        //     self.token_program.to_account_info(),
        //     anchor_spl::token::MintTo {
        //         mint: self.droplet_mint.to_account_info(),
        //         to: self.pool_droplet_vault.to_account_info(),
        //         authority: self.pool.to_account_info(),
        //     },
        //     pool_signer,
        // );
        // mint_to(cpi_context, amount)?;

        // self.pool.droplet_supply = self
        //     .pool
        //     .droplet_supply
        //     .checked_add(amount)
        //     .ok_or_else(|| ErrorCode::InvalidAmount)?;

        // self.pool.droplet_liquidity = self
        //     .pool
        //     .droplet_liquidity
        //     .checked_add(amount)
        //     .ok_or_else(|| ErrorCode::InvalidAmount)?;

        self.pool.uri = metadata_uri;
        self.pool.droplet_mint = self.droplet_mint.key();
        self.pool.is_initialized = true;
        Ok(())
    }
}
