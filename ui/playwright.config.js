import { defineConfig, devices } from "@playwright/test";

const visualMode = process.env.VISUAL_MODE === "headed" ? "headed" : "headless";

export default defineConfig({
  testDir: "./tests-visual",
  snapshotPathTemplate: `{testDir}/__screenshots__/${visualMode}/{testFilePath}/{arg}{ext}`,
  use: {
    baseURL: "http://127.0.0.1:4173",
    headless: visualMode !== "headed",
    viewport: { width: 960, height: 720 },
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 960, height: 720 },
      },
    },
  ],
  webServer: {
    command: "python3 -m http.server 4173 --bind 127.0.0.1",
    port: 4173,
    cwd: ".",
    reuseExistingServer: true,
  },
});
