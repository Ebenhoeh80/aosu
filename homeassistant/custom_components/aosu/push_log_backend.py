"""Reads the normalized JSONL file `tools/notification_bridge.py
--jsonl-out` produces (see docs/event-pipeline.md). Pure Python, no
`homeassistant` import, so it's testable without that package
installed -- mirrors the Rust `NotificationLogTransport`
(crates/aosu-core/src/transports/notification_log.rs): same file
format, same classification already done upstream by event_pipeline.py,
this module only reads and exposes it.

This, not an in-HA adb call, is the realistic integration point: adb
access typically lives on whatever machine has the phone attached, not
on the Home Assistant host itself.
"""
from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class PushLogEvent:
    event_type: str
    camera_name: Optional[str]
    channel: str
    timestamp_ms: Optional[int]
    confidence: str
    classification_note: Optional[str]


def read_events(path: str | Path) -> list[PushLogEvent]:
    """Returns every parseable line. Malformed lines are skipped, never
    fatal -- see docs/event-pipeline.md "nie verworfene Rohdaten": the
    *content* is never silently dropped, but a line that isn't even
    valid JSON genuinely can't be interpreted and is reported as
    skipped, not crashed on."""
    p = Path(path)
    if not p.is_file():
        return []
    events: list[PushLogEvent] = []
    for line in p.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            data = json.loads(line)
        except json.JSONDecodeError:
            continue
        events.append(
            PushLogEvent(
                event_type=data.get("event_type", "UnknownVendorEvent"),
                camera_name=data.get("camera_name"),
                channel=data.get("channel", ""),
                timestamp_ms=data.get("timestamp_ms"),
                confidence=data.get("confidence", "none"),
                classification_note=data.get("classification_note"),
            )
        )
    return events


def camera_names(events: list[PushLogEvent]) -> list[str]:
    """Distinct camera names seen, in first-seen order -- used to build
    one binary_sensor per camera without needing a separate device
    registry (this backend genuinely only knows what it has seen)."""
    seen: list[str] = []
    for e in events:
        if e.camera_name and e.camera_name not in seen:
            seen.append(e.camera_name)
    return seen


def latest_event_for_camera(
    events: list[PushLogEvent], camera_name: str
) -> Optional[PushLogEvent]:
    matching = [e for e in events if e.camera_name == camera_name]
    if not matching:
        return None
    return max(matching, key=lambda e: e.timestamp_ms or 0)
