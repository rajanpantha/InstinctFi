use anchor_lang::prelude::*;

/// Initial admin pubkey — used only in `initialize_platform` to prevent
/// front-running of platform setup. After init, admin is stored in
/// PlatformConfig PDA and can be rotated via `update_platform_config`.
/// Wallet: 62PFLSvnG4Zp8jYS9AFymETvV5e8xBA2JBW2UhjqyNmS
pub const INITIAL_ADMIN: Pubkey = Pubkey::new_from_array([
    74,165,43,158,91,189,190,113,41,134,75,0,110,157,151,8,
    192,58,184,92,42,146,252,113,22,211,49,209,118,16,62,133,
]);

/// Grace period (7 days) during which only the platform admin can settle.
/// After this window, anyone can void the poll via `settle_poll`.
pub const ADMIN_SETTLE_GRACE_SECONDS: i64 = 7 * 24 * 60 * 60;

/// Maximum option-coins a single user can buy per `cast_vote` call.
pub const MAX_COINS_PER_VOTE: u64 = 1_000;

/// Flat creation fee paid by poll creator (0.5 SOL).
/// Goes into treasury PDA. Non-refundable once the poll has votes.
pub const POLL_CREATION_FEE: u64 = 500_000_000;

/// Minimum unit price per option-coin (0.001 SOL).
pub const MIN_UNIT_PRICE: u64 = 1_000_000;

/// Minimum poll duration in seconds (1 hour).
pub const MIN_POLL_DURATION: i64 = 3_600;

/// Creator pool reward: 2% of total voter pool paid to creator on settlement.
pub const CREATOR_POOL_REWARD_BPS: u64 = 200;

/// Platform settlement fee: 3% of voter pool kept in treasury on settlement.
pub const PLATFORM_POOL_FEE_BPS: u64 = 300;

// ─── User Account ───────────────────────────────────────────────────────────
// PDA seeds: ["user", authority.key]
// Tracks user stats. No "demo balance" — all value is real SOL.
#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    /// Wallet public key (owner)
    pub authority: Pubkey,
    /// Total polls created
    pub total_polls_created: u64,
    /// Total vote-coins purchased across all polls
    pub total_votes_cast: u64,
    /// Number of polls won
    pub polls_won: u64,
    /// Total lamports staked across all polls
    pub total_staked: u64,
    /// Total lamports won across all polls
    pub total_winnings: u64,
    /// Account creation timestamp
    pub created_at: i64,
    /// PDA bump
    pub bump: u8,
}

// ─── Poll Account ───────────────────────────────────────────────────────────
// PDA seeds: ["poll", creator.key, poll_id.to_le_bytes()]
// All monetary values in LAMPORTS.
#[account]
#[derive(InitSpace)]
pub struct PollAccount {
    /// Unique poll id (scoped per creator)
    pub poll_id: u64,
    /// Creator's public key
    pub creator: Pubkey,
    /// Poll title (max 64 chars)
    #[max_len(64)]
    pub title: String,
    /// Poll description (max 256 chars)
    #[max_len(256)]
    pub description: String,
    /// Category tag (max 32 chars)
    #[max_len(32)]
    pub category: String,
    /// Off-chain image URL (max 256 chars)
    #[max_len(256)]
    pub image_url: String,
    /// Option labels (2–6 options, max 32 chars each)
    #[max_len(6, 32)]
    pub options: Vec<String>,
    /// Vote tally per option (same index)
    #[max_len(6)]
    pub vote_counts: Vec<u64>,
    /// Price per option-coin in lamports
    pub unit_price: u64,
    /// Unix timestamp when poll ends
    pub end_time: i64,
    /// Distributable pool in lamports (excludes fees)
    pub total_pool: u64,
    /// Creator's initial investment in lamports
    pub creator_investment: u64,
    /// Platform creation fee in lamports (0.5 SOL flat fee, stays in treasury)
    pub platform_fee: u64,
    /// Creator pool reward in lamports (2% of voter pool, paid at settlement)
    pub creator_reward: u64,
    /// 0 = Active, 1 = Settled
    pub status: u8,
    /// Winning option index (255 = unset)
    pub winning_option: u8,
    /// Treasury PDA bump
    pub treasury_bump: u8,
    /// Poll account PDA bump
    pub bump: u8,
    /// Total unique voters
    pub total_voters: u32,
    /// Created-at timestamp
    pub created_at: i64,
}

impl PollAccount {
    pub const STATUS_ACTIVE: u8 = 0;
    pub const STATUS_SETTLED: u8 = 1;
    pub const STATUS_VOIDED: u8 = 2;

    pub fn is_active(&self) -> bool {
        self.status == Self::STATUS_ACTIVE
    }

    pub fn is_voided(&self) -> bool {
        self.status == Self::STATUS_VOIDED
    }

    pub fn is_ended(&self, clock: &Clock) -> bool {
        clock.unix_timestamp >= self.end_time
    }
}

// ─── Vote Account ───────────────────────────────────────────────────────────
// PDA seeds: ["vote", poll_account.key, voter.key]
// Tracks a single user's votes across all options in one poll.
#[account]
#[derive(InitSpace)]
pub struct VoteAccount {
    /// The poll this vote belongs to
    pub poll: Pubkey,
    /// The voter's public key
    pub voter: Pubkey,
    /// Option-coins bought per option
    #[max_len(6)]
    pub votes_per_option: Vec<u64>,
    /// Total lamports staked in this poll
    pub total_staked: u64,
    /// Whether rewards have been claimed
    pub claimed: bool,
    /// PDA bump
    pub bump: u8,
}

// ─── Platform Config ────────────────────────────────────────────────────────
// PDA seeds: ["platform_config"]
// Stores platform-wide configuration: admin authority and pause state.
// Replaces the hardcoded PLATFORM_ADMIN constant for admin checks.
#[account]
#[derive(InitSpace)]
pub struct PlatformConfig {
    /// Current platform admin public key
    pub admin: Pubkey,
    /// When true, blocks new polls and votes
    pub paused: bool,
    /// PDA bump
    pub bump: u8,
}
