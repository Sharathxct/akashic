use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, MintTo, Token, TokenAccount},
};

use crate::{errors::AkashicErrors, state::Vow};

#[derive(Accounts)]
pub struct Long<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        has_one = long_mint,
        has_one = short_mint,
        has_one = vault,
    )]
    pub vow: Account<'info, Vow>,

    #[account(
        init_if_needed,
        payer = user,
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

    #[account(
        mut,
        seeds = [b"short", vow.key().as_ref()],
        bump = vow.short_mint_bump,
    )]
    pub short_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = short_mint,
        associated_token::authority = vow,
    )]
    pub vault_short: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Long<'_> {
    pub fn handle(ctx: Context<Long>, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp < ctx.accounts.vow.deadline,
            AkashicErrors::DeadlinePassed
        );

        require!(amount > 0, AkashicErrors::InvalidAmount);

        let transfer_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        transfer(transfer_ctx, amount)?;

        let token_amount = amount;

        let vow_seeds = &[
            &b"vow"[..],
            &ctx.accounts.vow.seeds.to_le_bytes(),
            &ctx.accounts.vow.authority.as_ref(),
            &[ctx.accounts.vow.vow_bump],
        ];

        let signer_seeds = &[&vow_seeds[..]];

        let mint_long_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.long_mint.to_account_info(),
                to: ctx.accounts.user_long.to_account_info(),
                authority: ctx.accounts.vow.to_account_info(),
            },
            signer_seeds,
        );
        anchor_spl::token::mint_to(mint_long_ctx, token_amount)?;

        let mint_short_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.short_mint.to_account_info(),
                to: ctx.accounts.vault_short.to_account_info(),
                authority: ctx.accounts.vow.to_account_info(),
            },
            signer_seeds,
        );
        anchor_spl::token::mint_to(mint_short_ctx, token_amount)?;

        Ok(())
    }
}
