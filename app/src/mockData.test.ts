import { describe, expect, it } from "vitest";
import { mockAvailableTransports, mockCapabilityReport } from "./mockData";
import type { Capability } from "./types";

const BUILTIN_TRANSPORTS = [
  "cloud",
  "kestrel-mock",
  "local",
  "mock",
  "push-log",
  "tuya-cloud",
];

const ALL_CAPABILITIES: Capability[] = [
  "discover",
  "deviceList",
  "deviceStatus",
  "events",
  "notifications",
  "liveVideo",
  "twoWayAudio",
  "ptz",
  "recordings",
  "commands",
];

describe("offline fallback data", () => {
  it("lists exactly the transports the Rust registry registers", () => {
    expect([...mockAvailableTransports()].sort()).toEqual(
      [...BUILTIN_TRANSPORTS].sort(),
    );
  });

  it("reports every capability so the UI never sees an unknown feature", () => {
    const report = mockCapabilityReport();
    for (const capability of ALL_CAPABILITIES) {
      expect(report[capability], `missing ${capability}`).toBeDefined();
    }
  });

  it("does not claim PTZ support for the mock transport", () => {
    expect(mockCapabilityReport().ptz).toEqual({ type: "unsupported" });
  });
});
