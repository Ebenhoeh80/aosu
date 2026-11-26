//! A fully working, fake transport covering a realistic spread of device
//! health states -- not just "everything is fine". Lets the whole UI and
//! the Home Assistant bridge be built, demoed and tested before any real
//! protocol question is resolved, and without any real hardware ever.

use crate::capability::{Capability, CapabilityReport, Support};
use crate::device::*;
use crate::transport::{Result, Transport, TransportError};
use std::sync::Mutex;

pub struct MockTransport {
    devices: Vec<Device>,
    events: Mutex<Vec<Event>>,
    /// Lets tests (and a future "simulate a network failure" settings
    /// toggle) exercise the error path without needing real hardware.
    pub simulate_network_failure: bool,
}

impl Default for MockTransport {
    fn default() -> Self {
        let devices = vec![
            Device {
                id: DeviceId("mock-cam-1".into()),
                name: "Terasse Norden (Demo)".into(),
                vendor: "Aosu (Demo)".into(),
                model: "C7S".into(),
                kind: "camera".into(),
                online: true,
                health: HealthStatus::Healthy,
                battery_percent: Some(81),
                wifi_signal_percent: Some(70),
                firmware_version: Some("5.11.20-demo".into()),
            },
            Device {
                id: DeviceId("mock-cam-2".into()),
                name: "Garten Westen (Demo)".into(),
                vendor: "Aosu (Demo)".into(),
                model: "C7S".into(),
                kind: "camera".into(),
                online: true,
                health: HealthStatus::Degraded,
                battery_percent: Some(12),
                wifi_signal_percent: Some(24),
                firmware_version: Some("5.11.20-demo".into()),
            },
            Device {
                id: DeviceId("mock-cam-3".into()),
                name: "Einfahrt Süden (Demo, offline)".into(),
                vendor: "Aosu (Demo)".into(),
                model: "C7S".into(),
                kind: "camera".into(),
                online: false,
                health: HealthStatus::Offline,
                battery_percent: None,
                wifi_signal_percent: None,
                firmware_version: Some("5.11.20-demo".into()),
            },
            Device {
                id: DeviceId("mock-cam-4".into()),
                name: "Prototyp-Kamera (Demo, nicht unterstützt)".into(),
                vendor: "Aosu (Demo)".into(),
                model: "X9-Unreleased".into(),
                kind: "camera".into(),
                online: true,
                health: HealthStatus::Unsupported,
                battery_percent: Some(55),
                wifi_signal_percent: Some(88),
                firmware_version: None,
            },
            Device {
                id: DeviceId("mock-cam-5".into()),
                name: "Dachkamera (Demo, teilweise unterstützt)".into(),
                vendor: "Aosu (Demo)".into(),
                model: "C9C".into(),
                kind: "camera".into(),
                online: true,
                health: HealthStatus::PartiallySupported,
                battery_percent: Some(64),
                wifi_signal_percent: Some(40),
                firmware_version: Some("5.11.20-demo".into()),
            },
            Device {
                id: DeviceId("mock-base".into()),
                name: "Basisstation (Demo)".into(),
                vendor: "Aosu (Demo)".into(),
                model: "H1L".into(),
                kind: "base_station".into(),
                online: true,
                health: HealthStatus::Healthy,
                battery_percent: None,
                wifi_signal_percent: None,
                firmware_version: Some("2.4.1-demo".into()),
            },
        ];
        let now = Timestamp::now();
        let events = vec![
            Event {
                device: DeviceId("mock-cam-1".into()),
                kind: EventKind::Motion,
                severity: EventKind::Motion.default_severity(),
                at: now,
                thumbnail_url: None,
                metadata: Default::default(),
            },
            Event {
                device: DeviceId("mock-cam-2".into()),
                kind: EventKind::LowBattery,
                severity: EventKind::LowBattery.default_severity(),
                at: now,
                thumbnail_url: None,
                metadata: Default::default(),
            },
            Event {
                device: DeviceId("mock-cam-3".into()),
                kind: EventKind::Connectivity,
                severity: EventKind::Connectivity.default_severity(),
                at: now,
                thumbnail_url: None,
                metadata: Default::default(),
            },
            Event {
                device: DeviceId("mock-base".into()),
                kind: EventKind::System,
                severity: EventKind::System.default_severity(),
                at: now,
                thumbnail_url: None,
                metadata: Default::default(),
            },
        ];
        Self {
            devices,
            events: Mutex::new(events),
            simulate_network_failure: false,
        }
    }
}

impl Transport for MockTransport {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn discover(&self) -> Result<Vec<Device>> {
        self.devices()
    }

