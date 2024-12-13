use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [MOONPOOL_SEED],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<Moonpool>(),
    )]
    pub moonpool: Box<Account<'info, Moonpool>>,

    #[account(
        init,
        seeds = [FEE_VAULT_SEED],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<FeeVault>(),
    )]
    pub fee_vault: Box<Account<'info, FeeVault>>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn handler(&mut self) -> Result<()> {
        self.moonpool.admin = self.payer.key();
        self.moonpool.pools = 0;
        self.fee_vault.admin = self.payer.key();
        Ok(())
    }
}
