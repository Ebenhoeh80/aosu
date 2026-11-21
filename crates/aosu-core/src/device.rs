//! Transport-agnostic device and event model. Deliberately brand-agnostic:
//! `vendor`/`model`/`kind` are free strings so a second (even entirely
//! fictional, for architecture validation) vendor needs no changes here.

use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    Motion,
    Person,
    Doorbell,
    LowBattery,
    Tamper,
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
            | EventKind::Doorbell
            | EventKind::System
            | EventKind::Diagnostic => Severity::Info,
            EventKind::Other(_) => Severity::Info,
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
}
