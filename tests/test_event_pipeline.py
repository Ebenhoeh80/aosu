import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "tools"))
from notification_bridge import BridgeEvent  # noqa: E402
from event_pipeline import normalize, NormalizedEventType  # noqa: E402


def make_event(channel, text=None, title=None, camera_name=None, when_ms=100, key="k"):
    return BridgeEvent(
        channel=channel,
        kind="irrelevant-legacy-field",
        title=title,
        text=text,
        key=key,
        when_ms=when_ms,
        camera_name=camera_name,
    )


# --- Real-world-shaped fixture (anonymized camera name, see
# docs/event-pipeline.md) -----------------------------------------------


def test_real_observed_pattern_maps_to_motion_with_low_confidence():
    event = make_event(
        "com.glazero.android.event.v2",
        text="Ereignis vor Kamera 1 entdeckt",
        camera_name="Kamera 1",
    )
    result = normalize(event)
    assert result.event_type == NormalizedEventType.MOTION_DETECTED
    assert result.confidence == "low"
    assert result.camera_name == "Kamera 1"
    assert "Korrekturbefund" in (result.classification_note or "")


def test_raw_payload_is_never_discarded():
    event = make_event("com.glazero.android.event.v2", text="Ereignis vor X entdeckt")
    result = normalize(event)
    assert result.metadata["raw"]["text"] == "Ereignis vor X entdeckt"
    assert result.metadata["raw"]["channel"] == "com.glazero.android.event.v2"


# --- Channel-based high-confidence rules (from real channel names) -----


def test_ring_channel_maps_to_doorbell_pressed_regardless_of_text():
    event = make_event("com.glazero.android.ring.v2", text=None)
    result = normalize(event)
    assert result.event_type == NormalizedEventType.DOORBELL_PRESSED
    assert result.confidence == "high"


def test_dismantle_channel_maps_to_tamper_detected():
    result = normalize(make_event("com.glazero.android.dismantle.v2"))
    assert result.event_type == NormalizedEventType.TAMPER_DETECTED


def test_lowpower_channel_maps_to_battery_low():
    result = normalize(make_event("com.glazero.android.lowpower.v2"))
    assert result.event_type == NormalizedEventType.BATTERY_LOW


def test_call_channel_maps_to_intercom_call():
    result = normalize(make_event("com.glazero.android.call.v2"))
    assert result.event_type == NormalizedEventType.INTERCOM_CALL


# --- Keyword-based rules (not yet observed for real, but must work the
# instant a real sample with this wording appears) -----------------------


def test_motion_keyword_takes_precedence_over_generic_phrasing():
    result = normalize(make_event("com.glazero.android.event.v2", text="Bewegung erkannt"))
    assert result.event_type == NormalizedEventType.MOTION_DETECTED
    assert result.confidence == "high"


def test_person_keyword_maps_to_human_detected():
    result = normalize(make_event("com.glazero.android.event.v2", text="Person im Garten erkannt"))
    assert result.event_type == NormalizedEventType.HUMAN_DETECTED


def test_vehicle_keyword_maps_to_vehicle_detected():
    result = normalize(make_event("com.glazero.android.event.v2", text="Fahrzeug erkannt"))
    assert result.event_type == NormalizedEventType.VEHICLE_DETECTED


def test_package_keyword_maps_to_package_detected():
    result = normalize(make_event("com.glazero.android.event.v2", text="Paket entdeckt"))
    assert result.event_type == NormalizedEventType.PACKAGE_DETECTED


def test_doorbell_keyword_on_event_channel_still_maps_to_doorbell():
    result = normalize(make_event("com.glazero.android.event.v2", text="Klingel gedrückt"))
    assert result.event_type == NormalizedEventType.DOORBELL_PRESSED


# --- Unknown / malformed payloads: must remain visible, never raise ----


def test_unrecognized_channel_and_text_becomes_unknown_vendor_event():
    result = normalize(make_event("com.glazero.android.totally_new.v9", text="??"))
    assert result.event_type == NormalizedEventType.UNKNOWN_VENDOR_EVENT
    assert result.confidence == "none"


def test_missing_text_does_not_crash_and_is_unknown():
    result = normalize(make_event("com.glazero.android.default.v2", text=None))
    assert result.event_type == NormalizedEventType.UNKNOWN_VENDOR_EVENT
    assert result.metadata["raw"]["text"] is None


def test_empty_string_text_does_not_crash():
    result = normalize(make_event("com.glazero.android.event.v2", text=""))
    assert result.event_type == NormalizedEventType.UNKNOWN_VENDOR_EVENT


def test_silence_channel_with_no_text_is_unknown_not_a_crash():
    result = normalize(make_event("com.glazero.android.silence.v2", text=None, when_ms=None))
    assert result.event_type == NormalizedEventType.UNKNOWN_VENDOR_EVENT
    assert result.timestamp_ms is None


def test_to_json_round_trips_the_event_type_as_a_plain_string():
    import json

    result = normalize(make_event("com.glazero.android.ring.v2"))
    parsed = json.loads(result.to_json())
    assert parsed["event_type"] == "DoorbellPressed"
    assert isinstance(parsed["event_type"], str)
