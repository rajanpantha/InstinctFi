use anchor_lang::prelude::*;
use crate::state::{PlatformConfig, INITIAL_ADMIN};
use crate::errors::InstinctFiError;

/// One-time initialization of the PlatformConfig PDA.
///
/// Must be called once after program deployment. Only the INITIAL_ADMIN
/// wallet can call this, preventing front-running of platform setup.
/// After initialization, admin authority can be transferred via
/// `update_platform_config`.
pub fn handler(ctx: Context<InitializePlatform>) -> Result<()> {
    require!(
        ctx.accounts.admin.key() == INITIAL_ADMIN,
        InstinctFiError::Unauthorized
    );

    let config = &mut ctx.accounts.platform_config;
    config.admin = ctx.accounts.admin.key();
    config.paused = false;
    config.bump = ctx.bumps.platform_config;

    msg!("Platform initialized. Admin: {}", config.admin);
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePlatform<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + PlatformConfig::INIT_SPACE,
        seeds = [b"platform_config"],
        bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    pub system_program: Program<'info, System>,
}
