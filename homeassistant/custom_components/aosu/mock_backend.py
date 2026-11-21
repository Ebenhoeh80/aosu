"""Pure-Python mock device backend -- deliberately testable without the
`homeassistant` package installed, since it contains no HA imports.
Mirrors `aosu-core`'s MockTransport (same six devices, same health
spread) so the HA entities and the desktop app agree on what "demo
data" looks like.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Optional


@dataclass
class MockDevice:
    id: str
    name: str
    vendor: str
    model: str
    kind: str
    online: bool
    health: str
    battery_percent: Optional[int]
    wifi_signal_percent: Optional[int]
    firmware_version: Optional[str]


@dataclass
class MockNotification:
    id: str
    device: Optional[str]
    severity: str
    title: str


def list_mock_devices() -> list[MockDevice]:
    return [
        MockDevice(
            "mock-cam-1", "Terasse Norden (Demo)", "Aosu (Demo)", "C7S", "camera",
            True, "Healthy", 81, 70, "5.11.20-demo",
        ),
        MockDevice(
            "mock-cam-2", "Garten Westen (Demo)", "Aosu (Demo)", "C7S", "camera",
            True, "Degraded", 12, 24, "5.11.20-demo",
        ),
        MockDevice(
            "mock-cam-3", "Einfahrt Süden (Demo, offline)", "Aosu (Demo)", "C7S",
            "camera", False, "Offline", None, None, "5.11.20-demo",
        ),
        MockDevice(
            "mock-cam-4", "Prototyp-Kamera (Demo, nicht unterstützt)", "Aosu (Demo)",
            "X9-Unreleased", "camera", True, "Unsupported", 55, 88, None,
        ),
        MockDevice(
            "mock-cam-5", "Dachkamera (Demo, teilweise unterstützt)", "Aosu (Demo)",
            "C9C", "camera", True, "PartiallySupported", 64, 40, "5.11.20-demo",
        ),
        MockDevice(
            "mock-base", "Basisstation (Demo)", "Aosu (Demo)", "H1L", "base_station",
            True, "Healthy", None, None, "2.4.1-demo",
        ),
    ]


def cameras(devices: list[MockDevice]) -> list[MockDevice]:
    return [d for d in devices if d.kind == "camera"]


def list_mock_notifications() -> list[MockNotification]:
    return [
        MockNotification("mock-cam-2-lowbattery", "mock-cam-2", "Warning", "LowBattery"),
        MockNotification("mock-cam-3-connectivity", "mock-cam-3", "Warning", "Connectivity"),
    ]
