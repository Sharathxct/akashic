use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum VowResult {
    Pending,
    Success,
    Failure,
}

#[account]
#[derive(InitSpace)]
pub struct Vow {
    pub authority: Pubkey,
    pub seeds: u64,
    pub deadline: i64,

    pub long_mint: Pubkey,
    pub short_mint: Pubkey,
    pub vault: Pubkey,

    // Result tracking
    #[max_len(1)]
    pub result: VowResult,
    pub resolved: bool,

    // Bumps for PDAs
    pub vow_bump: u8,
    pub vault_bump: u8,
    pub long_mint_bump: u8,
    pub short_mint_bump: u8,
}
