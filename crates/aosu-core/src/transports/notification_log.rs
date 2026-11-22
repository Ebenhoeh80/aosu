//! Reads the normalized JSONL file `tools/notification_bridge.py
//! --jsonl-out` produces (see `docs/event-pipeline.md`) and exposes it
//! as a `Transport` -- the concrete integration point that lets the
//! desktop app show *real* captured events, independent of whatever
//! Home Assistant also reads from the same file.
//!
//! Deliberately read-only and append-log-shaped: no commands, no live
//! video (a push notification carries no media), no local_key needed.
//! Devices are synthesized from whichever camera names have appeared in
//! the log -- this transport genuinely doesn't know more than that.

use crate::capability::{Capability, CapabilityReport, Support};
use crate::device::*;
use crate::transport::{Result, Transport, TransportError};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

pub struct NotificationLogTransport {
    pub path: PathBuf,
}

impl NotificationLogTransport {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn read_lines(&self) -> Vec<String> {
        fs::read_to_string(&self.path)
            .map(|s| s.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    fn parse_events(&self) -> Vec<(String, Event)> {
        self.read_lines()
            .into_iter()
            .filter_map(|line| parse_jsonl_line(&line))
            .collect()
    }
}

/// Parses one line of the JSONL format `event_pipeline.py` writes,
/// without pulling in a JSON library for a handful of known fields --
/// this file format is specified in `docs/event-pipeline.md` and tested
/// against real captured output, not guessed at.
fn parse_jsonl_line(line: &str) -> Option<(String, Event)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let event_type = value.get("event_type")?.as_str()?;
    let camera_name = value
        .get("camera_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown-camera")
        .to_string();
    let channel = value.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    let confidence = value
        .get("confidence")
        .and_then(|v| v.as_str())
        .unwrap_or("none");
    let note = value
        .get("classification_note")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let when_ms = value.get("timestamp_ms").and_then(|v| v.as_u64());

    let kind = match event_type {
        "MotionDetected" => EventKind::Motion,
        "HumanDetected" => EventKind::Person,
        "VehicleDetected" => EventKind::Vehicle,
        "PackageDetected" => EventKind::Package,
        "DoorbellPressed" => EventKind::Doorbell,
        "TamperDetected" => EventKind::Tamper,
        "BatteryLow" => EventKind::LowBattery,
        "IntercomCall" => EventKind::IntercomCall,
        other => EventKind::Other(other.to_string()),
    };

    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("channel".to_string(), channel.to_string());
    metadata.insert("confidence".to_string(), confidence.to_string());
    metadata.insert("classification_note".to_string(), note.to_string());

    let device = DeviceId(camera_name.clone());
    let event = Event {
        severity: kind.default_severity(),
        device,
        at: when_ms
            .map(|ms| Timestamp(ms / 1000))
            .unwrap_or_else(Timestamp::now),
        thumbnail_url: None,
        kind,
        metadata,
    };
    Some((camera_name, event))
}

impl Transport for NotificationLogTransport {
    fn name(&self) -> &'static str {
        "push-log"
    }

    fn discover(&self) -> Result<Vec<Device>> {
        self.devices()
    }

    fn devices(&self) -> Result<Vec<Device>> {
        let names: BTreeSet<String> = self
            .parse_events()
            .into_iter()
            .map(|(camera, _)| camera)
            .collect();
        Ok(names
            .into_iter()
            .map(|name| Device {
                id: DeviceId(name.clone()),
                name: name.clone(),
                vendor: "Aosu (Push-Log)".to_string(),
                model: "unbekannt".to_string(),
                kind: "camera".to_string(),
                online: true,
                // Genuinely unknown: a push notification says nothing
                // about current online/offline state.
                health: HealthStatus::Unknown,
                battery_percent: None,
                wifi_signal_percent: None,
                firmware_version: None,
            })
            .collect())
    }

    fn events(&self, since: Timestamp) -> Result<Vec<Event>> {
        Ok(self
            .parse_events()
            .into_iter()
            .map(|(_, e)| e)
            .filter(|e| e.at.0 >= since.0)
            .collect())
    }

