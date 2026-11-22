//! Transport-agnostic device and event model. Deliberately brand-agnostic:
//! `vendor`/`model`/`kind` are free strings so a second (even entirely
//! fictional, for architecture validation) vendor needs no changes here.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceId(pub String);

/// Coarse health summary a device list can render a badge for without
/// the UI needing to know transport-specific detail. `Unknown` is a
/// legitimate, explicit state -- see `HealthStatus::Unknown` doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Offline,
    /// The transport recognizes the device but doesn't (yet) support it
    /// at all -- distinct from `PartiallySupported`, where some
    /// capabilities work.
    Unsupported,
    PartiallySupported,
    /// Never collapse an unknown state into `Healthy` or `Offline` just
    /// to have *something* to show -- see product requirement "do not
    /// hide uncertainty".
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub vendor: String,
    pub model: String,
    /// e.g. "camera", "base_station". Kept as a string so new hardware
    /// doesn't require a core-crate release.
    pub kind: String,
    pub online: bool,
    pub health: HealthStatus,
    pub battery_percent: Option<u8>,
    pub wifi_signal_percent: Option<u8>,
    pub firmware_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn now() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self(secs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// Vendor-agnostic normalized event type. See `docs/event-pipeline.md`
/// for the full classification rules (this is the Rust side of a
/// two-language contract also implemented in `tools/event_pipeline.py`,
/// verified by parity tests against the same fixtures -- not a single
/// shared binary, since the Python side runs where `adb` is available
/// and this side runs inside the desktop app / Home Assistant).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    Motion,
    Person,
    Vehicle,
    Package,
    Doorbell,
    LowBattery,
    Tamper,
    /// A two-way audio/intercom call, e.g. Aosu's "call.v2" channel.
    IntercomCall,
    /// The device/transport itself changed state (came online, went
    /// offline, degraded) -- distinct from something the camera *saw*.
    System,
    /// Network-level: reconnect, lost signal, DNS/relay failure, etc.
    Connectivity,
    /// Produced by the diagnostics subsystem, not by a device.
    Diagnostic,
    Other(String),
}

impl EventKind {
    pub fn default_severity(&self) -> Severity {
        match self {
            EventKind::Tamper => Severity::Critical,
            EventKind::LowBattery | EventKind::Connectivity => Severity::Warning,
            EventKind::Motion
            | EventKind::Person
            | EventKind::Vehicle
            | EventKind::Package
            | EventKind::Doorbell
            | EventKind::IntercomCall
            | EventKind::System
            | EventKind::Diagnostic => Severity::Info,
            EventKind::Other(_) => Severity::Info,
        }
    }

    /// The canonical normalized-event name from `docs/event-pipeline.md`
    /// (`MotionDetected`, `HumanDetected`, ...), used for anything
    /// consumer-facing (Tauri DTOs, HA, logs) instead of Rust's own
    /// idiomatic short variant names (`Motion`, `Person`, ...) -- the
    /// two naming styles are a deliberate per-language convention
    /// difference, not a drift to paper over silently.
    pub fn wire_name(&self) -> String {
        match self {
            EventKind::Motion => "MotionDetected".to_string(),
            EventKind::Person => "HumanDetected".to_string(),
            EventKind::Vehicle => "VehicleDetected".to_string(),
            EventKind::Package => "PackageDetected".to_string(),
            EventKind::Doorbell => "DoorbellPressed".to_string(),
            EventKind::LowBattery => "BatteryLow".to_string(),
            EventKind::Tamper => "TamperDetected".to_string(),
            EventKind::IntercomCall => "IntercomCall".to_string(),
            EventKind::System => "System".to_string(),
            EventKind::Connectivity => "Connectivity".to_string(),
            EventKind::Diagnostic => "Diagnostic".to_string(),
            EventKind::Other(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub device: DeviceId,
    pub kind: EventKind,
    pub severity: Severity,
    pub at: Timestamp,
    pub thumbnail_url: Option<String>,
    /// Never-discarded source detail (raw text, classification
    /// confidence/note, channel id, ...) -- see `docs/event-pipeline.md`.
    /// Empty for events that don't come from the notification pipeline
    /// (e.g. mock-generated ones).
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

/// An in-app notification -- distinct from `Event`: not every event
/// becomes a notification, and a notification tracks its own
/// acknowledgement state independent of the event stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub device: Option<DeviceId>,
    pub severity: Severity,
    pub title: String,
    pub body: String,
    pub at: Timestamp,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: Timestamp,
    pub to: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recording {
    pub device: DeviceId,
    pub started_at: Timestamp,
    pub duration_secs: u32,
    pub source: RecordingSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingSource {
    LocalBaseStation,
    Cloud,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    SetLight(bool),
    SetSiren(bool),
}

#[derive(Debug, Clone)]
pub struct MediaStreamHandle {
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct AudioChannelHandle {
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tamper_defaults_to_critical_severity() {
        assert_eq!(EventKind::Tamper.default_severity(), Severity::Critical);
    }

    #[test]
    fn motion_defaults_to_info_severity() {
        assert_eq!(EventKind::Motion.default_severity(), Severity::Info);
    }

    #[test]
    fn severity_orders_info_below_critical() {
        assert!(Severity::Info < Severity::Critical);
    }

    #[test]
    fn wire_name_matches_the_canonical_vocabulary_from_event_pipeline_doc() {
        assert_eq!(EventKind::Motion.wire_name(), "MotionDetected");
        assert_eq!(EventKind::Person.wire_name(), "HumanDetected");
        assert_eq!(EventKind::Vehicle.wire_name(), "VehicleDetected");
        assert_eq!(EventKind::Package.wire_name(), "PackageDetected");
        assert_eq!(EventKind::Doorbell.wire_name(), "DoorbellPressed");
        assert_eq!(EventKind::LowBattery.wire_name(), "BatteryLow");
        assert_eq!(EventKind::Tamper.wire_name(), "TamperDetected");
        assert_eq!(EventKind::IntercomCall.wire_name(), "IntercomCall");
    }

    #[test]
    fn wire_name_passes_through_other_variants_unchanged() {
        assert_eq!(
            EventKind::Other("UnknownVendorEvent".to_string()).wire_name(),
            "UnknownVendorEvent"
        );
    }
}
