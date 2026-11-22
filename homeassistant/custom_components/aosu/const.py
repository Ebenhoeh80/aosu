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

# Path to the JSONL file tools/notification_bridge.py --jsonl-out
# writes, read by push_log_backend.py. Only used when CONF_MODE is
# MODE_NOTIFICATION_BRIDGE. See docs/event-pipeline.md.
CONF_PUSH_LOG_PATH = "push_log_path"
DEFAULT_PUSH_LOG_PATH = "aosu-events.jsonl"

PLATFORMS = ["camera", "binary_sensor", "sensor"]