    fn devices(&self) -> Result<Vec<Device>> {
        if self.simulate_network_failure {
            return Err(TransportError::Other(
                "simulated network failure (mock)".into(),
            ));
        }
        Ok(self.devices.clone())
    }

    fn events(&self, since: Timestamp) -> Result<Vec<Event>> {
        if self.simulate_network_failure {
            return Err(TransportError::Other(
                "simulated network failure (mock)".into(),
            ));
        }
        Ok(self
            .events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.at.0 >= since.0)
            .cloned()
            .collect())
    }

    fn live(&self, device: &DeviceId) -> Result<MediaStreamHandle> {
        match self.devices.iter().find(|d| &d.id == device) {
            None => Err(TransportError::NotFound(device.0.clone())),
            Some(d) if d.health == HealthStatus::Offline => {
                Err(TransportError::Other("device is offline (mock)".into()))
            }
            Some(d) if d.health == HealthStatus::Unsupported => Err(TransportError::Unsupported),
            Some(d) => Ok(MediaStreamHandle {
                description: format!("mock live stream for {}", d.id.0),
                ..Default::default()
            }),
        }
    }

    fn talk(&self, device: &DeviceId) -> Result<AudioChannelHandle> {
        Ok(AudioChannelHandle {
            description: format!("mock audio channel for {}", device.0),
            two_way: true,
            ..Default::default()
        })
    }

    fn recordings(&self, device: &DeviceId, range: TimeRange) -> Result<Vec<Recording>> {
        Ok(vec![Recording {
            device: device.clone(),
            started_at: range.from,
            duration_secs: 30,
            source: RecordingSource::LocalBaseStation,
            url: None,
            thumbnail_url: None,
        }])
    }

    fn command(&self, _device: &DeviceId, _cmd: Command) -> Result<()> {
        Ok(())
    }

    fn capability_report(&self) -> CapabilityReport {
        let mut r = CapabilityReport::new();
        for cap in [
            Capability::Discover,
            Capability::DeviceList,
            Capability::DeviceStatus,
            Capability::Events,
            Capability::Notifications,
            Capability::LiveVideo,
            Capability::TwoWayAudio,
            Capability::Recordings,
            Capability::Commands,
        ] {
            r.set(cap, Support::Supported);
        }
        r.set(Capability::Ptz, Support::Unsupported);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_six_demo_devices_across_all_health_states() {
        let t = MockTransport::default();
        let devices = t.devices().unwrap();
        assert_eq!(devices.len(), 6);
        let states: std::collections::HashSet<_> = devices.iter().map(|d| d.health).collect();
        assert!(states.contains(&HealthStatus::Healthy));
        assert!(states.contains(&HealthStatus::Degraded));
        assert!(states.contains(&HealthStatus::Offline));
        assert!(states.contains(&HealthStatus::Unsupported));
        assert!(states.contains(&HealthStatus::PartiallySupported));
    }

    #[test]
    fn live_fails_for_unknown_device() {
        let t = MockTransport::default();
        let err = t.live(&DeviceId("does-not-exist".into())).unwrap_err();
        assert!(matches!(err, TransportError::NotFound(_)));
    }

    #[test]
    fn live_fails_gracefully_for_an_offline_device() {
        let t = MockTransport::default();
        let err = t.live(&DeviceId("mock-cam-3".into())).unwrap_err();
        assert!(matches!(err, TransportError::Other(_)));
    }

    #[test]
    fn live_reports_unsupported_for_an_unsupported_device() {
        let t = MockTransport::default();
        let err = t.live(&DeviceId("mock-cam-4".into())).unwrap_err();
        assert!(matches!(err, TransportError::Unsupported));
    }

    #[test]
    fn ptz_is_reported_unsupported() {
        let t = MockTransport::default();
        assert_eq!(
            t.capability_report().get(Capability::Ptz),
            Support::Unsupported
        );
    }

    #[test]
    fn simulated_network_failure_affects_devices_and_events() {
        let t = MockTransport {
            simulate_network_failure: true,
            ..MockTransport::default()
        };
        assert!(t.devices().is_err());
        assert!(t.events(Timestamp(0)).is_err());
    }

    #[test]
    fn notifications_are_derived_from_warning_and_above_events() {
        let t = MockTransport::default();
        let notifications = t.notifications().unwrap();
        // LowBattery (Warning), Connectivity (Warning) qualify; Motion
        // and System (Info) do not.
        assert_eq!(notifications.len(), 2);
        assert!(notifications
            .iter()
            .all(|n| n.severity >= Severity::Warning));
    }
}
