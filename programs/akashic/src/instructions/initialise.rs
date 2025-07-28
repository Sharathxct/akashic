use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token},
};

use crate::state::{Vow, VowResult};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialise<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        seeds = [b"long", vow.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = vow,
    )]
    pub long_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        seeds = [b"short", vow.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = vow,
    )]
    pub short_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault", vow.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        init,
        payer = authority,
        space = Vow::INIT_SPACE + 8,
        seeds = [b"vow", seed.to_le_bytes().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub vow: Account<'info, Vow>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Initialise<'_> {
    pub fn handle(ctx: Context<Initialise>, seed: u64, deadline: i64) -> Result<()> {
        ctx.accounts.vow.set_inner(Vow {
            authority: ctx.accounts.authority.key(),
            seeds: seed,
            deadline,
            long_mint: ctx.accounts.long_mint.key(),
            short_mint: ctx.accounts.short_mint.key(),
            vault: ctx.accounts.vault.key(),
            result: VowResult::Pending,
            resolved: false,
            vow_bump: ctx.bumps.vow,
            vault_bump: ctx.bumps.vault,
            long_mint_bump: ctx.bumps.long_mint,
            short_mint_bump: ctx.bumps.short_mint,
        });

        // Transfer SOL to the vault for rent exemption
        let cpi_program = ctx.accounts.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.authority.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, Rent::get()?.minimum_balance(0))?;

        Ok(())
    }
}
