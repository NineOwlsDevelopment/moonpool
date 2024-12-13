use crate::errors;
use anchor_lang::prelude::*;
use solana_program::{native_token::LAMPORTS_PER_SOL, pubkey};

pub const MOONPOOL_SEED: &[u8] = b"moonpool";
pub const POOL_SEED: &[u8] = b"pool";
pub const POOL_WSOL_VAULT_SEED: &[u8] = b"wsol_vault";
pub const POOL_DROPLET_VAULT_SEED: &[u8] = b"droplet_vault";
pub const ASSET_SEED: &[u8] = b"asset";
pub const ASSET_VAULT_SEED: &[u8] = b"asset_vault";
pub const DROPLET_MINT_SEED: &[u8] = b"mint";
pub const METADATA_SEED: &[u8] = b"metadata";
pub const FEE_VAULT_SEED: &[u8] = b"fee_vault";

pub const NATIVE_SOL_SPL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub const K: f64 = 0.001;
pub const BASE_DROPLET_PRICE: u64 = 1000; // LAMPORTS
pub const LIQUIDITY_FACTOR: f64 = 0.01;

pub const DROPLET_MINT_DECIMALS: u64 = 10_u64.pow(6);
pub const MAX_DROPLET_SUPPLY: u64 = 1_000_000_000_000_000;

pub const POOL_CREATION_FEE: u64 = 50000000;
pub const POOL_OWNER_FEE: u64 = 1; // 1%
pub const PROGRAM_FEE: u64 = 1; // 1%

#[account]
pub struct Moonpool {
    pub admin: Pubkey,
    pub pools: u64,
}

#[account]
pub struct FeeVault {
    pub admin: Pubkey,
}

#[account]
#[derive(Default, InitSpace)]
pub struct Pool {
    pub owner: Pubkey,
    #[max_len(64)]
    pub uri: String,
    #[max_len(24)]
    pub name: String,
    #[max_len(10)]
    pub symbol: String,
    pub droplet_mint: Pubkey,
    pub droplet_supply: u64,
    pub droplet_liquidity: u64,
    pub raise_goal: u64,
    pub total_raised: u64,
    pub raise_period_end: i64,
    pub maturity_date: i64,
    pub is_initialized: bool,
    pub bump: u8,
}

#[account]
pub struct Asset {
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
}

#[account]
pub struct Member {
    pub pool: Pubkey,
    pub user: Pubkey,
}

#[account]
pub struct Transaction {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

impl Pool {
    pub fn validate(&self, amount: u64) -> Result<()> {
        let new_supply = self
            .droplet_supply
            .checked_add(amount)
            .ok_or_else(|| errors::ErrorCode::InvalidAmount)?;

        if new_supply > MAX_DROPLET_SUPPLY {
            return Err(errors::ErrorCode::ExceedsMaximumSupply.into());
        }

        Ok(())
    }

    fn get_liquidity_factor(liquidity: u64) -> f64 {
        return 1.00 + liquidity as f64 * LIQUIDITY_FACTOR;
    }

    pub fn get_buy_price(&mut self, amount: u64) -> Result<u64> {
        if amount == 0 {
            return Err(errors::ErrorCode::InvalidAmount.into());
        }

        let ending_supply = self.droplet_supply + amount;
        let price = K * ((ending_supply.pow(2) - self.droplet_supply.pow(2)) as f64) / 2.0;
        Ok((price * BASE_DROPLET_PRICE as f64) as u64)
    }

    pub fn get_sell_price(&mut self, amount: u64) -> Result<u64> {
        if amount == 0 || amount > self.droplet_supply {
            return Err(errors::ErrorCode::InvalidAmount.into());
        }

        let ending_supply = self.droplet_supply - amount;
        let price = K * (self.droplet_supply.pow(2) - ending_supply.pow(2)) as f64 / 2.0;
        Ok((price * BASE_DROPLET_PRICE as f64) as u64)
    }

    pub fn get_current_price(&mut self) -> Result<u64> {
        Ok((K * self.droplet_supply as f64 * BASE_DROPLET_PRICE as f64) as u64)
    }

    // The price of each token in the funding round is c/r/LAMPORTS_PER_SOL.
    // c is the max amount of droplets per pool - 1,000,000,000
    // r is the amount of SOL to be raised
    pub fn calculate_sol_to_droplets(&mut self, sol_amount: u64) -> Result<u64> {
        let raise_goal = self.raise_goal as f64 / LAMPORTS_PER_SOL as f64;

        let droplets_per_sol = 1_000_000_000 as f64 / raise_goal;
        let float_sol_amount = sol_amount as f64 / LAMPORTS_PER_SOL as f64;
        let droplets_to_mint = float_sol_amount * droplets_per_sol;

        let final_droplets_to_mint = droplets_to_mint * DROPLET_MINT_DECIMALS as f64;

        Ok(final_droplets_to_mint as u64)
    }
}

// run a test to see if the price is correct
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_pricing() {
        let mut pool = Pool {
            owner: Pubkey::default(),
            uri: String::from(""),
            name: String::from("Test Pool"),
            symbol: String::from("TEST"),
            droplet_mint: Pubkey::default(),
            droplet_supply: 0,
            droplet_liquidity: 0,
            raise_goal: 300_000_000_000,
            total_raised: 0,
            raise_period_end: 0,
            maturity_date: 0,
            is_initialized: false,
            bump: 0,
        };

        // calculate the price of 1 SOL in droplets
        let sol_amount = 0.23 * LAMPORTS_PER_SOL as f64;
        let amount_of_droplets = pool.calculate_sol_to_droplets(sol_amount as u64).unwrap();
        println!(
            "{} SOL returns {} droplets",
            sol_amount / LAMPORTS_PER_SOL as f64,
            amount_of_droplets
        );
    }
}
