"""Motion/event binary sensors. Intended to be fed by the notification
bridge (tools/notification_bridge.py) once event content is confirmed
end-to-end; currently reports a static demo state via the Mock backend.
"""
from __future__ import annotations

from homeassistant.components.binary_sensor import (
    BinarySensorDeviceClass,
    BinarySensorEntity,
)
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .mock_backend import cameras, list_mock_devices


async def async_setup_entry(
    hass: HomeAssistant, entry: ConfigEntry, async_add_entities: AddEntitiesCallback
) -> None:
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
