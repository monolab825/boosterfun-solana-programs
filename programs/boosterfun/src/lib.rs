#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

mod constants;
mod errors;
mod instructions;
mod state;

// Set the correct key here
declare_id!("Dk35sdrj9qniZMSmxBc1BgYD42Fe2RviAyq2ZbeEavp1");

#[program]
pub mod boosterfun {
    pub use super::instructions::*;
    use super::*;

    pub fn create_amm(
        ctx: Context<CreateAmm>,
        id: Pubkey,
        fee: u16,
        max_per_wallet: u64,
    ) -> Result<()> {
        instructions::create_amm(ctx, id, fee, max_per_wallet)
    }

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        instructions::create_pool(ctx)
    }

    pub fn create_token_mint(
        ctx: Context<CreateTokenMint>,
        token_decimals: u8,
        token_name: String,
        token_symbol: String,
        token_uri: String,
        total_supply: u64,
    ) -> Result<()> {
        instructions::create_token_mint(
            ctx,
            token_decimals,
            token_name,
            token_symbol,
            token_uri,
            total_supply,
        )
    }

    pub fn deposit_liquidity(ctx: Context<DepositLiquidity>, amount_a: u64) -> Result<()> {
        instructions::deposit_liquidity(ctx, amount_a)
    }

    pub fn swap_exact_tokens_for_tokens(
        ctx: Context<SwapExactTokensForTokens>,
        swap_a: bool,
        input_amount: u64,
        min_output_amount: u64,
    ) -> Result<()> {
        instructions::swap_exact_tokens_for_tokens(ctx, swap_a, input_amount, min_output_amount)
    }

    /// initilaize market in raydium
    pub fn proxy_initialize(
        ctx: Context<ProxyInitialize>,
        nonce: u8,
        open_time: u64,
        init_pc_amount: u64,
        init_coin_amount: u64,
    ) -> Result<()> {
        instructions::initialize(ctx, nonce, open_time, init_pc_amount, init_coin_amount)
    }
}
