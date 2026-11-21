import { test, expect } from "@playwright/test";

// Runs against the Vite dev server (see playwright.config.ts), not the
// bundled Tauri app -- enough to catch "it doesn't even render" breakage
// without needing a native build.
test("device list renders demo devices", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("Aosu Desktop (inoffiziell)")).toBeVisible();
  await expect(page.getByText("Terasse Norden (Demo)")).toBeVisible();
});

test("diagnostics tab shows capability table", async ({ page }) => {
  await page.goto("/");
  await page.getByText("Diagnose").click();
  await expect(page.getByLabel("Fähigkeiten")).toBeVisible();
});
