"""Motion/tamper binary sensors.

Two modes (see const.py / config_flow.py):
  - MODE_MOCK: static demo entities, see mock_backend.py.
  - MODE_NOTIFICATION_BRIDGE: real entities, one per camera name seen in
    the JSONL file tools/notification_bridge.py --jsonl-out writes
    (see docs/event-pipeline.md, push_log_backend.py). This is the
    "first working Home Assistant path using the normalized event
    model" -- real push-notification data, not a mock.

Unknown event types (UnknownVendorEvent) do not create their own
entity; they still appear in the shared "last_event_type" attribute so
they stay visible, per docs/event-pipeline.md "nie verworfene
Rohdaten" -- not every observed event type warrants its own sensor,
but none are silently thrown away.
"""
from __future__ import annotations

from homeassistant.components.binary_sensor import (
    BinarySensorDeviceClass,
    BinarySensorEntity,
)
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .const import CONF_MODE, CONF_PUSH_LOG_PATH, DEFAULT_PUSH_LOG_PATH, MODE_NOTIFICATION_BRIDGE
from .mock_backend import cameras, list_mock_devices
from .push_log_backend import camera_names, latest_event_for_camera, read_events

# Event types considered "tamper-class" for the dedicated tamper sensor,
# vs. everything else surfacing only via the motion sensor's attributes.
# See docs/event-pipeline.md for the full vocabulary.
TAMPER_EVENT_TYPES = {"TamperDetected"}


async def async_setup_entry(
    hass: HomeAssistant, entry: ConfigEntry, async_add_entities: AddEntitiesCallback
) -> None:
    if entry.data.get(CONF_MODE) == MODE_NOTIFICATION_BRIDGE:
        path = entry.data.get(CONF_PUSH_LOG_PATH, DEFAULT_PUSH_LOG_PATH)
        events = read_events(path)
        entities = []
        for name in camera_names(events):
            entities.append(AosuPushLogMotionSensor(name, path))
            entities.append(AosuPushLogTamperSensor(name, path))
        async_add_entities(entities)
        return

    devices = cameras(list_mock_devices())
    async_add_entities(AosuMotionSensor(d) for d in devices)


class AosuMotionSensor(BinarySensorEntity):
    _attr_has_entity_name = True
    _attr_device_class = BinarySensorDeviceClass.MOTION

    def __init__(self, device) -> None:
        self._device = device
        self._attr_unique_id = f"{device.id}-motion"
        self._attr_name = f"{device.name} Bewegung"
        self._attr_is_on = False


class _PushLogSensorBase(BinarySensorEntity):
    """Polls the JSONL file on each HA update cycle. No push mechanism
    into HA exists yet (this reads a file, it doesn't tail it) -- see
    docs/ROADMAP.md for the known gap. `should_poll` defaults to True,
    which is correct here: HA is expected to call update() repeatedly
    for this file-backed entity."""

    def __init__(self, camera_name: str, path: str) -> None:
        self._camera_name = camera_name
        self._path = path
        self._attr_has_entity_name = True
        self._attr_is_on = False
        self._attr_extra_state_attributes: dict = {}

    def update(self) -> None:
        events = read_events(self._path)
        latest = latest_event_for_camera(events, self._camera_name)
        self._attr_extra_state_attributes = {
            "last_event_type": latest.event_type if latest else None,
            "last_confidence": latest.confidence if latest else None,
            "last_channel": latest.channel if latest else None,
        }
        self._attr_is_on = self._is_on_for(latest)

    def _is_on_for(self, latest) -> bool:  # pragma: no cover - overridden
        raise NotImplementedError


class AosuPushLogMotionSensor(_PushLogSensorBase):
    _attr_device_class = BinarySensorDeviceClass.MOTION

    def __init__(self, camera_name: str, path: str) -> None:
        super().__init__(camera_name, path)
        self._attr_unique_id = f"push-log-{camera_name}-motion"
        self._attr_name = f"{camera_name} Bewegung"

    def _is_on_for(self, latest) -> bool:
        return bool(latest) and latest.event_type not in TAMPER_EVENT_TYPES


class AosuPushLogTamperSensor(_PushLogSensorBase):
    _attr_device_class = BinarySensorDeviceClass.TAMPER

    def __init__(self, camera_name: str, path: str) -> None:
        super().__init__(camera_name, path)
        self._attr_unique_id = f"push-log-{camera_name}-tamper"
        self._attr_name = f"{camera_name} Manipulation"

    def _is_on_for(self, latest) -> bool:
        return bool(latest) and latest.event_type in TAMPER_EVENT_TYPES
