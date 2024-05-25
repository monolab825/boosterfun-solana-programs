use anchor_lang::prelude::*;
use solana_program::native_token::LAMPORTS_PER_SOL;

#[constant]
pub const AUTHORITY_SEED: &str = "authority";

#[constant]
pub const VIRTUAL_SOL: u64 = 24 * LAMPORTS_PER_SOL;
