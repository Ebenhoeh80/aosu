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
import mock_backend as mb  # noqa: E402


def test_lists_six_demo_devices_across_all_health_states():
    devices = mb.list_mock_devices()
    assert len(devices) == 6
    states = {d.health for d in devices}
    assert states == {
        "Healthy",
        "Degraded",
        "Offline",
        "Unsupported",
        "PartiallySupported",
    }


def test_cameras_filters_to_camera_kind_only():
    devices = mb.list_mock_devices()
    cams = mb.cameras(devices)
    assert len(cams) == 5
    assert all(d.kind == "camera" for d in cams)


def test_base_station_has_no_battery_or_wifi_fields():
    devices = mb.list_mock_devices()
    base = next(d for d in devices if d.kind == "base_station")
    assert base.battery_percent is None
    assert base.wifi_signal_percent is None


def test_unsupported_device_has_no_firmware_version():
    devices = mb.list_mock_devices()
    unsupported = next(d for d in devices if d.health == "Unsupported")
    assert unsupported.firmware_version is None


def test_mock_notifications_only_cover_warning_or_above_devices():
    notifications = mb.list_mock_notifications()
    assert len(notifications) == 2
    assert all(n.severity == "Warning" for n in notifications)
