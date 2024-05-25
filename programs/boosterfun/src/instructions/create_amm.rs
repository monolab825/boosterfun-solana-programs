use anchor_lang::prelude::*;

use crate::{errors::*, state::Amm};

pub fn create_amm(
    ctx: Context<CreateAmm>,
    id: Pubkey,
    fee: u16,
    max_per_wallet: u64,
) -> Result<()> {
    let amm = &mut ctx.accounts.amm;
    amm.id = id;
    amm.admin = ctx.accounts.admin.key();
    amm.fee = fee;
    amm.max_per_wallet = max_per_wallet;
    amm.lock = false;

    Ok(())
}

#[derive(Accounts)]
#[instruction(id: Pubkey, fee: u16)]
pub struct CreateAmm<'info> {
    #[account(
        init,
        payer = payer,
        space = Amm::LEN,
        seeds = [
            id.as_ref()
        ],
        bump,
        constraint = fee < 10000 @ TutorialError::InvalidFee,
    )]
    pub amm: Account<'info, Amm>,

    /// The admin of the AMM
    /// CHECK: Read only, delegatable creation
    pub admin: AccountInfo<'info>,

    /// The account paying for all rents
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana ecosystem accounts
    pub system_program: Program<'info, System>,
}
