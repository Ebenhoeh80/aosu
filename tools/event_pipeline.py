#!/usr/bin/env python3
"""Normalize raw Aosu push notifications (see notification_bridge.py)
into a vendor-agnostic event model usable by Home Assistant, the
desktop app, and anything else -- the single documented mapping table
lives in docs/event-pipeline.md, not scattered across consumers.

Design rules (see docs/event-pipeline.md for the full reasoning):
  - Classify primarily by *text content*, channel only as a fallback --
    a real device's channel display name ("Klingelereignisse") did not
    predict its actual content (generic "Ereignis entdeckt", not a
    doorbell ring). Don't trust channel names as ground truth.
  - Never invent a specific sub-type (human/vehicle/package) without an
    observed keyword backing it.
  - Always preserve the full raw record in `metadata["raw"]` -- nothing
    is ever discarded, and UnknownVendorEvent is a first-class, visible
    outcome, not a dead end.
"""
from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field
from enum import Enum
from typing import Optional

from notification_bridge import BridgeEvent


class NormalizedEventType(str, Enum):
    MOTION_DETECTED = "MotionDetected"
    HUMAN_DETECTED = "HumanDetected"
    VEHICLE_DETECTED = "VehicleDetected"
    PACKAGE_DETECTED = "PackageDetected"
    DOORBELL_PRESSED = "DoorbellPressed"
    TAMPER_DETECTED = "TamperDetected"
    BATTERY_LOW = "BatteryLow"
    INTERCOM_CALL = "IntercomCall"
    DEVICE_OFFLINE = "DeviceOffline"
    DEVICE_ONLINE = "DeviceOnline"
    UNKNOWN_VENDOR_EVENT = "UnknownVendorEvent"


@dataclass
class NormalizedEvent:
    event_type: NormalizedEventType
    camera_name: Optional[str]
    channel: str
    timestamp_ms: Optional[int]
    confidence: str  # "high" | "low" | "none"
    classification_note: Optional[str]
    metadata: dict = field(default_factory=dict)

    def to_json(self) -> str:
        d = asdict(self)
        d["event_type"] = self.event_type.value
        return json.dumps(d, ensure_ascii=False)


# Channels whose own Android display name unambiguously (as read
# directly off a real device, not guessed) determines the event type,
# regardless of message text.
CHANNEL_TO_HIGH_CONFIDENCE_TYPE = {
    "com.glazero.android.ring.v2": NormalizedEventType.DOORBELL_PRESSED,
    "com.glazero.android.dismantle.v2": NormalizedEventType.TAMPER_DETECTED,
    "com.glazero.android.lowpower.v2": NormalizedEventType.BATTERY_LOW,
    "com.glazero.android.call.v2": NormalizedEventType.INTERCOM_CALL,
}

# Keyword -> type, checked against message text first (more reliable
# than channel, see docs/event-pipeline.md). None of these specific
# keywords have been observed yet on a real device; they exist so a
# future real sample is classified correctly the moment it appears,
# without needing another round of guessing.
TEXT_KEYWORD_RULES: list[tuple[str, NormalizedEventType]] = [
    ("Bewegung", NormalizedEventType.MOTION_DETECTED),
    ("Person", NormalizedEventType.HUMAN_DETECTED),
    ("Mensch", NormalizedEventType.HUMAN_DETECTED),
    ("Fahrzeug", NormalizedEventType.VEHICLE_DETECTED),
    ("Auto", NormalizedEventType.VEHICLE_DETECTED),
    ("Paket", NormalizedEventType.PACKAGE_DETECTED),
    ("Klingel", NormalizedEventType.DOORBELL_PRESSED),
]

# The one pattern actually observed on a real device so far: generic,
# does not name a specific sub-type. See docs/event-pipeline.md
# "Wichtiger Korrekturbefund".
GENERIC_CAMERA_EVENT_PATTERN = "entdeckt"


def normalize(event: BridgeEvent) -> NormalizedEvent:
    confidence = "none"
    note: Optional[str] = None
    event_type = NormalizedEventType.UNKNOWN_VENDOR_EVENT

    text = event.text or ""
    for keyword, mapped_type in TEXT_KEYWORD_RULES:
        if keyword in text:
            event_type = mapped_type
            confidence = "high"
            note = f"text contains keyword '{keyword}'"
            break
    else:
        if GENERIC_CAMERA_EVENT_PATTERN in text:
            event_type = NormalizedEventType.MOTION_DETECTED
            confidence = "low"
            note = (
                "generic camera-event phrasing without a specific keyword; "
                "defaulted to MotionDetected as the broadest bucket -- see "
                "docs/event-pipeline.md Korrekturbefund"
            )
        elif event.channel in CHANNEL_TO_HIGH_CONFIDENCE_TYPE:
            event_type = CHANNEL_TO_HIGH_CONFIDENCE_TYPE[event.channel]
            confidence = "high"
            note = "channel display name unambiguously identifies this type"

    return NormalizedEvent(
        event_type=event_type,
        camera_name=event.camera_name,
        channel=event.channel,
        timestamp_ms=event.when_ms,
        confidence=confidence,
        classification_note=note,
        metadata={"raw": asdict(event)},
    )
