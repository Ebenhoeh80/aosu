"""Config flow: today only offers the Mock mode, so the integration is
installable and demoable in a real Home Assistant instance before any
real transport is authenticated. See docs/architecture.md section 5.
"""
from __future__ import annotations

import voluptuous as vol
from homeassistant import config_entries

from .const import CONF_MODE, DEFAULT_MODE, DOMAIN, MODE_MOCK, MODE_NOTIFICATION_BRIDGE


class AosuConfigFlow(config_entries.ConfigFlow, domain=DOMAIN):
    """Handle a config flow for the unofficial Aosu integration."""

    VERSION = 1

    async def async_step_user(self, user_input=None):
        errors: dict[str, str] = {}
        if user_input is not None:
            return self.async_create_entry(
                title="Aosu (unoffiziell)", data=user_input
            )

        schema = vol.Schema(
            {
                vol.Required(CONF_MODE, default=DEFAULT_MODE): vol.In(
                    [MODE_MOCK, MODE_NOTIFICATION_BRIDGE]
                ),
            }
        )
        return self.async_show_form(
            step_id="user", data_schema=schema, errors=errors
        )
