use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Burn, Mint, Token, TokenAccount},
};

use crate::{
    errors::AkashicErrors,
    state::{Vow, VowResult},
};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        has_one = long_mint,
        has_one = vault,
    )]
    pub vow: Account<'info, Vow>,

    #[account(
        mut,
        associated_token::mint = long_mint,
        associated_token::authority = user,
    )]
    pub user_long: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", vow.key().as_ref()],
        bump = vow.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"long", vow.key().as_ref()],
        bump = vow.long_mint_bump,
    )]
    pub long_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Claim<'_> {
    pub fn handle(ctx: Context<Claim>) -> Result<()> {
        let amount = ctx.accounts.user_long.amount;
        let vow = &ctx.accounts.vow;

        // Check if vow is resolved
        require!(vow.resolved, AkashicErrors::VowNotResolved);

        // Check if result is success (only long tokens can be redeemed)
        require!(
            vow.result == VowResult::Success,
            AkashicErrors::InvalidClaim
        );

        // Check amount is valid
        require!(amount > 0, AkashicErrors::InvalidAmount);

        // Check user has enough long tokens
        require!(
            ctx.accounts.user_long.amount >= amount,
            AkashicErrors::InvalidAmount
        );

        // Calculate SOL payout (2x: 1B tokens = 2 SOL)
        let sol_amount = amount
            .checked_mul(2)
            .ok_or(AkashicErrors::ArithmeticOverflow)?;

        // Check vault has enough SOL
        require!(
            ctx.accounts.vault.lamports() >= sol_amount,
            AkashicErrors::InsufficientShortTokens
        );

        let vow_seeds = &[
            &b"vow"[..],
            &ctx.accounts.vow.seeds.to_le_bytes(),
            &ctx.accounts.vow.authority.as_ref(),
            &[ctx.accounts.vow.vow_bump],
        ];
        let signer_seeds = &[&vow_seeds[..]];

        // Burn the long tokens
        let burn_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.long_mint.to_account_info(),
                from: ctx.accounts.user_long.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        anchor_spl::token::burn(burn_ctx, amount)?;

        // Transfer SOL from vault to user (2x payout)
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer_seeds,
        );
        transfer(transfer_ctx, sol_amount)?;

        Ok(())
    }
}
