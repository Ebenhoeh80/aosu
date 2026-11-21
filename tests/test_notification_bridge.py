import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "tools"))
import notification_bridge as nb  # noqa: E402


MOTION_BLOCK = """0x1: pkg=com.glazero.android user=UserHandle{0} id=1 tag=null key=0|com.glazero.android|1|null|10319
  channel=com.glazero.android.event.v2 importance=4
  android.title=String (Bewegung erkannt)
  android.text=String (Terasse Norden: Bewegung erkannt um 15:03)
  when=1732208580000
"""

OTHER_APP_BLOCK = """0x2: pkg=com.android.vending user=UserHandle{0} id=2 tag=null key=1|com.android.vending|2|null|10278
  channel=updates importance=1
  android.title=String (Update verfügbar)
"""

SILENT_BLOCK = """0x3: pkg=com.glazero.android user=UserHandle{0} id=3 tag=null key=2|com.glazero.android|3|null|10319
  channel=com.glazero.android.silence.v2 importance=4
  when=1732208590000
"""


def test_parses_motion_notification_with_channel_mapping():
    event = nb.parse_notification_block(MOTION_BLOCK)
    assert event is not None
    assert event.kind == "motion"
    assert "Terasse Norden" in (event.text or "")
    assert event.when_ms == 1732208580000


def test_ignores_other_packages():
    assert nb.parse_notification_block(OTHER_APP_BLOCK) is None


def test_unknown_channel_maps_to_unknown_not_crash():
    block = MOTION_BLOCK.replace("event.v2", "totally_new_channel.v3")
    event = nb.parse_notification_block(block)
    assert event is not None
    assert event.kind == "unknown"


def test_silent_channel_recognized_even_without_visible_text():
    event = nb.parse_notification_block(SILENT_BLOCK)
    assert event is not None
    assert event.kind == "silent_update"
    assert event.title is None and event.text is None


def test_parse_dumpsys_output_extracts_only_our_package():
    combined = "NotificationRecord(" + MOTION_BLOCK + "NotificationRecord(" + OTHER_APP_BLOCK
    events = nb.parse_dumpsys_output(combined)
    assert len(events) == 1
    assert events[0].kind == "motion"


def test_poll_once_deduplicates_by_key_and_timestamp():
    combined = "NotificationRecord(" + MOTION_BLOCK

    def fake_fetch(_device):
        return combined

    orig = nb.fetch_dumpsys
    nb.fetch_dumpsys = fake_fetch
    try:
        seen: set[str] = set()
        first = nb.poll_once("fake-device", seen)
        second = nb.poll_once("fake-device", seen)
        assert len(first) == 1
        assert len(second) == 0  # already seen
    finally:
        nb.fetch_dumpsys = orig


def test_run_calls_sink_for_each_fresh_event_and_stops_after_max_iterations():
    combined = "NotificationRecord(" + MOTION_BLOCK
    nb.fetch_dumpsys = lambda _device: combined
    collected = []
    nb.run("fake-device", interval_seconds=0, sink=collected.append, max_iterations=1)
    assert len(collected) == 1
    assert collected[0].kind == "motion"
