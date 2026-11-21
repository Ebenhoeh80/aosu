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


def test_lists_three_demo_devices():
    assert len(mb.list_mock_devices()) == 3


def test_cameras_filters_to_camera_kind_only():
    devices = mb.list_mock_devices()
    cams = mb.cameras(devices)
    assert len(cams) == 2
    assert all(d.kind == "camera" for d in cams)


def test_base_station_has_no_battery_or_wifi_fields():
    devices = mb.list_mock_devices()
    base = next(d for d in devices if d.kind == "base_station")
    assert base.battery_percent is None
    assert base.wifi_signal_percent is None
