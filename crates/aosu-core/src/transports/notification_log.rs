//! Reads the normalized JSONL file `tools/notification_bridge.py
//! --jsonl-out` produces (see `docs/event-pipeline.md`) and exposes it
//! as a `Transport`, backed by a persistent, queryable `EventStore`
//! (SQLite, local file next to the JSONL -- `<path>.sqlite`). The
//! concrete integration point that lets the desktop app show *real*
//! captured events, independent of whatever Home Assistant also reads
//! from the same JSONL file.
//!
//! Deliberately read-only and append-log-shaped: no commands, no live
//! video (a push notification carries no media), no local_key needed.
//! Devices are synthesized from whichever camera names have appeared in
//! the log -- this transport genuinely doesn't know more than that.
//!
//! `rusqlite::Connection` is `Send` but not `Sync`, and `Transport`
//! requires both -- the store is wrapped in a `Mutex` for that reason,
//! not because concurrent access is actually expected (callers already
//! go through an outer `Mutex<Box<dyn Transport>>` in the Tauri layer).

use crate::capability::{Capability, CapabilityReport, Support};
use crate::device::*;
use crate::event_store::{DeviceActivity, EventAnalytics, EventQuery, EventStore, StoredEvent};
use crate::transport::{ExportFormat, Result, Transport, TransportError};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct NotificationLogTransport {
    pub jsonl_path: PathBuf,
    store: Mutex<EventStore>,
}

impl NotificationLogTransport {
    pub fn new(jsonl_path: impl Into<PathBuf>) -> Self {
        let jsonl_path: PathBuf = jsonl_path.into();
        let db_path = derive_db_path(&jsonl_path);
        let store = EventStore::open(&db_path).unwrap_or_else(|_| {
            EventStore::open_in_memory().expect("in-memory store always opens")
        });
        Self {
            jsonl_path,
            store: Mutex::new(store),
        }
    }

    /// Re-reads the JSONL file and ingests any new lines (idempotent,
    /// cheap to call often -- see EventStore::ingest_jsonl_file).
    fn refresh(&self) {
        let store = self.store.lock().expect("event store mutex poisoned");
        let _ = store.ingest_jsonl_file(&self.jsonl_path);
    }

    fn refresh_and_lock(&self) -> std::sync::MutexGuard<'_, EventStore> {
        self.refresh();
        self.store.lock().expect("event store mutex poisoned")
    }
}

fn derive_db_path(jsonl_path: &std::path::Path) -> PathBuf {
    let mut os = jsonl_path.as_os_str().to_os_string();
    os.push(".sqlite");
    PathBuf::from(os)
}

fn event_kind_from_wire_name(event_type: &str) -> EventKind {
    match event_type {
        "MotionDetected" => EventKind::Motion,
        "HumanDetected" => EventKind::Person,
        "VehicleDetected" => EventKind::Vehicle,
        "PackageDetected" => EventKind::Package,
        "DoorbellPressed" => EventKind::Doorbell,
        "TamperDetected" => EventKind::Tamper,
        "BatteryLow" => EventKind::LowBattery,
        "IntercomCall" => EventKind::IntercomCall,
        other => EventKind::Other(other.to_string()),
    }
}