    fn live(&self, _device: &DeviceId) -> Result<MediaStreamHandle> {
        Err(TransportError::Unsupported)
    }

    fn talk(&self, _device: &DeviceId) -> Result<AudioChannelHandle> {
        Err(TransportError::Unsupported)
    }

    fn recordings(&self, _device: &DeviceId, _range: TimeRange) -> Result<Vec<Recording>> {
        Err(TransportError::Unsupported)
    }

    fn command(&self, _device: &DeviceId, _cmd: Command) -> Result<()> {
        Err(TransportError::Unsupported)
    }

    fn capability_report(&self) -> CapabilityReport {
        let mut r = CapabilityReport::new();
        r.set(Capability::Discover, Support::Supported);
        r.set(Capability::DeviceList, Support::Supported);
        r.set(Capability::Events, Support::Supported);
        r.set(Capability::Notifications, Support::Supported);
        for cap in [
            Capability::DeviceStatus,
            Capability::LiveVideo,
            Capability::TwoWayAudio,
            Capability::Ptz,
            Capability::Recordings,
            Capability::Commands,
        ] {
            r.set(cap, Support::Unsupported);
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_jsonl(lines: &[&str]) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        for line in lines {
            writeln!(f, "{line}").unwrap();
        }
        f
    }

    #[test]
    fn missing_file_yields_empty_devices_not_an_error() {
        let t = NotificationLogTransport::new("/does/not/exist.jsonl");
        assert_eq!(t.devices().unwrap(), vec![]);
    }

    #[test]
    fn real_sample_line_produces_one_camera_device() {
        let line = r#"{"event_type": "MotionDetected", "camera_name": "Kamera 1", "channel": "com.glazero.android.event.v2", "timestamp_ms": 1795330875651, "confidence": "low", "classification_note": "generic", "metadata": {}}"#;
        let f = write_temp_jsonl(&[line]);
        let t = NotificationLogTransport::new(f.path());
        let devices = t.devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Kamera 1");
        assert_eq!(devices[0].health, HealthStatus::Unknown);
    }

    #[test]
    fn events_are_classified_with_preserved_metadata() {
        let line = r#"{"event_type": "DoorbellPressed", "camera_name": "Tür", "channel": "com.glazero.android.ring.v2", "timestamp_ms": 1000000, "confidence": "high", "classification_note": "channel", "metadata": {}}"#;
        let f = write_temp_jsonl(&[line]);
        let t = NotificationLogTransport::new(f.path());
        let events = t.events(Timestamp(0)).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Doorbell);
        assert_eq!(events[0].metadata["confidence"], "high");
    }

    #[test]
    fn malformed_lines_are_skipped_not_fatal() {
        let f = write_temp_jsonl(&[
            "not json at all",
            "",
            r#"{"event_type": "MotionDetected", "camera_name": "C", "channel": "x", "timestamp_ms": 1, "confidence": "low", "classification_note": ""}"#,
        ]);
        let t = NotificationLogTransport::new(f.path());
        assert_eq!(t.devices().unwrap().len(), 1);
    }

    #[test]
    fn unknown_event_type_maps_to_other_and_stays_visible() {
        let line = r#"{"event_type": "UnknownVendorEvent", "camera_name": "C", "channel": "weird", "timestamp_ms": 1, "confidence": "none", "classification_note": ""}"#;
        let f = write_temp_jsonl(&[line]);
        let t = NotificationLogTransport::new(f.path());
        let events = t.events(Timestamp(0)).unwrap();
        assert_eq!(
            events[0].kind,
            EventKind::Other("UnknownVendorEvent".to_string())
        );
    }

    #[test]
    fn capability_report_marks_live_video_and_ptz_unsupported() {
        let t = NotificationLogTransport::new("/does/not/exist.jsonl");
        let report = t.capability_report();
        assert_eq!(report.get(Capability::LiveVideo), Support::Unsupported);
        assert_eq!(report.get(Capability::Events), Support::Supported);
    }
}
