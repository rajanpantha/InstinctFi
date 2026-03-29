/**
 * Unit tests for data conversion utilities.
 * Tests Supabase row → frontend type conversions.
 */
import {
    rowToDemoPoll,
    rowToDemoVote,
    rowToUserAccount,
    createPlaceholderUser,
    demoPollToRow,
    withFreshPeriods,
} from "../dataConverters";

describe("rowToDemoPoll", () => {
    const baseRow = {
        id: "poll-1",
        poll_id: "42",
        creator: "wallet-abc",
        title: "Will BTC hit $100k?",
        description: "Bitcoin prediction",
        category: "Crypto",
        image_url: "https://example.com/img.png",
        option_images: ["img1.png", "img2.png"],
        options: ["Yes", "No"],
        vote_counts: [10, 5],
        unit_price_cents: 10000000,
        end_time: 1700000000,
        total_pool_cents: 150000000,
        creator_investment_cents: 100000000,
        platform_fee_cents: 1000000,
        creator_reward_cents: 1000000,
        status: 0,
        winning_option: 255,
        total_voters: 15,
        created_at: "1699000000",
    };

    it("converts all fields correctly", () => {
        const poll = rowToDemoPoll(baseRow);
        expect(poll.id).toBe("poll-1");
        expect(poll.pollId).toBe(42);
        expect(poll.creator).toBe("wallet-abc");
        expect(poll.title).toBe("Will BTC hit $100k?");
        expect(poll.description).toBe("Bitcoin prediction");
        expect(poll.category).toBe("Crypto");
        expect(poll.imageUrl).toBe("https://example.com/img.png");
        expect(poll.optionImages).toEqual(["img1.png", "img2.png"]);
        expect(poll.options).toEqual(["Yes", "No"]);
        expect(poll.voteCounts).toEqual([10, 5]);
        expect(poll.unitPriceLamports).toBe(10000000);
        expect(poll.endTime).toBe(1700000000);
        expect(poll.totalPoolLamports).toBe(150000000);
        expect(poll.status).toBe(0);
        expect(poll.winningOption).toBe(255);
        expect(poll.totalVoters).toBe(15);
    });

    it("handles missing optional fields with defaults", () => {
        const minimalRow = {
            id: "poll-2",
            poll_id: "1",
            creator: "wallet-xyz",
            title: "Test",
            options: ["A", "B"],
            // All optional fields missing
        };
        const poll = rowToDemoPoll(minimalRow);
        expect(poll.description).toBe("");
        expect(poll.category).toBe("");
        expect(poll.imageUrl).toBe("");
        expect(poll.optionImages).toEqual([]);
        expect(poll.voteCounts).toEqual([]);
        expect(poll.unitPriceLamports).toBe(0);
        expect(poll.totalPoolLamports).toBe(0);
    });

    it("converts vote_counts strings to numbers", () => {
        const row = { ...baseRow, vote_counts: ["7", "3"] } as any;
        const poll = rowToDemoPoll(row);
        expect(poll.voteCounts).toEqual([7, 3]);
    });
});

describe("rowToDemoVote", () => {
    it("converts a vote row correctly", () => {
        const row = {
            poll_id: "poll-1",
            voter: "wallet-abc",
            votes_per_option: [5, 0, 3],
            total_staked_cents: 80000000,
            claimed: false,
        };
        const vote = rowToDemoVote(row);
        expect(vote.pollId).toBe("poll-1");
        expect(vote.voter).toBe("wallet-abc");
        expect(vote.votesPerOption).toEqual([5, 0, 3]);
        expect(vote.totalStakedLamports).toBe(80000000);
        expect(vote.claimed).toBe(false);
    });

    it("handles missing votes_per_option", () => {
        const row = {
            poll_id: "poll-2",
            voter: "wallet-xyz",
            votes_per_option: [],
            total_staked_cents: 0,
            claimed: false,
        };
        const vote = rowToDemoVote(row);
        expect(vote.votesPerOption).toEqual([]);
    });
});

describe("rowToUserAccount", () => {
    it("converts user row with all fields", () => {
        const row = {
            wallet: "wallet-abc",
            balance: 5000000000,
            signup_bonus_claimed: true,
            total_votes_cast: 25,
            total_polls_voted: 10,
            polls_won: 4,
            polls_created: 3,
            total_spent_cents: 250000000,
            total_winnings_cents: 400000000,
            created_at: "1699000000",
        };
        const user = rowToUserAccount(row);
        expect(user.wallet).toBe("wallet-abc");
        // Balance is always 0 from Supabase — on-chain is the sole source of truth
        expect(user.balance).toBe(0);
        expect(user.signupBonusClaimed).toBe(true);
        expect(user.totalVotesCast).toBe(25);
        expect(user.pollsWon).toBe(4);
        expect(user.pollsCreated).toBe(3);
        expect(user.totalSpentLamports).toBe(250000000);
        expect(user.totalWinningsLamports).toBe(400000000);
    });

    it("defaults missing numeric fields to 0", () => {
        const row = { wallet: "wallet-xyz" };
        const user = rowToUserAccount(row);
        expect(user.balance).toBe(0);
        expect(user.totalVotesCast).toBe(0);
        expect(user.totalSpentLamports).toBe(0);
        expect(user.totalWinningsLamports).toBe(0);
    });
});

