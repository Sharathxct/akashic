#![allow(unexpected_cfgs, deprecated)]
use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("BUMwdXP7tr6NTEP6GmbBxozW82GSweNzGUYJyGuqSbrN");

#[program]
pub mod akashic {
    use super::*;

    pub fn initialize(ctx: Context<Initialise>, seed: u64, deadline: i64) -> Result<()> {
        instructions::Initialise::handle(ctx, seed, deadline)
    }

    pub fn long(ctx: Context<Long>, amount: u64) -> Result<()> {
        instructions::Long::handle(ctx, amount)
    }

    pub fn buy_short(ctx: Context<BuyShort>, amount: u64) -> Result<()> {
        instructions::BuyShort::handle(ctx, amount)
    }

    pub fn sell_short(ctx: Context<SellShort>, amount: u64) -> Result<()> {
        instructions::SellShort::handle(ctx, amount)
    }

    pub fn submit_result(ctx: Context<SubmitResult>, result: VowResult) -> Result<()> {
        instructions::SubmitResult::handle(ctx, result)
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        instructions::Claim::handle(ctx)
    }

    pub fn admin_init(ctx: Context<AdminInit>, fee: u64) -> Result<()> {
        instructions::AdminInit::handle(ctx, fee)
    }
}
