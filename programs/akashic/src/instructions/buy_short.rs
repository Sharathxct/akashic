use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{errors::AkashicErrors, state::Vow};

#[derive(Accounts)]
pub struct BuyShort<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        has_one = short_mint,
        has_one = vault,
    )]
    pub vow: Account<'info, Vow>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = short_mint,
        associated_token::authority = user,
    )]
    pub user_short: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", vow.key().as_ref()],
        bump = vow.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"short", vow.key().as_ref()],
        bump = vow.short_mint_bump,
    )]
    pub short_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = short_mint,
        associated_token::authority = vow,
    )]
    pub vault_short: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl BuyShort<'_> {
    pub fn handle(ctx: Context<BuyShort>, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp < ctx.accounts.vow.deadline,
            AkashicErrors::DeadlinePassed
        );

        require!(amount > 0, AkashicErrors::InvalidAmount);

        require!(
            ctx.accounts.vault_short.amount >= amount,
            AkashicErrors::InsufficientShortTokens
        );

        let seeds_bytes = ctx.accounts.vow.seeds.to_le_bytes();
        let vow_seeds = &[
            &b"vow"[..],
            &seeds_bytes.as_ref(),
            &ctx.accounts.vow.authority.as_ref(),
            &[ctx.accounts.vow.vow_bump],
        ];

        let signer_seeds = &[&vow_seeds[..]];

        let transfer_sol_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        transfer(transfer_sol_ctx, amount)?;

        let transfer_short_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::TransferChecked {
                from: ctx.accounts.vault_short.to_account_info(),
                to: ctx.accounts.user_short.to_account_info(),
                mint: ctx.accounts.short_mint.to_account_info(),
                authority: ctx.accounts.vow.to_account_info(),
            },
            signer_seeds,
        );
        anchor_spl::token::transfer_checked(transfer_short_ctx, amount, 9)?;

        Ok(())
    }
}
