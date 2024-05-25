use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Amm {
    /// The primary key of the AMM
    pub id: Pubkey,

    /// Account that has admin authority over the AMM
    pub admin: Pubkey,

    /// The LP fee taken on each trade, in basis points
    pub fee: u16,

    /// Max tokens per wallet
    pub max_per_wallet: u64,

    /// Bool value for lock of bonding curves
    pub lock: bool,
}

impl Amm {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 2;
}

#[account]
#[derive(Default)]
pub struct Pool {
    /// Primary key of the AMM
    pub amm: Pubkey,

    /// Mint of token A
    pub mint_a: Pubkey,
}

impl Pool {
    pub const LEN: usize = 8 + 32 + 32;
}