impl Transport for NotificationLogTransport {
    fn name(&self) -> &'static str {
        "push-log"
    }

    fn discover(&self) -> Result<Vec<Device>> {
        self.devices()
    }

    fn devices(&self) -> Result<Vec<Device>> {
        let store = self.refresh_and_lock();
        let activity = store
            .device_activity()
            .map_err(|e| TransportError::Other(e.to_string()))?;
        let names: BTreeSet<String> = activity.into_iter().map(|a| a.camera_name).collect();
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
        let store = self.refresh_and_lock();
        let q = EventQuery {
            since_ms: Some(since.0 as i64 * 1000),
            limit: 1000,
            ..Default::default()
        };
        let rows = store
            .query(&q)
            .map_err(|e| TransportError::Other(e.to_string()))?;
        Ok(rows.into_iter().map(stored_event_to_event).collect())
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

    fn query_events(&self, query: &EventQuery) -> Result<Vec<StoredEvent>> {
        let store = self.refresh_and_lock();
        store
            .query(query)
            .map_err(|e| TransportError::Other(e.to_string()))
    }

    fn event_analytics(&self, now_ms: i64) -> Result<EventAnalytics> {
        let store = self.refresh_and_lock();
        store
            .analytics(now_ms)
            .map_err(|e| TransportError::Other(e.to_string()))
    }

    fn device_activity(&self) -> Result<Vec<DeviceActivity>> {
        let store = self.refresh_and_lock();
        store
            .device_activity()
            .map_err(|e| TransportError::Other(e.to_string()))
    }

    fn export_events(&self, query: &EventQuery, format: ExportFormat) -> Result<String> {
        let store = self.refresh_and_lock();
        let result = match format {
            ExportFormat::Json => store.export_json(query),
            ExportFormat::Csv => store.export_csv(query),
        };
        result.map_err(|e| TransportError::Other(e.to_string()))
    }

    fn apply_event_retention(
        &self,
        now_ms: i64,
        max_age_days: Option<u32>,
        max_rows: Option<i64>,
    ) -> Result<i64> {
        let store = self.refresh_and_lock();
        store
            .apply_retention(now_ms, max_age_days, max_rows)
            .map_err(|e| TransportError::Other(e.to_string()))
    }
}

