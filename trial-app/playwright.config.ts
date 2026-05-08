import { defineConfig, devices } from "@playwright/test";

/**
 * M21 — trial-app E2E configuration.
 *
 * Each spec under `e2e/` covers one or more user stories from
 * `ralph/prd.json`. The webServer block boots the production build via
 * `next start` so the suite verifies the same artifact that ships to
 * trial.realstory.blog. Set `PLAYWRIGHT_REUSE_SERVER=1` if you already
 * have `npm run dev` running on port 3000.
 */
const PORT = Number(process.env.PORT ?? 3100);
const BASE_URL = `http://127.0.0.1:${PORT}`;

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: [["list"]],
  use: {
    baseURL: BASE_URL,
    trace: "retain-on-failure",
    actionTimeout: 10_000,
  },
  webServer: process.env.PLAYWRIGHT_REUSE_SERVER === "1"
    ? undefined
    : {
        // Build is required because `next start` only serves a built
        // app. We run it inside Playwright's lifecycle so a single
        // `npm run e2e` invocation handles boot + tests + teardown.
        command: `npm run build && PORT=${PORT} npm run start`,
        url: BASE_URL,
        timeout: 180_000,
        reuseExistingServer: false,
        env: {
          PORT: String(PORT),
          NODE_ENV: "production",
          DATABASE_PATH: ".playwright/trial.db",
        },
      },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
