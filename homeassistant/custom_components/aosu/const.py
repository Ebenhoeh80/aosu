"""Constants for the unofficial Aosu integration.

iot_class is "local_polling" because today's only working backend is the
Mock transport plus the notification bridge; no cloud credential handling
happens in this integration (see docs/adr-0002-path-reassessment.md for
why real cloud/local transports are intentionally not implemented yet).
"""

DOMAIN = "aosu"

CONF_MODE = "mode"
MODE_MOCK = "mock"
MODE_NOTIFICATION_BRIDGE = "notification_bridge"

DEFAULT_MODE = MODE_MOCK

PLATFORMS = ["camera", "binary_sensor", "sensor"]
