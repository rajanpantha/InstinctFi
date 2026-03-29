use anchor_lang::prelude::*;
use crate::state::PlatformConfig;
use crate::errors::InstinctFiError;
use crate::events::{PlatformPausedEvent, AdminTransferredEvent};

/// Update platform configuration. Only the current admin can call.
///
/// Supports toggling the platform pause state and transferring admin
/// authority. To only change one field, pass the current value for the
/// field you want to keep unchanged.
pub fn handler(
    ctx: Context<UpdatePlatformConfig>,
    paused: bool,
    new_admin: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.platform_config;
    let old_paused = config.paused;
    let old_admin = config.admin;

    config.paused = paused;
    config.admin = new_admin;

    if paused != old_paused {
        emit!(PlatformPausedEvent {
            paused,
            admin: old_admin,
        });
        msg!("Platform paused: {}", paused);
    }

    if new_admin != old_admin {
        emit!(AdminTransferredEvent {
            old_admin,
            new_admin,
        });
        msg!("Admin transferred: {} -> {}", old_admin, new_admin);
    }

    Ok(())
}

#[derive(Accounts)]
pub struct UpdatePlatformConfig<'info> {
    #[account(
        mut,
        constraint = admin.key() == platform_config.admin @ InstinctFiError::Unauthorized,
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"platform_config"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,
}
