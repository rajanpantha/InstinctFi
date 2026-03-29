use anchor_lang::prelude::*;

/// Emitted when a new prediction poll is created.
#[event]
pub struct PollCreatedEvent {
    pub poll_id: u64,
    pub creator: Pubkey,
    pub end_time: i64,
    pub total_pool: u64,
    pub num_options: u8,
}

/// Emitted when a user buys option-coins on a poll.
#[event]
pub struct VoteCastEvent {
    pub poll_id: u64,
    pub voter: Pubkey,
    pub option_index: u8,
    pub num_coins: u64,
    pub cost: u64,
}

/// Emitted when a poll is settled by admin with a declared winner.
#[event]
pub struct PollSettledEvent {
    pub poll_id: u64,
    pub winning_option: u8,
    pub total_pool: u64,
}

/// Emitted when a winner claims their reward payout.
#[event]
pub struct RewardClaimedEvent {
    pub poll_id: u64,
    pub claimer: Pubkey,
    pub reward: u64,
}

/// Emitted when a poll is voided after the admin grace period expired.
#[event]
pub struct PollVoidedEvent {
    pub poll_id: u64,
    pub creator_refund: u64,
}

/// Emitted when the platform pause state changes.
#[event]
pub struct PlatformPausedEvent {
    pub paused: bool,
    pub admin: Pubkey,
}

/// Emitted when platform admin authority is transferred.
#[event]
pub struct AdminTransferredEvent {
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
}
