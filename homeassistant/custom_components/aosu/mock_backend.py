"""Pure-Python mock device backend -- deliberately testable without the
`homeassistant` package installed, since it contains no HA imports.
Mirrors `aosu-core`'s MockTransport so UI/entity work can proceed before
any real transport is authenticated.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Optional


@dataclass
class MockDevice:
    id: str
    name: str
    model: str
    kind: str
    online: bool
    battery_percent: Optional[int]
    wifi_signal_percent: Optional[int]


def list_mock_devices() -> list[MockDevice]:
    return [
        MockDevice("mock-cam-1", "Terasse Norden (Demo)", "C7S", "camera", True, 81, 70),
        MockDevice("mock-cam-2", "Garten Westen (Demo)", "C7S", "camera", True, 54, 54),
        MockDevice("mock-base", "Basisstation (Demo)", "H1L", "base_station", True, None, None),
    ]


def cameras(devices: list[MockDevice]) -> list[MockDevice]:
    return [d for d in devices if d.kind == "camera"]
