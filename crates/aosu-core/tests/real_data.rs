//! Integration test proving the Rust `NotificationLogTransport` reads
//! the *actual* JSONL bytes `tools/event_pipeline.py` produces from a
//! real captured notification (camera name anonymized to a placeholder,
//! everything else byte-identical to the real output) -- not a
//! hand-written guess at the format. See docs/event-pipeline.md.

use aosu_core::device::{EventKind, HealthStatus, Timestamp};
use aosu_core::transports::notification_log::NotificationLogTransport;
use aosu_core::Transport;

#[test]
fn reads_a_real_captured_and_anonymized_event() {
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/real_capture_anonymized.jsonl"
    );
    let transport = NotificationLogTransport::new(fixture);

    let devices = transport.devices().unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].name, "Kamera 1");
    assert_eq!(devices[0].health, HealthStatus::Unknown);

    let events = transport.events(Timestamp(0)).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, EventKind::Motion);
    assert_eq!(events[0].metadata["confidence"], "low");
    assert!(events[0].metadata["classification_note"].contains("Korrekturbefund"));
}