fn stored_event_to_event(row: crate::event_store::StoredEvent) -> Event {
    let kind = event_kind_from_wire_name(&row.event_type);
    let mut metadata = std::collections::BTreeMap::new();
    metadata.insert("channel".to_string(), row.channel);
    metadata.insert("confidence".to_string(), row.confidence);
    if let Some(note) = row.classification_note {
        metadata.insert("classification_note".to_string(), note);
    }
    metadata.insert("event_id".to_string(), row.id);
    Event {
        device: DeviceId(
            row.camera_name
                .unwrap_or_else(|| "unknown-camera".to_string()),
        ),
        severity: kind.default_severity(),
        at: Timestamp((row.timestamp_ms / 1000) as u64),
        thumbnail_url: None,
        kind,
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_jsonl(lines: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.jsonl");
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{line}").unwrap();
        }
        dir
    }

    fn jsonl_path(dir: &tempfile::TempDir) -> PathBuf {
        dir.path().join("events.jsonl")
    }

    #[test]
    fn missing_file_yields_empty_devices_not_an_error() {
        let t = NotificationLogTransport::new("/does/not/exist/events.jsonl");
        assert_eq!(t.devices().unwrap(), vec![]);
    }

    #[test]
    fn real_sample_line_produces_one_camera_device() {
        let line = r#"{"event_type": "MotionDetected", "camera_name": "Kamera 1", "channel": "com.glazero.android.event.v2", "timestamp_ms": 1795330875651, "confidence": "low", "classification_note": "generic"}"#;
        let dir = write_temp_jsonl(&[line]);
        let t = NotificationLogTransport::new(jsonl_path(&dir));
        let devices = t.devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Kamera 1");
        assert_eq!(devices[0].health, HealthStatus::Unknown);
    }

    #[test]
    fn events_are_classified_with_preserved_metadata() {
        let line = r#"{"event_type": "DoorbellPressed", "camera_name": "Tuer", "channel": "com.glazero.android.ring.v2", "timestamp_ms": 1000000, "confidence": "high", "classification_note": "channel"}"#;
        let dir = write_temp_jsonl(&[line]);
        let t = NotificationLogTransport::new(jsonl_path(&dir));
        let events = t.events(Timestamp(0)).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Doorbell);
        assert_eq!(events[0].metadata["confidence"], "high");
    }

    #[test]
    fn malformed_lines_are_skipped_not_fatal() {
        let dir = write_temp_jsonl(&[
            "not json at all",
            "",
            r#"{"event_type": "MotionDetected", "camera_name": "C", "channel": "x", "timestamp_ms": 1, "confidence": "low", "classification_note": ""}"#,
        ]);
        let t = NotificationLogTransport::new(jsonl_path(&dir));
        assert_eq!(t.devices().unwrap().len(), 1);
    }

    #[test]
    fn unknown_event_type_maps_to_other_and_stays_visible() {
        let line = r#"{"event_type": "UnknownVendorEvent", "camera_name": "C", "channel": "weird", "timestamp_ms": 1, "confidence": "none", "classification_note": ""}"#;
        let dir = write_temp_jsonl(&[line]);
        let t = NotificationLogTransport::new(jsonl_path(&dir));
        let events = t.events(Timestamp(0)).unwrap();
        assert_eq!(
            events[0].kind,
            EventKind::Other("UnknownVendorEvent".to_string())
        );
    }

    #[test]
    fn capability_report_marks_live_video_and_ptz_unsupported() {
        let t = NotificationLogTransport::new("/does/not/exist/events.jsonl");
        let report = t.capability_report();
        assert_eq!(report.get(Capability::LiveVideo), Support::Unsupported);
        assert_eq!(report.get(Capability::Events), Support::Supported);
    }

    #[test]
    fn events_persist_across_a_new_transport_instance_pointed_at_the_same_file() {
        let line = r#"{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1000, "confidence": "low", "classification_note": ""}"#;
        let dir = write_temp_jsonl(&[line]);
        {
            let t = NotificationLogTransport::new(jsonl_path(&dir));
            assert_eq!(t.devices().unwrap().len(), 1);
        }
        // New instance, same files -- the SQLite store must have
        // persisted to disk, not just lived in the first instance's memory.
        let t2 = NotificationLogTransport::new(jsonl_path(&dir));
        assert_eq!(t2.devices().unwrap().len(), 1);
    }

    #[test]
    fn device_activity_reflects_ingested_events() {
        let line = r#"{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1000, "confidence": "low", "classification_note": ""}"#;
        let dir = write_temp_jsonl(&[line]);
        let t = NotificationLogTransport::new(jsonl_path(&dir));
        let activity = t.device_activity().unwrap();
        assert_eq!(activity.len(), 1);
        assert_eq!(activity[0].camera_name, "A");
        assert_eq!(activity[0].event_count, 1);
    }

    #[test]
    fn query_events_supports_filtering_through_the_transport_trait() {
        let dir = write_temp_jsonl(&[
            r#"{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1000, "confidence": "low", "classification_note": ""}"#,
            r#"{"event_type": "TamperDetected", "camera_name": "B", "channel": "x", "timestamp_ms": 2000, "confidence": "high", "classification_note": ""}"#,
        ]);
        let t = NotificationLogTransport::new(jsonl_path(&dir));
        let q = EventQuery {
            camera_name: Some("B".to_string()),
            limit: 10,
            ..Default::default()
        };
        let rows = t.query_events(&q).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_type, "TamperDetected");
    }

    #[test]
    fn event_analytics_and_export_work_through_the_transport_trait() {
        let dir = write_temp_jsonl(&[
            r#"{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1000, "confidence": "low", "classification_note": ""}"#,
        ]);
        let t = NotificationLogTransport::new(jsonl_path(&dir));
        let analytics = t.event_analytics(999_999_999).unwrap();
        assert_eq!(analytics.by_camera, vec![("A".to_string(), 1)]);
        let json = t
            .export_events(&EventQuery::page(10, 0), ExportFormat::Json)
            .unwrap();
        assert!(json.contains("MotionDetected"));
    }

    #[test]
    fn retention_works_through_the_transport_trait() {
        let dir = write_temp_jsonl(&[
            r#"{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 0, "confidence": "low", "classification_note": ""}"#,
        ]);
        let t = NotificationLogTransport::new(jsonl_path(&dir));
        let deleted = t
            .apply_event_retention(999_999_999_999, Some(1), None)
            .unwrap();
        assert_eq!(deleted, 1);
    }

    #[test]
    fn transports_without_an_event_store_report_unsupported() {
        use crate::transports::mock::MockTransport;
        let t = MockTransport::default();
        assert!(matches!(
            t.query_events(&EventQuery::page(10, 0)),
            Err(TransportError::Unsupported)
        ));
    }
}
