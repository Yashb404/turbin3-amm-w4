use anchor_lang::prelude::*;
use crate::error::ErrorCode;
use crate::state::Config;

#[derive(Accounts)]
pub struct Lock<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
        constraint = config.authority == Some(signer.key()) @ ErrorCode::Unauthorized,
    )]
    pub config: Account<'info, Config>,
}


impl <'info> Lock<'info> {
    pub fn lock(&mut self, lock: bool) -> Result<()> {
        self.config.locked = lock;
        Ok(())
    }
}
