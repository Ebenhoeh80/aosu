// Mirrors crates/aosu-core/src/{device,capability}.rs so the UI and the
// Rust core agree on shape. Kept hand-written (not codegen) for now; if
// this drifts, api.test.ts's DTO-shape tests should catch it.

export type DeviceId = string;

export type HealthStatus =
  | "Healthy"
  | "Degraded"
  | "Offline"
  | "Unsupported"
  | "PartiallySupported"
  | "Unknown";

export interface Device {
  id: DeviceId;
  name: string;
  vendor: string;
  model: string;
  kind: string;
  online: boolean;
  health: HealthStatus;
  batteryPercent: number | null;
  wifiSignalPercent: number | null;
  firmwareVersion: string | null;
}

export type Severity = "Info" | "Warning" | "Critical";

export type EventKind =
  | "Motion"
  | "Person"
  | "Doorbell"
  | "LowBattery"
  | "Tamper"
  | "System"
  | "Connectivity"
  | "Diagnostic"
  | string;

export interface DeviceEvent {
  device: DeviceId;
  kind: EventKind;
  severity: Severity;
  atEpochSeconds: number;
}

export interface AppNotification {
  id: string;
  device: DeviceId | null;
  severity: Severity;
  title: string;
  body: string;
  atEpochSeconds: number;
  acknowledged: boolean;
}

export type Support =
  | { type: "supported" }
  | { type: "blocked"; reason: string }
  | { type: "unsupported" }
  | { type: "unknown" };

export type Capability =
  | "discover"
  | "deviceList"
  | "deviceStatus"
  | "events"
  | "notifications"
  | "liveVideo"
  | "twoWayAudio"
  | "ptz"
  | "recordings"
  | "commands";

export type CapabilityReport = Partial<Record<Capability, Support>>;

export interface PortCheck {
  port: number;
  open: boolean;
}

export interface DiagnosticsBundle {
  generatedAtEpochSeconds: number;
  appVersion: string;
  os: string;
  arch: string;
  activeTransport: string;
  capabilityReport: CapabilityReport;
  network: { host: string; ports: PortCheck[] } | null;
}
