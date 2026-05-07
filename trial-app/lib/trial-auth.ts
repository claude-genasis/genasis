import { NextResponse } from "next/server";

export const TRIAL_SECRET_HEADER = "x-genasis-trial-secret";

/**
 * Authentication for trial bridge routes.
 *
 * Two paths are accepted:
 *   1. `X-Genasis-Trial-Secret` header matches `TRIAL_SHARED_SECRET` —
 *      used by the genasis Rust providers running server-side or by any
 *      cross-origin caller.
 *   2. The request looks same-origin via `Sec-Fetch-Site: same-origin` —
 *      used by the trial-app's own UI components (LiveKanbanBoard /
 *      LiveChatThread) so the secret never has to leak to the browser.
 *      Browsers attach this header automatically; server-to-server
 *      clients (curl, reqwest) do NOT, so they must present the secret.
 */
export function checkTrialSecret(req: Request): NextResponse | null {
  const expected = process.env.TRIAL_SHARED_SECRET;
  const provided = req.headers.get(TRIAL_SECRET_HEADER);
  if (expected && provided === expected) return null;
  if (req.headers.get("sec-fetch-site") === "same-origin") return null;
  if (!expected) {
    return NextResponse.json(
      { error: "trial_bridge_not_configured" },
      { status: 503 },
    );
  }
  return NextResponse.json({ error: "unauthorized" }, { status: 401 });
}
