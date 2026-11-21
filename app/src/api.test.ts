import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { api, wasLastCallMocked } from "./api";

describe("api (offline-first wrapper)", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("returns the real Tauri result when invoke succeeds", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([{ id: "real-1" }]);
    const devices = await api.listDevices();
    expect(devices).toEqual([{ id: "real-1" }]);
    expect(wasLastCallMocked()).toBe(false);
  });

  it("falls back to mock devices when invoke fails (no Tauri runtime)", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("no tauri context"));
    const devices = await api.listDevices();
    expect(devices.length).toBeGreaterThan(0);
    expect(wasLastCallMocked()).toBe(true);
  });

  it("falls back for every endpoint, not just listDevices", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("no tauri context"));
    await expect(api.recentEvents()).resolves.not.toHaveLength(0);
    await expect(api.listNotifications()).resolves.toBeInstanceOf(Array);
    await expect(api.listAvailableTransports()).resolves.toContain("mock");
    await expect(api.activeTransport()).resolves.toBe("mock");
    const diag = await api.exportDiagnostics();
    expect(JSON.parse(diag)).toHaveProperty("activeTransport");
  });

  it("acknowledging a notification in mock mode persists across calls", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("no tauri context"));
    const before = await api.listNotifications();
    const target = before[0];
    await api.acknowledgeNotification(target.id);
    const after = await api.listNotifications();
    const updated = after.find((n) => n.id === target.id);
    expect(updated?.acknowledged).toBe(true);
  });
});
