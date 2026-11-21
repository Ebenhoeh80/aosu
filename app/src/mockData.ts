import type { CapabilityReport, Device, DeviceEvent } from "./types";

// Deliberately mirrors crates/aosu-core/src/transports/mock.rs so the
// frontend can be built and demoed before any Tauri command is wired to
// the real Rust MockTransport.

export function mockDevices(): Device[] {
  return [
    {
      id: "mock-cam-1",
      name: "Terasse Norden (Demo)",
      model: "C7S",
      kind: "camera",
      online: true,
      batteryPercent: 81,
      wifiSignalPercent: 70,
    },
    {
      id: "mock-cam-2",
      name: "Garten Westen (Demo)",
      model: "C7S",
      kind: "camera",
      online: true,
      batteryPercent: 54,
      wifiSignalPercent: 54,
    },
    {
      id: "mock-base",
      name: "Basisstation (Demo)",
      model: "H1L",
      kind: "base_station",
      online: true,
      batteryPercent: null,
      wifiSignalPercent: null,
    },
  ];
}

export function mockEvents(): DeviceEvent[] {
  return [
    { device: "mock-cam-1", kind: "motion", atEpochSeconds: Math.floor(Date.now() / 1000) },
  ];
}

export function mockCapabilityReport(): CapabilityReport {
  return {
    discover: { type: "supported" },
    deviceList: { type: "supported" },
    deviceStatus: { type: "supported" },
    events: { type: "supported" },
    liveVideo: { type: "supported" },
    twoWayAudio: { type: "supported" },
    recordings: { type: "supported" },
    commands: { type: "supported" },
    ptz: { type: "unsupported" },
  };
}
