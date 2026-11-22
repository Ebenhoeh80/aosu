#!/usr/bin/env python3
"""Bridge Android system notifications from the official Aosu app into
Home-Assistant-friendly events -- no protocol reverse engineering, no
credentials, no device key. Purely: read your own phone's own
notification shade (via `adb shell dumpsys notification`, a standard
Android debugging feature) and turn recognized notifications into
structured events.

Two independently testable halves:
  - `parse_notification_block`: pure text parsing, fully unit-testable
    with fixture strings, no device needed.
  - `poll_once` / `run`: the device-facing I/O loop, calls `adb` and
    forwards parsed events to a configurable sink (default: print as
    JSON lines; can be pointed at a Home Assistant webhook URL).

Known channel IDs observed on a real device (see docs/apk-findings.md):
  com.glazero.android.event.v2      -> motion/other camera events
  com.glazero.android.ring.v2       -> doorbell ring
  com.glazero.android.lowpower.v2   -> low battery
  com.glazero.android.dismantle.v2  -> tamper/dismantle
  com.glazero.android.call.v2       -> intercom call
  com.glazero.android.silence.v2    -> silent/background update (no
                                        user-visible content expected)

Usage:
  python tools/notification_bridge.py --device <adb-serial> --once
  python tools/notification_bridge.py --device <adb-serial> --interval 15
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from typing import Callable, Iterable, Optional

PACKAGE = "com.glazero.android"

CHANNEL_TO_EVENT_KIND = {
    f"{PACKAGE}.event.v2": "motion",
    f"{PACKAGE}.ring.v2": "doorbell",
    f"{PACKAGE}.lowpower.v2": "low_battery",
    f"{PACKAGE}.dismantle.v2": "tamper",
    f"{PACKAGE}.call.v2": "intercom_call",
    f"{PACKAGE}.silence.v2": "silent_update",
    f"{PACKAGE}.default.v2": "other",
}


# Only the "event.v2" / motion phrasing has actually been observed on a
# real device so far ("Ereignis vor <Kamera> entdeckt", de-DE, 2026-11-22,
# see docs/apk-findings.md). Not yet confirmed for other channels or
# locales -- extend this list only after observing the real text, not by
# guessing the pattern for an untested channel.
CAMERA_NAME_PATTERNS = [
    r"^Ereignis vor (.+) entdeckt$",
]


@dataclass
class BridgeEvent:
    channel: str
    kind: str
    title: Optional[str]
    text: Optional[str]
    key: Optional[str]
    when_ms: Optional[int]
    camera_name: Optional[str] = None


def extract_camera_name(text: Optional[str]) -> Optional[str]:
    """Best-effort extraction of the camera name from a notification's
    body text, using only patterns actually observed on a real device.
    Returns None rather than guessing when nothing matches."""
    if not text:
        return None
    for pattern in CAMERA_NAME_PATTERNS:
        m = re.match(pattern, text)
        if m:
            return m.group(1)
    return None


def parse_notification_block(block: str) -> Optional[BridgeEvent]:
    """Parse one `NotificationRecord(...)` block's text. Returns None if
    the block isn't from our package."""
    if f"pkg={PACKAGE}" not in block:
        return None
    channel = _match(block, r"channel=([^\s]+)")
    title = _match_extra(block, "android.title")
    text = _match_extra(block, "android.text")
    key = _match(block, r"key=([^:\s]+)")
    when_raw = _match(block, r"when=(\d+)")
    kind = CHANNEL_TO_EVENT_KIND.get(channel or "", "unknown")
    return BridgeEvent(
        channel=channel or "unknown",
        kind=kind,
        title=title,
        text=text,
        key=key,
        when_ms=int(when_raw) if when_raw else None,
        camera_name=extract_camera_name(text),
    )


def _match(text: str, pattern: str) -> Optional[str]:
    m = re.search(pattern, text)
    return m.group(1) if m else None


def _match_extra(block: str, field: str) -> Optional[str]:
    """Parse one `dumpsys notification` extras line. Real devices use at
    least two formats for the same field:
      android.title=null                        -> genuinely absent
      android.text=String (Ereignis ... entdeckt) -> wrapped, trailing ')'
    """
    m = re.search(rf"{re.escape(field)}=(.+)", block)
    if not m:
        return None
    raw = m.group(1).rstrip("\r\n")
    if raw == "null":
        return None
    wrapped = re.match(r"String \((.*)\)$", raw)
    return wrapped.group(1) if wrapped else raw


def parse_dumpsys_output(output: str) -> list[BridgeEvent]:
    blocks = output.split("NotificationRecord(")[1:]
    events = []
    for block in blocks:
        event = parse_notification_block(block)
        if event:
            events.append(event)
    return events


def fetch_dumpsys(device: str) -> str:
    result = subprocess.run(
        ["adb", "-s", device, "shell", "dumpsys", "notification", "--noredact"],
        capture_output=True,
        text=True,
        timeout=15,
    )
    return result.stdout


def poll_once(device: str, seen_keys: set[str]) -> list[BridgeEvent]:
    events = parse_dumpsys_output(fetch_dumpsys(device))
    fresh = []
    for e in events:
        dedup_key = f"{e.key}|{e.when_ms}"
        if dedup_key in seen_keys:
            continue
        seen_keys.add(dedup_key)
        fresh.append(e)
    return fresh


def run(
    device: str,
    interval_seconds: float,
    sink: Callable[[BridgeEvent], None],
    max_iterations: Optional[int] = None,
) -> None:
    seen_keys: set[str] = set()
    i = 0
    while max_iterations is None or i < max_iterations:
        for event in poll_once(device, seen_keys):
            sink(event)
        i += 1
        if max_iterations is None or i < max_iterations:
            time.sleep(interval_seconds)


def print_sink(event: BridgeEvent) -> None:
    print(json.dumps(asdict(event)))


def make_jsonl_sink(path: str) -> Callable[[BridgeEvent], None]:
    """Appends one normalized event (see event_pipeline.py) per line --
    the shared integration point for Home Assistant and the desktop app,
    see docs/event-pipeline.md. Import is local to avoid a hard
    dependency for callers that only want raw BridgeEvents.
    """
    from event_pipeline import normalize

    def sink(event: BridgeEvent) -> None:
        normalized = normalize(event)
        with open(path, "a", encoding="utf-8") as f:
            f.write(normalized.to_json() + "\n")

    return sink


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--device", required=True, help="adb device serial")
    ap.add_argument("--interval", type=float, default=15.0)
    ap.add_argument("--once", action="store_true", help="poll a single time and exit")
    ap.add_argument(
        "--jsonl-out",
        help="append normalized events (event_pipeline.py) to this file, "
        "one JSON object per line -- read by Home Assistant and/or the "
        "desktop app's push-log transport (see docs/event-pipeline.md)",
    )
    args = ap.parse_args()
    sink = make_jsonl_sink(args.jsonl_out) if args.jsonl_out else print_sink
    run(
        args.device,
        args.interval,
        sink,
        max_iterations=1 if args.once else None,
    )


if __name__ == "__main__":
    main()
