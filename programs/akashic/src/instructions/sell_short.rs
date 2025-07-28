use crate::{errors::AkashicErrors, state::Vow};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct SellShort<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        has_one = short_mint,
        has_one = vault,
    )]
    pub vow: Account<'info, Vow>,

    #[account(
        mut,
        associated_token::mint = short_mint,
        associated_token::authority = user,
        constraint = user_short.amount >= amount @ AkashicErrors::InsufficientShortTokens,
    )]
    pub user_short: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", vow.key().as_ref()],
        bump = vow.vault_bump,
        constraint = vault_short.amount >= amount @ AkashicErrors::InsufficientShortTokens,
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
        constraint = vault_short.amount >= amount @ AkashicErrors::InsufficientShortTokens,
    )]
    pub vault_short: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl SellShort<'_> {
    pub fn handle(ctx: Context<SellShort>, amount: u64) -> Result<()> {
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

        // transfer short tokens to vault_short
        let seeds_bytes = ctx.accounts.vow.seeds.to_le_bytes();
        let vow_seeds = &[
            &b"vow"[..],
            &seeds_bytes.as_ref(),
            &ctx.accounts.vow.authority.as_ref(),
            &[ctx.accounts.vow.vow_bump],
        ];

        let signer_seeds = &[&vow_seeds[..]];

        let transfer_short_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::TransferChecked {
                from: ctx.accounts.user_short.to_account_info(),
                to: ctx.accounts.vault_short.to_account_info(),
                mint: ctx.accounts.short_mint.to_account_info(),
                authority: ctx.accounts.vow.to_account_info(),
            },
        );
        anchor_spl::token::transfer_checked(transfer_short_ctx, amount, 9)?;

        // transfer sol to user
        let transfer_sol_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer_seeds,
        );
        anchor_lang::system_program::transfer(transfer_sol_ctx, amount)?;
        Ok(())
    }
}
