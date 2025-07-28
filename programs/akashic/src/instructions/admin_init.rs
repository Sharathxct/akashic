use crate::state::Config;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AdminInit<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(init, payer = admin, space = 8 + 32 + 8, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

impl AdminInit<'_> {
    pub fn handle(ctx: Context<AdminInit>, fee: u64) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.fee = fee;
        Ok(())
    }
}
