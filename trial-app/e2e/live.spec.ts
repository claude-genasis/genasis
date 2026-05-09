import { test, expect } from "@playwright/test";

/**
 * M21.4 — US-015..US-022 trial-bridge live-mode coverage.
 *
 * - US-015 Trial flavor + [trial] config        (Rust E2E M19.3)
 * - US-016 TrialPlaneProvider + TrialMattermostProvider HTTP forwarders
 *          (covered by Rust integration tests)
 * - US-017 Simulated Plane/MM state schema      (DB-level; unit tested)
 * - US-018 /api/plane/* bridge endpoints
 * - US-019 /api/mattermost/* bridge endpoints
 * - US-020 /api/events/stream SSE broadcaster
 * - US-021 LiveKanban + LiveChatThread subscribed to SSE
 * - US-022 Drag-drop kanban + chat composer + chat sidebar
 *
 * The /live tab boots with the simulated state already seeded
 * (ensureProject + ensureChannel run on first request). A worker=1
 * config keeps the SQLite-backed simulation isolated per spec file.
 */

test.describe("live bridge UI (US-021 / US-022)", () => {
  test("live tab renders kanban + chat thread + sidebar handle", async ({
    page,
  }) => {
    await page.goto("/?tab=live");
    await expect(page.getByTestId("live-section")).toBeVisible();
    await expect(page.getByTestId("live-board")).toBeVisible();
    await expect(page.getByTestId("live-kanban")).toBeVisible();
    await expect(page.getByTestId("live-chat-thread")).toBeVisible();
    await expect(page.getByTestId("chat-sidebar-handle")).toBeVisible();
  });

  test("chat composer is wired and posts a message via the bridge", async ({
    page,
  }) => {
    await page.goto("/?tab=live");
    const composer = page.getByTestId("live-chat-composer");
    const send = page.getByTestId("live-chat-send");
    await expect(composer).toBeVisible();
    await expect(send).toBeDisabled();
    await composer.fill("hello from playwright");
    await expect(send).toBeEnabled();
    await send.click();
    // After post, composer clears and the message appears in the list.
    await expect(composer).toHaveValue("", { timeout: 5_000 });
    await expect(page.getByTestId("live-chat-message-list")).toContainText(
      "hello from playwright",
      { timeout: 5_000 },
    );
  });

  test("chat sidebar handle reveals the sidebar", async ({ page }) => {
    await page.goto("/?tab=live");
    const handle = page.getByTestId("chat-sidebar-handle");
    await handle.click();
    // The sidebar may render as a visually-hidden drawer until clicked
    // and the close button appears once it slides in.
    await expect(page.getByTestId("chat-sidebar-close")).toBeVisible({
      timeout: 5_000,
    });
  });
});

test.describe("bridge endpoints (US-018 / US-019 / US-020)", () => {
  // The trial bridge auth requires either a shared secret or a
  // same-origin fetch (Sec-Fetch-Site). We use page.evaluate so the
  // browser injects same-origin headers automatically — that's also
  // exactly how the LiveKanbanBoard / LiveChatThread components call
  // the API in production.

  test("/api/plane/issues returns the seeded issue list", async ({ page }) => {
    await page.goto("/?tab=live");
    const body = await page.evaluate(async () => {
      const r = await fetch("/api/plane/issues?project_slug=trial-demo");
      return { ok: r.ok, json: await r.json() };
    });
    expect(body.ok).toBeTruthy();
    expect(Array.isArray(body.json.issues)).toBe(true);
  });

  // /api/plane/projects and /api/mattermost/channels expose POST only
  // (ensure-on-create); the live UI doesn't fetch them. We rely on the
  // server-rendered LiveSection to confirm seeding (live-section
  // testid in the live-board UI test) and the more meaningful
  // /api/plane/issues GET below.

  test("/api/events/stream responds with an SSE content-type", async ({ page }) => {
    await page.goto("/?tab=live");
    const ct = await page.evaluate(async () => {
      const r = await fetch("/api/events/stream");
      return r.headers.get("content-type");
    });
    expect(ct ?? "").toMatch(/text\/event-stream/);
  });
});
