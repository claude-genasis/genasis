import { test, expect } from "@playwright/test";

/**
 * M21.1 smoke — verifies the prod build boots and the two-tab landing
 * page renders. Anchors the harness; deeper US coverage lands in the
 * remaining M21 specs.
 */

test.describe("trial-app smoke", () => {
  test("landing page renders Genasis Trial heading + tab nav", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page).toHaveTitle(/Genasis Trial/i);
    await expect(page.getByRole("link", { name: /체험하기|Try/i })).toBeVisible();
    await expect(page.getByRole("link", { name: /신청하기|Apply/i })).toBeVisible();
  });

  test("?tab=signup query routes to the signup form", async ({ page }) => {
    await page.goto("/?tab=signup");
    await expect(page.locator("#signup-heading")).toBeVisible();
    await expect(page.locator("input[type=email]").first()).toBeVisible();
  });
});
