//! Rust side of the two-language event-normalization contract. See
//! `docs/event-pipeline.md` for the documented mapping table and the
//! real-device correction this is built from, and `tools/event_pipeline.py`
//! for the Python side (where `adb` capture actually happens). This
//! module re-implements the *same rules*, verified by parity tests
//! against the same fixtures -- there is no single shared binary across
//! the two languages, so "same pipeline" means "same documented rules,
//! independently testable on each side".

use crate::device::{DeviceId, Event, EventKind, Timestamp};
use std::collections::BTreeMap;

pub struct RawNotification<'a> {
    pub channel: &'a str,
    pub text: Option<&'a str>,
    pub camera_name: Option<&'a str>,
    pub when_ms: Option<u64>,
}

fn classify_by_keyword(text: &str) -> Option<(EventKind, &'static str)> {
    let rules: &[(&str, EventKind)] = &[
        ("Bewegung", EventKind::Motion),
        ("Person", EventKind::Person),
        ("Mensch", EventKind::Person),
        ("Fahrzeug", EventKind::Vehicle),
        ("Auto", EventKind::Vehicle),
        ("Paket", EventKind::Package),
        ("Klingel", EventKind::Doorbell),
    ];
    for (keyword, kind) in rules {
        if text.contains(keyword) {
            return Some((kind.clone(), keyword));
        }
    }
    None
}

fn classify_by_channel(channel: &str) -> Option<EventKind> {
    match channel {
        "com.glazero.android.ring.v2" => Some(EventKind::Doorbell),
        "com.glazero.android.dismantle.v2" => Some(EventKind::Tamper),
        "com.glazero.android.lowpower.v2" => Some(EventKind::LowBattery),
        "com.glazero.android.call.v2" => Some(EventKind::IntercomCall),
        _ => None,
    }
}

const GENERIC_CAMERA_EVENT_PATTERN: &str = "entdeckt";

