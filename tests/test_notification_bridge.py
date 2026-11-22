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

# Structurally identical to a real device's `dumpsys notification` output
# observed during this project's own testing (single-line
# `key=...: Notification(...)` format, literal `android.title=null`) --
# the camera name below is a placeholder, not the real one, to avoid
# publishing the tester's property layout in a public test fixture.
REAL_DEVICE_BLOCK = """0x0b64f3db: pkg=com.glazero.android user=UserHandle{0} id=3 tag=null importance=4 key=0|com.glazero.android|3|null|10319: Notification(channel=com.glazero.android.event.v2 shortcut=null contentView=null vibrate=null sound=null defaults=0 flags=AUTO_CANCEL color=0xff2a8bd2 actions=2 vis=PRIVATE semFlags=0x2008 semPriority=0 semMissedCount=0))
      uid=10319 userId=0
      opPkg=com.glazero.android
                android.title=null
                android.text=String (Ereignis vor Kamera 1 entdeckt)
                android.bigText=String (Ereignis vor Kamera 1 entdeckt)
  when=1795330875651
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


def test_real_device_block_strips_trailing_paren_from_wrapped_text():
    event = nb.parse_notification_block(REAL_DEVICE_BLOCK)
    assert event is not None
    assert event.text == "Ereignis vor Kamera 1 entdeckt"
    assert not event.text.endswith(")")


def test_real_device_block_treats_literal_null_title_as_none():
    event = nb.parse_notification_block(REAL_DEVICE_BLOCK)
    assert event.title is None


def test_real_device_block_key_stops_before_the_trailing_notification_dump():
    event = nb.parse_notification_block(REAL_DEVICE_BLOCK)
    assert event.key == "0|com.glazero.android|3|null|10319"


def test_real_device_block_identifies_the_camera_named_in_the_text():
    event = nb.parse_notification_block(REAL_DEVICE_BLOCK)
    assert "Kamera 1" in event.text


def test_real_device_block_extracts_camera_name_into_its_own_field():
    event = nb.parse_notification_block(REAL_DEVICE_BLOCK)
    assert event.camera_name == "Kamera 1"


def test_extract_camera_name_returns_none_for_unrecognized_phrasing():
    assert nb.extract_camera_name("Some other message entirely") is None


def test_extract_camera_name_returns_none_for_missing_text():
    assert nb.extract_camera_name(None) is None


def test_run_calls_sink_for_each_fresh_event_and_stops_after_max_iterations():
    combined = "NotificationRecord(" + MOTION_BLOCK
    nb.fetch_dumpsys = lambda _device: combined
    collected = []
    nb.run("fake-device", interval_seconds=0, sink=collected.append, max_iterations=1)
    assert len(collected) == 1
    assert collected[0].kind == "motion"
