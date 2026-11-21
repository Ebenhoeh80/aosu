"""Battery and WiFi-signal sensors, backed by the Mock transport."""
from __future__ import annotations

from homeassistant.components.sensor import SensorDeviceClass, SensorEntity
from homeassistant.config_entries import ConfigEntry
from homeassistant.const import PERCENTAGE
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .mock_backend import cameras, list_mock_devices


async def async_setup_entry(
    hass: HomeAssistant, entry: ConfigEntry, async_add_entities: AddEntitiesCallback
) -> None:
    devices = cameras(list_mock_devices())
    entities = []
    for d in devices:
        entities.append(AosuBatterySensor(d))
        entities.append(AosuWifiSensor(d))
    async_add_entities(entities)


class AosuBatterySensor(SensorEntity):
    _attr_has_entity_name = True
    _attr_device_class = SensorDeviceClass.BATTERY
    _attr_native_unit_of_measurement = PERCENTAGE

    def __init__(self, device) -> None:
        self._attr_unique_id = f"{device.id}-battery"
        self._attr_name = f"{device.name} Akku"
        self._attr_native_value = device.battery_percent


class AosuWifiSensor(SensorEntity):
    _attr_has_entity_name = True
    _attr_native_unit_of_measurement = PERCENTAGE

    def __init__(self, device) -> None:
        self._attr_unique_id = f"{device.id}-wifi"
        self._attr_name = f"{device.name} WLAN-Signal"
        self._attr_native_value = device.wifi_signal_percent