describe("createPlaceholderUser", () => {
    it("creates a user with all zero counters", () => {
        const user = createPlaceholderUser("wallet-new");
        expect(user.wallet).toBe("wallet-new");
        expect(user.balance).toBe(0);
        expect(user.signupBonusClaimed).toBe(false);
        expect(user.totalVotesCast).toBe(0);
        expect(user.pollsWon).toBe(0);
        expect(user.pollsCreated).toBe(0);
        expect(user.createdAt).toBeGreaterThan(0);
    });
});

describe("demoPollToRow (roundtrip)", () => {
    it("roundtrips through rowToDemoPoll → demoPollToRow", () => {
        const originalRow = {
            id: "poll-rt",
            poll_id: "99",
            creator: "wallet-rt",
            title: "Roundtrip test",
            description: "Testing roundtrip",
            category: "Tech",
            image_url: "",
            option_images: [],
            options: ["X", "Y", "Z"],
            vote_counts: [1, 2, 3],
            unit_price_cents: 10000000,
            end_time: 1700000000,
            total_pool_cents: 60000000,
            creator_investment_cents: 50000000,
            platform_fee_cents: 500000,
            creator_reward_cents: 500000,
            status: 0,
            winning_option: 255,
            total_voters: 6,
            created_at: "1699000000",
        };

        const poll = rowToDemoPoll(originalRow);
        const backToRow = demoPollToRow(poll);

        expect(backToRow.id).toBe(originalRow.id);
        expect(backToRow.title).toBe(originalRow.title);
        expect(backToRow.options).toEqual(originalRow.options);
        expect(backToRow.vote_counts).toEqual(originalRow.vote_counts);
        expect(backToRow.total_pool_cents).toBe(originalRow.total_pool_cents);
    });
});

describe("withFreshPeriods", () => {
    const NOW = Date.now();
    const WEEK_MS = 7 * 24 * 60 * 60 * 1000;
    const MONTH_MS = 30 * 24 * 60 * 60 * 1000;

    function makeUser(overrides: Partial<ReturnType<typeof createPlaceholderUser>>) {
        return { ...createPlaceholderUser("wallet-test"), ...overrides };
    }

    it("does not reset counters when periods have not elapsed", () => {
        const user = makeUser({
            weeklyVotesCast: 5,
            monthlyVotesCast: 10,
            weeklyResetTs: NOW - 1000,
            monthlyResetTs: NOW - 1000,
        });
        const refreshed = withFreshPeriods(user);
        expect(refreshed.weeklyVotesCast).toBe(5);
        expect(refreshed.monthlyVotesCast).toBe(10);
    });

    it("resets weekly counters when a week has elapsed", () => {
        const user = makeUser({
            weeklyVotesCast: 8,
            weeklyWinningsLamports: 500,
            weeklySpentLamports: 200,
            weeklyPollsWon: 3,
            weeklyPollsVoted: 5,
            monthlyVotesCast: 15,
            weeklyResetTs: NOW - WEEK_MS - 1,
            monthlyResetTs: NOW - 1000,
        });
        const refreshed = withFreshPeriods(user);
        expect(refreshed.weeklyVotesCast).toBe(0);
        expect(refreshed.weeklyWinningsLamports).toBe(0);
        expect(refreshed.weeklySpentLamports).toBe(0);
        expect(refreshed.weeklyPollsWon).toBe(0);
        expect(refreshed.weeklyPollsVoted).toBe(0);
        // Monthly should be untouched
        expect(refreshed.monthlyVotesCast).toBe(15);
    });

    it("resets monthly counters when a month has elapsed", () => {
        const user = makeUser({
            weeklyVotesCast: 3,
            monthlyVotesCast: 20,
            monthlyWinningsLamports: 1000,
            monthlySpentLamports: 400,
            monthlyPollsWon: 7,
            monthlyPollsVoted: 12,
            weeklyResetTs: NOW - 1000,
            monthlyResetTs: NOW - MONTH_MS - 1,
        });
        const refreshed = withFreshPeriods(user);
        expect(refreshed.monthlyVotesCast).toBe(0);
        expect(refreshed.monthlyWinningsLamports).toBe(0);
        expect(refreshed.monthlySpentLamports).toBe(0);
        expect(refreshed.monthlyPollsWon).toBe(0);
        expect(refreshed.monthlyPollsVoted).toBe(0);
        // Weekly should be untouched
        expect(refreshed.weeklyVotesCast).toBe(3);
    });

    it("resets both weekly and monthly when both have elapsed", () => {
        const user = makeUser({
            weeklyVotesCast: 5,
            monthlyVotesCast: 20,
            weeklyResetTs: NOW - WEEK_MS - 1,
            monthlyResetTs: NOW - MONTH_MS - 1,
        });
        const refreshed = withFreshPeriods(user);
        expect(refreshed.weeklyVotesCast).toBe(0);
        expect(refreshed.monthlyVotesCast).toBe(0);
    });
});
