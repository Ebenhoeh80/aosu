"""Config flow: offers Mock mode (installable/demoable without any real
data) and Notification-Bridge mode (reads the JSONL file
tools/notification_bridge.py --jsonl-out produces -- see
docs/event-pipeline.md). See docs/architecture.md section 5.
"""
from __future__ import annotations

import voluptuous as vol
from homeassistant import config_entries

from .const import (
    CONF_MODE,
    CONF_PUSH_LOG_PATH,
    DEFAULT_MODE,
    DEFAULT_PUSH_LOG_PATH,
    DOMAIN,
    MODE_MOCK,
    MODE_NOTIFICATION_BRIDGE,
)


class AosuConfigFlow(config_entries.ConfigFlow, domain=DOMAIN):
    """Handle a config flow for the unofficial Aosu integration."""

    VERSION = 1

    async def async_step_user(self, user_input=None):
        errors: dict[str, str] = {}
        if user_input is not None:
            if user_input[CONF_MODE] == MODE_NOTIFICATION_BRIDGE:
                return await self.async_step_push_log(user_input)
            return self.async_create_entry(title="Aosu (unoffiziell)", data=user_input)

        schema = vol.Schema(
            {
                vol.Required(CONF_MODE, default=DEFAULT_MODE): vol.In(
                    [MODE_MOCK, MODE_NOTIFICATION_BRIDGE]
                ),
            }
        )
        return self.async_show_form(step_id="user", data_schema=schema, errors=errors)

    async def async_step_push_log(self, user_input=None):
        """Second step for MODE_NOTIFICATION_BRIDGE: where is the JSONL
        file? See docs/event-pipeline.md -- this file is written by
        tools/notification_bridge.py wherever adb/the phone is
        reachable, which is typically not the Home Assistant host
        itself, so the path must point at a location HA can read (e.g.
        a network share or a path inside HA's config directory if the
        bridge writes there directly)."""
        if user_input is not None and CONF_PUSH_LOG_PATH in user_input:
            data = {
                CONF_MODE: MODE_NOTIFICATION_BRIDGE,
                CONF_PUSH_LOG_PATH: user_input[CONF_PUSH_LOG_PATH],
            }
            return self.async_create_entry(title="Aosu (unoffiziell)", data=data)

        schema = vol.Schema({vol.Required(CONF_PUSH_LOG_PATH, default=DEFAULT_PUSH_LOG_PATH): str})
        return self.async_show_form(step_id="push_log", data_schema=schema)
