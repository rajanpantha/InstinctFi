use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::{PollAccount, PlatformConfig, CREATOR_POOL_REWARD_BPS, PLATFORM_POOL_FEE_BPS};
use crate::errors::InstinctFiError;
use crate::events::PollSettledEvent;

/// Admin-settle a prediction market poll by declaring the real-world outcome.
///
/// This is the Polymarket-style settlement flow:
/// 1. Users bet on outcomes they believe will happen
/// 2. The real-world event occurs
/// 3. The PLATFORM_ADMIN calls this instruction with the correct `winning_option`
/// 4. Winners (those who bet on the declared outcome) can then call `claim_reward`
///
/// Only the PLATFORM_ADMIN wallet can call this instruction.
/// The poll must be active and its end_time must have passed.
/// The admin provides the `winning_option` index — this is the option that
/// **actually happened in reality**, regardless of vote counts.
///
/// If no votes were placed on the winning option, the creator_reward is still
/// paid to the creator, and the remaining pool stays in treasury until swept.
pub fn handler(
    ctx: Context<AdminSettlePoll>,
    _poll_id: u64,
    winning_option: u8,
) -> Result<()> {
    let clock = Clock::get()?;

    // ── Read immutable data ──
    let poll_key = ctx.accounts.poll_account.key();
    let treasury_bump = ctx.accounts.poll_account.treasury_bump;
    let status = ctx.accounts.poll_account.status;
    let end_time = ctx.accounts.poll_account.end_time;
    let total_pool = ctx.accounts.poll_account.total_pool;
    let options_len = ctx.accounts.poll_account.options.len();
    let vote_counts = ctx.accounts.poll_account.vote_counts.clone();
    let poll_id_val = ctx.accounts.poll_account.poll_id;

    // ── Guards ──
    require!(status == PollAccount::STATUS_ACTIVE, InstinctFiError::AlreadySettled);
    require!(clock.unix_timestamp >= end_time, InstinctFiError::PollNotEnded);
    require!((winning_option as usize) < options_len, InstinctFiError::InvalidOption);

    // ── PDA signer seeds for treasury ──
    let seeds: &[&[u8]] = &[b"treasury", poll_key.as_ref(), &[treasury_bump]];
    let signer_seeds = &[seeds];

    // ── Check if any votes were cast at all ──
    let total_votes: u64 = vote_counts.iter().sum();

    if total_votes == 0 {
        // No votes — just settle. The 0.5 SOL creation fee stays in treasury
        // and is swept to the platform admin via sweep_dust.
        let poll = &mut ctx.accounts.poll_account;
        poll.status = PollAccount::STATUS_SETTLED;
        poll.winning_option = winning_option;

        msg!(
            "AdminSettle: Poll {} settled with no votes. Winner: option {}. Creation fee stays in treasury.",
            poll_id_val, winning_option
        );

        emit!(PollSettledEvent {
            poll_id: poll_id_val,
            winning_option,
            total_pool: 0,
        });
        return Ok(());
    }

    // ── Compute fee splits from voter pool ──
    // 2% of voter pool → creator reward
    let creator_pool_reward = total_pool * CREATOR_POOL_REWARD_BPS / 10_000;
    // 3% of voter pool → stays in treasury (platform fee, swept via sweep_dust)
    let platform_pool_fee = total_pool * PLATFORM_POOL_FEE_BPS / 10_000;
    // 95% of voter pool → distributable to winners via claim_reward
    let distributable = total_pool
        .checked_sub(creator_pool_reward)
        .ok_or(InstinctFiError::Overflow)?
        .checked_sub(platform_pool_fee)
        .ok_or(InstinctFiError::Overflow)?;

    // ── Pay creator reward (2% of voter pool) ──
    if creator_pool_reward > 0 {
        let rent = Rent::get()?;
        let rent_exempt_min = rent.minimum_balance(0);
        let treasury_available = ctx.accounts.treasury.lamports()
            .saturating_sub(rent_exempt_min);
        require!(
            treasury_available >= creator_pool_reward,
            InstinctFiError::TreasuryInsufficient
        );

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.treasury.to_account_info(),
                    to: ctx.accounts.creator.to_account_info(),
                },
                signer_seeds,
            ),
            creator_pool_reward,
        )?;
    }

    // ── Mark settled — set total_pool to distributable (95%) ──
    // claim_reward divides user_votes/total_winning_votes × total_pool,
    // so total_pool must reflect only the winners' share.
    let poll = &mut ctx.accounts.poll_account;
    poll.status = PollAccount::STATUS_SETTLED;
    poll.winning_option = winning_option;
    poll.total_pool = distributable;
    poll.creator_reward = creator_pool_reward;  // record actual amount paid

    let winning_votes = vote_counts[winning_option as usize];

    emit!(PollSettledEvent {
        poll_id: poll_id_val,
        winning_option,
        total_pool: distributable,
    });

    msg!(
        "AdminSettle: Poll {} settled. Winner: option {} ({} votes / {} total). Creator reward: {} lamports. Distributable: {} lamports.",
        poll_id_val,
        winning_option,
        winning_votes,
        total_votes,
        creator_pool_reward,
        distributable
    );
    Ok(())
}

// ─── Accounts ────────────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(poll_id: u64, winning_option: u8)]
pub struct AdminSettlePoll<'info> {
    /// The platform admin — ONLY this wallet can admin-settle polls.
    #[account(
        mut,
        constraint = admin.key() == platform_config.admin @ InstinctFiError::Unauthorized,
    )]
    pub admin: Signer<'info>,

    /// Platform config PDA — source of admin authority
    #[account(
        seeds = [b"platform_config"],
        bump = platform_config.bump,
    )]
    pub platform_config: Account<'info, PlatformConfig>,

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
