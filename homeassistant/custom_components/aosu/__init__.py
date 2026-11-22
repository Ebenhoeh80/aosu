"""Unofficial Aosu integration.

Status: full entity model plus a real notification-bridge path. The default
backend is mock (see mock_backend.py), so the entity model is installable
and testable without hardware. In notification-bridge mode, binary_sensor.py
creates real motion/tamper sensors from the JSONL push log. Real local (H1L)
and cloud transports remain unimplemented -- see
docs/adr-0002-path-reassessment.md for the open blockers.
"""
from __future__ import annotations

from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant

from .const import DOMAIN, PLATFORMS


async def async_setup_entry(hass: HomeAssistant, entry: ConfigEntry) -> bool:
    hass.data.setdefault(DOMAIN, {})
    hass.data[DOMAIN][entry.entry_id] = entry.data
    await hass.config_entries.async_forward_entry_setups(entry, PLATFORMS)
    return True


async def async_unload_entry(hass: HomeAssistant, entry: ConfigEntry) -> bool:
    unloaded = await hass.config_entries.async_unload_platforms(entry, PLATFORMS)
    if unloaded:
        hass.data[DOMAIN].pop(entry.entry_id, None)
    return unloaded
