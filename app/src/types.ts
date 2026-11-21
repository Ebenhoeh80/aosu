// Mirrors crates/aosu-core/src/{device,capability}.rs so the UI and the
// Rust core agree on shape. Kept hand-written (not codegen) for now; if
// this drifts, a Tauri command round-trip test will catch it (see
// src-tauri/src/commands.rs).

export type DeviceId = string;

export interface Device {
  id: DeviceId;
  name: string;
  model: string;
  kind: "camera" | "base_station" | string;
  online: boolean;
  batteryPercent: number | null;
  wifiSignalPercent: number | null;
}

export type EventKind =
  | "motion"
  | "person"
  | "doorbell"
  | "lowBattery"
  | "tamper"
  | string;

export interface DeviceEvent {
  device: DeviceId;
  kind: EventKind;
  atEpochSeconds: number;
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
  | "liveVideo"
  | "twoWayAudio"
  | "ptz"
  | "recordings"
  | "commands";

export type CapabilityReport = Partial<Record<Capability, Support>>;
