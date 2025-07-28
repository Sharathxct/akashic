use anchor_lang::prelude::*;

use crate::{
    errors::AkashicErrors,
    state::{Vow, VowResult},
};

#[derive(Accounts)]
pub struct SubmitResult<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        // For now, allow anyone to submit result - you can add authority check later
    )]
    pub vow: Account<'info, Vow>,
}

impl SubmitResult<'_> {
    pub fn handle(ctx: Context<SubmitResult>, result: VowResult) -> Result<()> {
        let vow = &mut ctx.accounts.vow;

        // Check if vow is already resolved
        require!(!vow.resolved, AkashicErrors::VowAlreadyResolved);

        // Check if deadline has passed (results can only be submitted after deadline)
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp >= vow.deadline,
            AkashicErrors::DeadlinePassed
        );

        // Set the result
        vow.result = result;
        vow.resolved = true;

        Ok(())
    }
}
