import sys
from pathlib import Path

sys.path.insert(
    0,
    str(
        Path(__file__).resolve().parent.parent
        / "homeassistant"
        / "custom_components"
        / "aosu"
    ),
)
import push_log_backend as plb  # noqa: E402

FIXTURES = Path(__file__).resolve().parent / "fixtures"
REAL_CAPTURE = FIXTURES / "real_capture_anonymized.jsonl"


def test_missing_file_returns_empty_list_not_an_error(tmp_path):
    assert plb.read_events(tmp_path / "does-not-exist.jsonl") == []


def test_reads_the_same_real_captured_fixture_rust_uses():
    events = plb.read_events(REAL_CAPTURE)
    assert len(events) == 1
    assert events[0].event_type == "MotionDetected"
    assert events[0].camera_name == "Kamera 1"
    assert events[0].confidence == "low"


def test_camera_names_lists_distinct_cameras_in_first_seen_order(tmp_path):
    p = tmp_path / "events.jsonl"
    p.write_text(
        '{"event_type": "MotionDetected", "camera_name": "B", "channel": "x", "timestamp_ms": 2, "confidence": "low", "classification_note": null}\n'
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1, "confidence": "low", "classification_note": null}\n'
        '{"event_type": "MotionDetected", "camera_name": "B", "channel": "x", "timestamp_ms": 3, "confidence": "low", "classification_note": null}\n',
        encoding="utf-8",
    )
    events = plb.read_events(p)
    assert plb.camera_names(events) == ["B", "A"]


def test_latest_event_for_camera_picks_the_highest_timestamp(tmp_path):
    p = tmp_path / "events.jsonl"
    p.write_text(
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1, "confidence": "low", "classification_note": null}\n'
        '{"event_type": "TamperDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 99, "confidence": "high", "classification_note": null}\n',
        encoding="utf-8",
    )
    events = plb.read_events(p)
    latest = plb.latest_event_for_camera(events, "A")
    assert latest.event_type == "TamperDetected"


def test_latest_event_returns_none_for_unknown_camera(tmp_path):
    p = tmp_path / "events.jsonl"
    p.write_text("", encoding="utf-8")
    assert plb.latest_event_for_camera(plb.read_events(p), "Nobody") is None


def test_malformed_lines_are_skipped_not_fatal(tmp_path):
    p = tmp_path / "events.jsonl"
    p.write_text(
        "not json at all\n"
        "\n"
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1, "confidence": "low", "classification_note": null}\n',
        encoding="utf-8",
    )
    events = plb.read_events(p)
    assert len(events) == 1


def test_unknown_event_type_remains_visible_not_dropped(tmp_path):
    p = tmp_path / "events.jsonl"
    p.write_text(
        '{"event_type": "UnknownVendorEvent", "camera_name": "A", "channel": "weird.v9", "timestamp_ms": 1, "confidence": "none", "classification_note": null}\n',
        encoding="utf-8",
    )
    events = plb.read_events(p)
    assert len(events) == 1
    assert events[0].event_type == "UnknownVendorEvent"
