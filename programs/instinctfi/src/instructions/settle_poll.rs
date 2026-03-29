use anchor_lang::prelude::*;
use crate::state::{PollAccount, ADMIN_SETTLE_GRACE_SECONDS};
use crate::errors::InstinctFiError;
use crate::events::PollVoidedEvent;

/// Void an expired poll after the admin grace period.
///
/// After the 7-day admin grace period, if the admin hasn't settled the poll,
/// anyone can call this to void it. This replaces the previous vote-count-based
/// settlement which was incorrect for prediction markets (popularity ≠ correctness).
///
/// Flow:
/// 1. Sets poll status to VOIDED (status = 2)
/// 2. Refunds the creator's investment from the treasury
/// 3. Voters individually claim refunds via `refund_tied_poll`
///
/// If no votes: refunds entire treasury to creator.
/// If votes: refunds creator_investment to creator; voters refund individually.
pub fn handler(ctx: Context<SettlePoll>, _poll_id: u64) -> Result<()> {
    let clock = Clock::get()?;

    // ── Read immutable data first ──
    let status = ctx.accounts.poll_account.status;
    let end_time = ctx.accounts.poll_account.end_time;
    let vote_counts = ctx.accounts.poll_account.vote_counts.clone();
    let poll_id_val = ctx.accounts.poll_account.poll_id;

    // ── Guards ──
    require!(status == PollAccount::STATUS_ACTIVE, InstinctFiError::AlreadySettled);
    require!(clock.unix_timestamp >= end_time, InstinctFiError::PollNotEnded);

    // ── Admin grace period must be expired ──
    let grace_deadline = end_time.checked_add(ADMIN_SETTLE_GRACE_SECONDS).unwrap_or(i64::MAX);
    require!(
        clock.unix_timestamp >= grace_deadline,
        InstinctFiError::AdminGracePeriodActive
    );

    let total_votes: u64 = vote_counts.iter().sum();

    // ── Mark voided ──
    // The 0.5 SOL creation fee stays in treasury — it is a non-refundable platform fee.
    // Voters claim their individual SOL back via `refund_tied_poll`.
    let poll = &mut ctx.accounts.poll_account;
    poll.status = PollAccount::STATUS_VOIDED;
    poll.winning_option = 255;

    emit!(PollVoidedEvent {
        poll_id: poll_id_val,
        creator_refund: 0,
    });

    msg!(
        "Poll {} voided after grace period ({} votes). Voters may claim refunds via refund_tied_poll.",
        poll_id_val,
        total_votes
    );
    Ok(())
}

// ─── Accounts ────────────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct SettlePoll<'info> {
    /// Anyone can trigger settlement (permissionless crank)
    #[account(mut)]
    pub settler: Signer<'info>,

    /// CHECK: Poll creator — receives creator reward. Validated by constraint.
    #[account(
        mut,
        constraint = creator.key() == poll_account.creator @ InstinctFiError::UnauthorizedNotCreator,
    )]
    pub creator: UncheckedAccount<'info>,

    /// The poll to settle
    #[account(
        mut,
        seeds = [b"poll", poll_account.creator.as_ref(), &poll_id.to_le_bytes()],
        bump = poll_account.bump,
    )]
    pub poll_account: Account<'info, PollAccount>,

    /// CHECK: Treasury PDA — SOL source for creator reward
    #[account(
        mut,
        seeds = [b"treasury", poll_account.key().as_ref()],
        bump = poll_account.treasury_bump,
    )]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
