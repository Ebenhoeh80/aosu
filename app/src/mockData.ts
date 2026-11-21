import type {
  AppNotification,
  CapabilityReport,
  Device,
  DeviceEvent,
  DiagnosticsBundle,
} from "./types";

// Deliberately mirrors crates/aosu-core/src/transports/mock.rs: the same
// six devices across every health state, so "offline mode" (no Tauri
// runtime, no hardware) shows the exact same realistic spread as the
// Rust MockTransport would. See api.ts for how/when this is used as a
// fallback.

export function mockDevices(): Device[] {
  return [
    {
      id: "mock-cam-1",
      name: "Terasse Norden (Demo)",
      vendor: "Aosu (Demo)",
      model: "C7S",
      kind: "camera",
      online: true,
      health: "Healthy",
      batteryPercent: 81,
      wifiSignalPercent: 70,
      firmwareVersion: "5.11.20-demo",
    },
    {
      id: "mock-cam-2",
      name: "Garten Westen (Demo)",
      vendor: "Aosu (Demo)",
      model: "C7S",
      kind: "camera",
      online: true,
      health: "Degraded",
      batteryPercent: 12,
      wifiSignalPercent: 24,
      firmwareVersion: "5.11.20-demo",
    },
    {
      id: "mock-cam-3",
      name: "Einfahrt Süden (Demo, offline)",
      vendor: "Aosu (Demo)",
      model: "C7S",
      kind: "camera",
      online: false,
      health: "Offline",
      batteryPercent: null,
      wifiSignalPercent: null,
      firmwareVersion: "5.11.20-demo",
    },
    {
      id: "mock-cam-4",
      name: "Prototyp-Kamera (Demo, nicht unterstützt)",
      vendor: "Aosu (Demo)",
      model: "X9-Unreleased",
      kind: "camera",
      online: true,
      health: "Unsupported",
      batteryPercent: 55,
      wifiSignalPercent: 88,
      firmwareVersion: null,
    },
    {
      id: "mock-cam-5",
      name: "Dachkamera (Demo, teilweise unterstützt)",
      vendor: "Aosu (Demo)",
      model: "C9C",
      kind: "camera",
      online: true,
      health: "PartiallySupported",
      batteryPercent: 64,
      wifiSignalPercent: 40,
      firmwareVersion: "5.11.20-demo",
    },
    {
      id: "mock-base",
      name: "Basisstation (Demo)",
      vendor: "Aosu (Demo)",
      model: "H1L",
      kind: "base_station",
      online: true,
      health: "Healthy",
      batteryPercent: null,
      wifiSignalPercent: null,
      firmwareVersion: "2.4.1-demo",
    },
    // Second, explicitly fictional vendor (see
    // crates/aosu-core/src/transports/vendor2_mock.rs) -- demonstrates
    // the UI handles more than one vendor without change.
    {
      id: "kestrel-ptz-1",
      name: "Kestrel PTZ Dome (Demo)",
      vendor: "Kestrel Labs (fiktiv, Demo)",
      model: "K2-PTZ",
      kind: "camera",
      online: true,
      health: "Healthy",
      batteryPercent: null,
      wifiSignalPercent: 90,
      firmwareVersion: "1.0.0-fictional",
    },
  ];
}

const now = Math.floor(Date.now() / 1000);

export function mockEvents(): DeviceEvent[] {
  return [
    { device: "mock-cam-1", kind: "Motion", severity: "Info", atEpochSeconds: now - 60 },
    { device: "mock-cam-2", kind: "LowBattery", severity: "Warning", atEpochSeconds: now - 300 },
    {
      device: "mock-cam-3",
      kind: "Connectivity",
      severity: "Warning",
      atEpochSeconds: now - 900,
    },
    { device: "mock-base", kind: "System", severity: "Info", atEpochSeconds: now - 1800 },
    { device: "mock-cam-1", kind: "Person", severity: "Info", atEpochSeconds: now - 3600 },
  ];
}

const acknowledgedIds = new Set<string>();

export function mockNotifications(): AppNotification[] {
  return mockEvents()
    .filter((e) => e.severity !== "Info")
    .map((e, i) => ({
      id: `${e.device}-${i}`,
      device: e.device,
      severity: e.severity,
      title: e.kind,
      body: `${e.kind} at ${e.device}`,
      atEpochSeconds: e.atEpochSeconds,
      acknowledged: acknowledgedIds.has(`${e.device}-${i}`),
    }));
}

export function acknowledgeMockNotification(id: string): void {
  acknowledgedIds.add(id);
}

export function mockCapabilityReport(): CapabilityReport {
  return {
    discover: { type: "supported" },
    deviceList: { type: "supported" },
    deviceStatus: { type: "supported" },
    events: { type: "supported" },
    notifications: { type: "supported" },
    liveVideo: { type: "supported" },
    twoWayAudio: { type: "supported" },
    recordings: { type: "supported" },
    commands: { type: "supported" },
    ptz: { type: "unsupported" },
  };
}

export function mockAvailableTransports(): string[] {
  return ["cloud", "kestrel-mock", "local", "mock"];
}

export function mockDiagnosticsBundle(): DiagnosticsBundle {
  return {
    generatedAtEpochSeconds: now,
    appVersion: "0.1.0",
    os: "offline-mock",
    arch: "n/a",
    activeTransport: "mock",
    capabilityReport: mockCapabilityReport(),
    network: null,
  };
}