/// Normalize one raw notification into an `Event`. Never panics on
/// missing/unexpected input; falls back to `EventKind::Other("UnknownVendorEvent")`
/// with the uncertainty recorded in `metadata`, not hidden.
pub fn normalize(raw: &RawNotification, device: DeviceId) -> Event {
    let text = raw.text.unwrap_or("");
    let mut metadata = BTreeMap::new();
    metadata.insert("channel".to_string(), raw.channel.to_string());
    metadata.insert("raw_text".to_string(), raw.text.unwrap_or("").to_string());
    if let Some(cam) = raw.camera_name {
        metadata.insert("camera_name".to_string(), cam.to_string());
    }

    let (kind, confidence, note): (EventKind, &str, String) =
        if let Some((kind, keyword)) = classify_by_keyword(text) {
            (kind, "high", format!("text contains keyword '{keyword}'"))
        } else if text.contains(GENERIC_CAMERA_EVENT_PATTERN) {
            (
                EventKind::Motion,
                "low",
                "generic camera-event phrasing without a specific keyword; \
                 defaulted to Motion as the broadest bucket -- see \
                 docs/event-pipeline.md Korrekturbefund"
                    .to_string(),
            )
        } else if let Some(kind) = classify_by_channel(raw.channel) {
            (
                kind,
                "high",
                "channel display name unambiguously identifies this type".to_string(),
            )
        } else {
            (
                EventKind::Other("UnknownVendorEvent".to_string()),
                "none",
                "no keyword, phrasing, or known channel matched".to_string(),
            )
        };

    metadata.insert("confidence".to_string(), confidence.to_string());
    metadata.insert("classification_note".to_string(), note);

    Event {
        device,
        severity: kind.default_severity(),
        at: raw
            .when_ms
            .map(|ms| Timestamp(ms / 1000))
            .unwrap_or_else(Timestamp::now),
        thumbnail_url: None,
        kind,
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev() -> DeviceId {
        DeviceId("test-device".into())
    }

    // Mirrors tests/test_event_pipeline.py 1:1 -- same fixtures, same
    // expected outcomes, proving parity between the two languages.

    #[test]
    fn real_observed_pattern_maps_to_motion_with_low_confidence() {
        let raw = RawNotification {
            channel: "com.glazero.android.event.v2",
            text: Some("Ereignis vor Kamera 1 entdeckt"),
            camera_name: Some("Kamera 1"),
            when_ms: Some(100_000),
        };
        let event = normalize(&raw, dev());
        assert_eq!(event.kind, EventKind::Motion);
        assert_eq!(event.metadata["confidence"], "low");
        assert!(event.metadata["classification_note"].contains("Korrekturbefund"));
    }

    #[test]
    fn raw_text_is_never_discarded() {
        let raw = RawNotification {
            channel: "com.glazero.android.event.v2",
            text: Some("Ereignis vor X entdeckt"),
            camera_name: None,
            when_ms: None,
        };
        let event = normalize(&raw, dev());
        assert_eq!(event.metadata["raw_text"], "Ereignis vor X entdeckt");
    }

    #[test]
    fn ring_channel_maps_to_doorbell_regardless_of_text() {
        let raw = RawNotification {
            channel: "com.glazero.android.ring.v2",
            text: None,
            camera_name: None,
            when_ms: None,
        };
        assert_eq!(normalize(&raw, dev()).kind, EventKind::Doorbell);
    }

    #[test]
    fn dismantle_channel_maps_to_tamper() {
        let raw = RawNotification {
            channel: "com.glazero.android.dismantle.v2",
            text: None,
            camera_name: None,
            when_ms: None,
        };
        assert_eq!(normalize(&raw, dev()).kind, EventKind::Tamper);
    }

    #[test]
    fn lowpower_channel_maps_to_low_battery() {
        let raw = RawNotification {
            channel: "com.glazero.android.lowpower.v2",
            text: None,
            camera_name: None,
            when_ms: None,
        };
        assert_eq!(normalize(&raw, dev()).kind, EventKind::LowBattery);
    }

    #[test]
    fn call_channel_maps_to_intercom_call() {
        let raw = RawNotification {
            channel: "com.glazero.android.call.v2",
            text: None,
            camera_name: None,
            when_ms: None,
        };
        assert_eq!(normalize(&raw, dev()).kind, EventKind::IntercomCall);
    }

    #[test]
    fn motion_keyword_takes_precedence_over_generic_phrasing() {
        let raw = RawNotification {
            channel: "com.glazero.android.event.v2",
            text: Some("Bewegung erkannt"),
            camera_name: None,
            when_ms: None,
        };
        let event = normalize(&raw, dev());
        assert_eq!(event.kind, EventKind::Motion);
        assert_eq!(event.metadata["confidence"], "high");
    }

    #[test]
    fn person_keyword_maps_to_person() {
        let raw = RawNotification {
            channel: "com.glazero.android.event.v2",
            text: Some("Person im Garten erkannt"),
            camera_name: None,
            when_ms: None,
        };
        assert_eq!(normalize(&raw, dev()).kind, EventKind::Person);
    }

    #[test]
    fn vehicle_keyword_maps_to_vehicle() {
        let raw = RawNotification {
            channel: "com.glazero.android.event.v2",
            text: Some("Fahrzeug erkannt"),
            camera_name: None,
            when_ms: None,
        };
        assert_eq!(normalize(&raw, dev()).kind, EventKind::Vehicle);
    }

    #[test]
    fn package_keyword_maps_to_package() {
        let raw = RawNotification {
            channel: "com.glazero.android.event.v2",
            text: Some("Paket entdeckt"),
            camera_name: None,
            when_ms: None,
        };
        assert_eq!(normalize(&raw, dev()).kind, EventKind::Package);
    }

    #[test]
    fn unrecognized_channel_and_text_becomes_unknown_vendor_event() {
        let raw = RawNotification {
            channel: "com.glazero.android.totally_new.v9",
            text: Some("??"),
            camera_name: None,
            when_ms: None,
        };
        let event = normalize(&raw, dev());
        assert_eq!(
            event.kind,
            EventKind::Other("UnknownVendorEvent".to_string())
        );
        assert_eq!(event.metadata["confidence"], "none");
    }

    #[test]
    fn missing_text_does_not_panic_and_is_unknown() {
        let raw = RawNotification {
            channel: "com.glazero.android.default.v2",
            text: None,
            camera_name: None,
            when_ms: None,
        };
        let event = normalize(&raw, dev());
        assert_eq!(
            event.kind,
            EventKind::Other("UnknownVendorEvent".to_string())
        );
    }
}
