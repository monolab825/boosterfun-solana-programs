use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use fixed::types::I128F0;

use crate::{
    constants::{AUTHORITY_SEED, VIRTUAL_SOL},
    errors::*,
    state::{Amm, Pool},
};

pub fn swap_exact_tokens_for_tokens(
    ctx: Context<SwapExactTokensForTokens>,
    swap_a: bool,
    input_amount: u64,
    min_output_amount: u64,
) -> Result<()> {
    let amm = &mut ctx.accounts.amm;

    // Prevent lock of the bonding curves
    assert!(amm.lock == false);

    // Prevent depositing assets the depositor does not own
    let input = if swap_a && input_amount > ctx.accounts.trader_account_a.amount {
        ctx.accounts.trader_account_a.amount
    } else if !swap_a && input_amount > ctx.accounts.trader.lamports() {
        ctx.accounts.trader.lamports()
    } else {
        input_amount
    };

    let pool_a = &ctx.accounts.pool_account_a;
    let pool_b = &ctx.accounts.pool_authority;
    let output = if swap_a {
        I128F0::from_num(input)
            .checked_mul(I128F0::from_num(pool_b.lamports() + VIRTUAL_SOL))
            .unwrap()
            .checked_div(
                I128F0::from_num(pool_a.amount)
                    .checked_add(I128F0::from_num(input))
                    .unwrap(),
            )
            .unwrap()
    } else {
        I128F0::from_num(input)
            .checked_mul(I128F0::from_num(pool_a.amount))
            .unwrap()
            .checked_div(
                I128F0::from_num(pool_b.lamports() + VIRTUAL_SOL)
                    .checked_add(I128F0::from_num(input))
                    .unwrap(),
            )
            .unwrap()
    }
    .to_num::<u64>();

    if output < min_output_amount {
        return err!(TutorialError::OutputTooSmall);
    }

    if !swap_a && output > amm.max_per_wallet {
        return err!(TutorialError::InvalidTooMany);
    }

    // Transfer tokens to the pool
    let authority_bump = ctx.bumps.pool_authority;
    let authority_seeds = &[
        &ctx.accounts.pool.amm.to_bytes(),
        &ctx.accounts.mint_a.key().to_bytes(),
        AUTHORITY_SEED.as_bytes(),
        &[authority_bump],
    ];
    let signer_seeds = &[&authority_seeds[..]];
    if swap_a {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.trader_account_a.to_account_info(),
                    to: ctx.accounts.pool_account_a.to_account_info(),
                    authority: ctx.accounts.trader.to_account_info(),
                },
            ),
            input,
        )?;

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.pool_authority.to_account_info(),
                    to: ctx.accounts.trader.to_account_info(),
                },
                signer_seeds,
            ),
            output - output * amm.fee as u64 / 10000,
        )?;

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.pool_authority.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
                signer_seeds,
            ),
            output * amm.fee as u64 / 10000,
        )?;
    } else {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.trader.to_account_info(),
                    to: ctx.accounts.pool_authority.to_account_info(),
                },
            ),
            input,
        )?;

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.trader.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
            ),
            input * amm.fee as u64 / 10000,
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_account_a.to_account_info(),
                    to: ctx.accounts.trader_account_a.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                signer_seeds,
            ),
            output,
        )?;
    }

    msg!("Traded {} tokens for {}", input, output);

    if pool_b.lamports() > 85 * VIRTUAL_SOL {
        amm.lock = true;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_account_a.to_account_info(),
                    to: ctx.accounts.treasury_account_a.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                signer_seeds,
            ),
            pool_a.amount,
        )?;

        let rent = &ctx.accounts.rent;
        let rent_exempt_minimum = rent.minimum_balance(48);

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.pool_authority.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
                signer_seeds,
            ),
            pool_b.lamports() - rent_exempt_minimum,
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct SwapExactTokensForTokens<'info> {
    #[account(
        seeds = [
            amm.id.as_ref()
        ],
        bump,
    )]
    pub amm: Account<'info, Amm>,

    #[account(
        seeds = [
            pool.amm.as_ref(),
            pool.mint_a.key().as_ref(),
        ],
        bump,
        has_one = amm,
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

    /// The account doing the swap
    pub trader: Signer<'info>,

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
        associated_token::authority = trader,
    )]
    pub trader_account_a: Box<Account<'info, TokenAccount>>,

    //The treasury account
    ///CHECK: safe , it's treasury account
    #[account(mut)]
    pub treasury: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = treasury,
    )]
    pub treasury_account_a: Box<Account<'info, TokenAccount>>,

    /// The account paying for all rents
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana ecosystem accounts
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,
}
