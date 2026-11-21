import { test, expect } from "@playwright/test";

// Runs against the Vite dev server (see playwright.config.ts), not the
// bundled Tauri app -- enough to catch "it doesn't even render" breakage
// without needing a native build. There is no Tauri runtime here, so
// every api.* call exercises the real mock-data fallback path -- this
// is, deliberately, also a check that the app works with zero hardware
// and zero backend reachable.

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.getByText("Überspringen").click(); // skip onboarding
});

test("device inventory renders demo devices across vendors", async ({ page }) => {
  await expect(page.getByText("Aosu Desktop (inoffiziell)")).toBeVisible();
  await expect(page.getByText("Terasse Norden (Demo)")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Kestrel Labs (fiktiv, Demo)" })).toBeVisible();
});

test("shows the demo-mode banner since no real transport is connected", async ({ page }) => {
  await expect(page.getByRole("status").filter({ hasText: "Demo-Modus" })).toBeVisible();
});

test("can drill into a device and back out", async ({ page }) => {
  await page.getByText("Terasse Norden (Demo)").click();
  await expect(page.getByRole("tablist", { name: "Gerätedetails" })).toBeVisible();
  await page.getByText("← Zurück zur Geräteliste").click();
  await expect(page.getByText("Terasse Norden (Demo)")).toBeVisible();
});

test("diagnostics tab shows capability table and can export", async ({ page }) => {
  await page.getByRole("tab", { name: "Diagnose" }).click();
  await expect(page.getByLabel("Fähigkeiten")).toBeVisible();
  await expect(page.getByText(/Diagnosebericht exportieren/)).toBeVisible();
});

test("notifications can be acknowledged", async ({ page }) => {
  await page.getByRole("tab", { name: "Benachrichtigungen" }).click();
  const ackButton = page.getByText("Als gelesen markieren").first();
  await ackButton.click();
  await expect(page.getByText("gelesen").first()).toBeVisible();
});

test("settings tab lists available transports including the fictional second vendor", async ({
  page,
}) => {
  await page.getByRole("tab", { name: "Einstellungen" }).click();
  await expect(page.getByRole("button", { name: "kestrel-mock" })).toBeVisible();
});
