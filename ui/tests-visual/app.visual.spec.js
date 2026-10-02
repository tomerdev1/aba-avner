import { expect, test } from "@playwright/test";

async function openState(page, state) {
  await page.goto(`/index.html?visual-test-state=${state}`);
  await page.waitForLoadState("networkidle");
}

test("idle state matches baseline", async ({ page }) => {
  await openState(page, "idle");
  await expect(page.locator(".shell")).toHaveScreenshot("idle-card.png");
});

test("idle RTL state matches baseline", async ({ page }) => {
  await openState(page, "idle-rtl");
  await expect(page.locator(".shell")).toHaveScreenshot("idle-rtl-card.png");
});

test("settings state matches baseline", async ({ page }) => {
  await openState(page, "settings");
  await expect(page.locator(".shell")).toHaveScreenshot("settings-card.png");
});

test("settings RTL state matches baseline", async ({ page }) => {
  await openState(page, "settings-rtl");
  await expect(page.locator(".shell")).toHaveScreenshot("settings-rtl-card.png");
});

test("progress state matches baseline", async ({ page }) => {
  await openState(page, "progress");
  await expect(page.locator(".shell")).toHaveScreenshot("progress-card.png");
});

test("progress RTL state matches baseline", async ({ page }) => {
  await openState(page, "progress-rtl");
  await expect(page.locator(".shell")).toHaveScreenshot("progress-rtl-card.png");
});

test("canceling state matches baseline", async ({ page }) => {
  await openState(page, "canceling");
  await expect(page.locator(".shell")).toHaveScreenshot("canceling-card.png");
});

test("canceling RTL state matches baseline", async ({ page }) => {
  await openState(page, "canceling-rtl");
  await expect(page.locator(".shell")).toHaveScreenshot("canceling-rtl-card.png");
});

test("complete state matches baseline", async ({ page }) => {
  await openState(page, "complete");
  await expect(page.locator(".shell")).toHaveScreenshot("complete-card.png");
});

test("complete RTL state matches baseline", async ({ page }) => {
  await openState(page, "complete-rtl");
  await expect(page.locator(".shell")).toHaveScreenshot("complete-rtl-card.png");
});

test("review state matches baseline", async ({ page }) => {
  await openState(page, "review");
  await expect(page.locator(".shell")).toHaveScreenshot("review-card.png");
});

test("review RTL state matches baseline", async ({ page }) => {
  await openState(page, "review-rtl");
  await expect(page.locator(".shell")).toHaveScreenshot("review-rtl-card.png");
});

test("warning flood state matches baseline", async ({ page }) => {
  await openState(page, "warning-flood");
  await expect(page.locator(".shell")).toHaveScreenshot("warning-flood-card.png");
});

test("warning flood RTL state matches baseline", async ({ page }) => {
  await openState(page, "warning-flood-rtl");
  await expect(page.locator(".shell")).toHaveScreenshot("warning-flood-rtl-card.png");
});

test("warning flood state matches baseline at narrow width", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 900 });
  await openState(page, "warning-flood");
  await expect(page.locator(".shell")).toHaveScreenshot("warning-flood-card-narrow.png");
});

test("failure flood state matches baseline", async ({ page }) => {
  await openState(page, "failure-flood");
  await expect(page.locator(".shell")).toHaveScreenshot("failure-flood-card.png");
});

test("failure flood RTL state matches baseline", async ({ page }) => {
  await openState(page, "failure-flood-rtl");
  await expect(page.locator(".shell")).toHaveScreenshot("failure-flood-rtl-card.png");
});
