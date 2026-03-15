import { createAdminRpcHandler } from "../_handler";
import { SettlePollInput, zodValidator } from "../_validation";

export const POST = createAdminRpcHandler(
    "settle_poll_atomic",
    (wallet, body) => ({
        p_wallet: wallet,
        p_poll_id: body.p_poll_id,
        // p_winning_option: 0–19 = explicit winner (prediction market admin override);
        // 255 (default) = auto-determine from vote counts.
        p_winning_option: body.p_winning_option ?? 255,
    }),
    zodValidator(SettlePollInput)
);
