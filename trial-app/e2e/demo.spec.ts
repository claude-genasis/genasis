import { test, expect } from "@playwright/test";

/**
 * M21.2 — US-001..US-006 demo-mode coverage.
 *
 * - US-001 Next.js 15 boot + metadata title  (smoke spec already)
 * - US-003 App bar with two-tab nav          (smoke spec already)
 * - US-004 Static kanban with three columns
 * - US-005 Static chat thread with messages
 * - US-006 Scripted 8-step demo sprint state machine
 *
 * The demo-board lives at `?tab=demo` (default tab). The simulated
 * sprint runs entirely in-browser, so it never touches Plane/MM —
 * which makes it the cheapest UI to gate v0.1.0 against.
 */

test.describe("demo mode (US-004 / US-005 / US-006)", () => {
  test("kanban shows the three canonical columns", async ({ page }) => {
    await page.goto("/");
    const kanban = page.getByTestId("kanban-board");
    await expect(kanban).toBeVisible();
    // Look for column headings in either locale.
    const columns = kanban.locator("[data-column]");
    await expect(columns).toHaveCount(3);
  });

  test("chat thread renders with the typing indicator hidden initially", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByTestId("chat-thread")).toBeVisible();
    await expect(page.getByTestId("chat-typing-indicator")).toBeHidden();
  });

  test("[Run Demo Sprint] kicks off the scripted sequence", async ({ page }) => {
    await page.goto("/");
    const board = page.getByTestId("demo-board");
    await expect(board).toBeVisible();
    await expect(board).toHaveAttribute("data-status", "idle");

    await page.getByTestId("demo-run-button").click();

    // Status flips to running once the script starts.
    await expect(board).toHaveAttribute("data-status", "running", {
      timeout: 5_000,
    });

    // Wait for chat messages to start landing (US-005 + US-006).
    await expect(page.getByTestId("chat-message-list").locator("> *").first()).toBeVisible(
      { timeout: 10_000 }
    );
  });

  test("[Reset] button returns the board to idle", async ({ page }) => {
    await page.goto("/");
    await page.getByTestId("demo-run-button").click();
    // Let a couple of script steps fire so the reset is meaningful.
    await page.waitForTimeout(2_000);
    await page.getByTestId("demo-reset-button").click();
    await expect(page.getByTestId("demo-board")).toHaveAttribute(
      "data-status",
      "idle",
    );
  });
});
