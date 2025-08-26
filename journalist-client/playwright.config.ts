import type { PlaywrightTestConfig } from "@playwright/test";
import { devices } from "@playwright/test";

const config: PlaywrightTestConfig = {
  testDir: "./.storybook",
  snapshotPathTemplate: "{arg}{-projectName}{-platform}{ext}",
  timeout: 25_000,
  expect: {
    timeout: 10_000,
  },
  retries: process.env.CI ? 3 : 0,
  fullyParallel: true,
  workers: process.env.CI ? 6 : 2,
  reporter: [
    [
      "html",
      {
        open: process.env.CI ? "never" : "on-failure",
        outputFolder: "playwright-report",
      },
    ],
  ],
  outputDir: "playwright-output",
  use: {
    actionTimeout: 0,
    baseURL: "http://localhost:6006",
    screenshot: "on",
    trace: "off",
    video: "off",
    viewport: { width: 1280, height: 720 },
  },
  webServer: {
    command: "http-server ./storybook-static --p 6006 --silent",
    url: "http://localhost:6006/iframe.html",
    reuseExistingServer: !process.env.CI,
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
      },
    },
  ],
};

export default config;
