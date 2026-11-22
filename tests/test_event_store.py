import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "tools"))
from event_store import EventQuery, EventStore  # noqa: E402

FIXTURE = Path(__file__).resolve().parent / "fixtures" / "real_capture_anonymized.jsonl"


def test_ingests_the_real_anonymized_fixture():
    store = EventStore.open_in_memory()
    report = store.ingest_jsonl_file(FIXTURE)
    assert report.inserted >= 1
    assert report.malformed == 0
    assert store.count_total() == report.inserted


def test_re_ingesting_the_same_content_is_idempotent():
    store = EventStore.open_in_memory()
    first = store.ingest_jsonl_file(FIXTURE)
    second = store.ingest_jsonl_file(FIXTURE)
    assert second.inserted == 0
    assert second.duplicates == first.inserted


def test_malformed_lines_are_counted_not_fatal():
    store = EventStore.open_in_memory()
    report = store.ingest_jsonl_str("not json\n\n" + FIXTURE.read_text(encoding="utf-8"))
    assert report.malformed == 1
    assert "not json" not in "".join(report.malformed_samples)  # it's the parser's error text
    assert len(report.malformed_samples) == 1


def test_query_filters_by_camera_and_supports_pagination():
    store = EventStore.open_in_memory()
    store.ingest_jsonl_str(
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1000, "confidence": "low"}\n'
        '{"event_type": "TamperDetected", "camera_name": "B", "channel": "x", "timestamp_ms": 2000, "confidence": "high"}\n'
    )
    rows = store.query(EventQuery(camera_name="B", limit=10, offset=0))
    assert len(rows) == 1
    assert rows[0].event_type == "TamperDetected"

    page1 = store.query(EventQuery(limit=1, offset=0))
    page2 = store.query(EventQuery(limit=1, offset=1))
    assert page1[0].id != page2[0].id


def test_device_activity_aggregates_per_camera():
    store = EventStore.open_in_memory()
    store.ingest_jsonl_str(
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1000, "confidence": "low"}\n'
        '{"event_type": "TamperDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 2000, "confidence": "high"}\n'
    )
    activity = store.device_activity()
    assert len(activity) == 1
    assert activity[0].camera_name == "A"
    assert activity[0].event_count == 2
    assert activity[0].last_seen_ms == 2000
    assert set(activity[0].distinct_event_types) == {"MotionDetected", "TamperDetected"}


def test_analytics_counts_events_today_and_by_type():
    store = EventStore.open_in_memory()
    now_ms = 1_700_000_000_000
    today_start = (now_ms // 86_400_000) * 86_400_000
    store.ingest_jsonl_str(
        f'{{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": {today_start + 1000}, "confidence": "low"}}\n'
    )
    analytics = store.analytics(now_ms)
    assert analytics.events_today == 1
    assert analytics.by_type == [("MotionDetected", 1)]
    assert len(analytics.daily_counts_last_14_days) == 14


def test_retention_by_age_deletes_old_rows():
    store = EventStore.open_in_memory()
    store.ingest_jsonl_str(
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 0, "confidence": "low"}\n'
    )
    deleted = store.apply_retention(now_ms=999_999_999_999, max_age_days=1)
    assert deleted == 1
    assert store.count_total() == 0


def test_retention_by_row_count_keeps_newest():
    store = EventStore.open_in_memory()
    store.ingest_jsonl_str(
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1, "confidence": "low"}\n'
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 2, "confidence": "low"}\n'
    )
    deleted = store.apply_retention(now_ms=0, max_rows=1)
    assert deleted == 1
    remaining = store.query(EventQuery(limit=10, offset=0))
    assert len(remaining) == 1
    assert remaining[0].timestamp_ms == 2


def test_export_json_and_csv_round_trip():
    store = EventStore.open_in_memory()
    store.ingest_jsonl_str(
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1, "confidence": "low"}\n'
    )
    assert "MotionDetected" in store.export_json(EventQuery(limit=10, offset=0))
    csv_out = store.export_csv(EventQuery(limit=10, offset=0))
    assert "MotionDetected" in csv_out
    assert csv_out.splitlines()[0].startswith("id,timestamp_ms")


def test_schema_version_persists_across_reopen(tmp_path):
    db_path = tmp_path / "events.sqlite"
    store = EventStore(db_path)
    store.ingest_jsonl_str(
        '{"event_type": "MotionDetected", "camera_name": "A", "channel": "x", "timestamp_ms": 1, "confidence": "low"}\n'
    )
    store.close()

    reopened = EventStore(db_path)
    assert reopened.count_total() == 1
