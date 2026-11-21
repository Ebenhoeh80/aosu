"""Camera entities backed by the Mock transport.

No PTZ entity is created: all observed C7S devices showed no pan/tilt
controls in the official app (see docs/apk-findings.md), so PTZ support
is out of scope until contradicting evidence appears.
"""
from __future__ import annotations

from homeassistant.components.camera import Camera
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .mock_backend import cameras, list_mock_devices


async def async_setup_entry(
    hass: HomeAssistant, entry: ConfigEntry, async_add_entities: AddEntitiesCallback
) -> None:
    devices = cameras(list_mock_devices())
    async_add_entities(AosuMockCamera(d) for d in devices)


class AosuMockCamera(Camera):
    """Reports itself but cannot stream yet (no authenticated transport)."""

    _attr_has_entity_name = True

    def __init__(self, device) -> None:
        super().__init__()
        self._device = device
        self._attr_unique_id = device.id
        self._attr_name = device.name

    @property
    def is_streaming(self) -> bool:
        return False

    async def async_camera_image(self, width=None, height=None):
        return None
