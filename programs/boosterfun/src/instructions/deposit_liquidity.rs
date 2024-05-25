use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

use crate::{constants::AUTHORITY_SEED, state::Pool};

pub fn deposit_liquidity(ctx: Context<DepositLiquidity>, amount_a: u64) -> Result<()> {
    // Prevent depositing assets the depositor does not own
    let amount_a = if amount_a > ctx.accounts.depositor_account_a.amount {
        ctx.accounts.depositor_account_a.amount
    } else {
        amount_a
    };

    // Making sure they are provided in the same proportion as existing liquidity
    let pool_a = &ctx.accounts.pool_account_a;
    let pool_b = &ctx.accounts.pool_authority;
    // Defining pool creation like this allows attackers to frontrun pool creation with bad ratios
    assert!(pool_a.amount == 0 && pool_b.lamports() == 0);

    // Transfer tokens to the pool
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.depositor_account_a.to_account_info(),
                to: ctx.accounts.pool_account_a.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount_a,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(
        seeds = [
            pool.amm.as_ref(),
            pool.mint_a.key().as_ref(),
        ],
        bump,
        has_one = mint_a,
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: Read only authority
    #[account(
        mut,
        seeds = [
            pool.amm.as_ref(),
            mint_a.key().as_ref(),
            AUTHORITY_SEED.as_ref(),
        ],
        bump,
    )]
    pub pool_authority: AccountInfo<'info>,

    /// The account paying for all rents
    pub depositor: Signer<'info>,

    pub mint_a: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = pool_authority,
    )]
    pub pool_account_a: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = depositor,
    )]
    pub depositor_account_a: Box<Account<'info, TokenAccount>>,

    /// The account paying for all rents
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana ecosystem accounts
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
