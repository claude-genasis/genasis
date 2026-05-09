#!/usr/bin/env node
// Spawned as a sub-process by genasis-cli (`crates/genasis-cli/src/cmd_init.rs`).
// Drives the Plane web UI via Playwright to create per-agent user accounts
// and issue per-user Personal Access Tokens, because Plane's REST API does
// not expose user creation directly.
//
// Protocol (stdio):
//   stdin   : JSON object { plane_url, admin_email, admin_password,
//             workspace_slug,
//             agents: [{ role, email }, ...],
//             humans: [{ name, email, role?, plane_role? }, ...] }
//   stdout  : JSON object { agents: [{ role, email, user_id, pat }, ...],
//             humans: [{ email, user_id, status }, ...],
//             status: "ok" | "error", error?: "..." }
//   exit 0  : success
//   exit 2  : recoverable error (printed in stdout JSON)
//   exit 3  : Playwright not installed / driver error
//
// Humans differ from agents in that they receive workspace Member role
// (not a bot) and no PAT is issued — they authenticate through the
// Plane UI like any other user. Idempotency: if a human's email is
// already a workspace member, return status="joined"; if the invite
// was just sent, return status="invited".
//
// The Rust caller (M4) parses stdout JSON and surfaces errors. Real
// browser automation logic lands as the original Genesis bash script's
// Playwright code is ported milestone-by-milestone — for now this script
// returns a recognisable "not-implemented" envelope so the integration
// boundary is testable.

import process from "node:process";

let raw = "";
for await (const chunk of process.stdin) {
    raw += chunk;
}

let input;
try {
    input = raw.trim() ? JSON.parse(raw) : {};
} catch (e) {
    process.stdout.write(
        JSON.stringify({
            status: "error",
            error: `bad input json: ${e.message}`,
        }) + "\n",
    );
    process.exit(2);
}

let playwright;
try {
    playwright = await import("playwright");
} catch (e) {
    process.stdout.write(
        JSON.stringify({
            status: "error",
            error: `playwright not installed; run: npm install --prefix $(dirname $0) && npx playwright install chromium`,
            hint: e.message,
        }) + "\n",
    );
    process.exit(3);
}

// M4 boundary: the actual UI automation is ported in a follow-up commit.
// For now we emit a structured "stub" so the Rust caller can verify the
// plumbing works end-to-end. The Rust caller treats `status: "stub"` as
// a no-op success so init does not fail.
const humans = Array.isArray(input.humans) ? input.humans : [];
const stub = {
    status: "stub",
    note: "Playwright is installed and reachable; UI automation is ported incrementally (M4-port).",
    received: {
        plane_url: input.plane_url ?? null,
        workspace_slug: input.workspace_slug ?? null,
        agent_roles: Array.isArray(input.agents)
            ? input.agents.map((a) => a.role)
            : [],
        human_emails: humans.map((h) => h.email),
    },
    // Echo a stub humans payload so the Rust caller's lock-file writer
    // has a deterministic shape to consume even before real UI
    // automation lands.
    humans: humans.map((h) => ({
        email: String(h.email || "").toLowerCase(),
        user_id: `stub-plane-${String(h.email || "").toLowerCase()}`,
        status: "stub",
    })),
    playwright_version: playwright?.default?.version ?? "unknown",
};
process.stdout.write(JSON.stringify(stub) + "\n");
process.exit(0);
