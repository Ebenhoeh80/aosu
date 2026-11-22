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

// Canonical normalized-event vocabulary, see docs/event-pipeline.md.
// Matches aosu-core's EventKind::wire_name() -- the wire format always
// uses these names, even though Rust's own enum variants are named
// more tersely (Motion, Person, ...) by Rust convention.
export type EventKind =
  | "MotionDetected"
  | "HumanDetected"
  | "VehicleDetected"
  | "PackageDetected"
  | "DoorbellPressed"
  | "BatteryLow"
  | "TamperDetected"
  | "IntercomCall"
  | "System"
  | "Connectivity"
  | "Diagnostic"
  | "UnknownVendorEvent"
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
