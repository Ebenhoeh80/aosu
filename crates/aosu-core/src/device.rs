//! Transport-agnostic device and event model.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub model: String,
    /// e.g. "camera", "base_station". Kept as a string so new hardware
    /// doesn't require a core-crate release.
    pub kind: String,
    pub online: bool,
    pub battery_percent: Option<u8>,
    pub wifi_signal_percent: Option<u8>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    Motion,
    Person,
    Doorbell,
    LowBattery,
    Tamper,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub device: DeviceId,
    pub kind: EventKind,
    pub at: Timestamp,
    pub thumbnail_url: Option<String>,
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
