//! A fully working, fake transport. Lets the whole UI and the Home
//! Assistant bridge be built and demoed before any real protocol question
//! is resolved.

use crate::capability::{Capability, CapabilityReport, Support};
use crate::device::*;
use crate::transport::{Result, Transport};
use std::sync::Mutex;

pub struct MockTransport {
    devices: Vec<Device>,
    events: Mutex<Vec<Event>>,
}

impl Default for MockTransport {
    fn default() -> Self {
        let devices = vec![
            Device {
                id: DeviceId("mock-cam-1".into()),
                name: "Terasse Norden (Demo)".into(),
                model: "C7S".into(),
                kind: "camera".into(),
                online: true,
                battery_percent: Some(81),
                wifi_signal_percent: Some(70),
            },
            Device {
                id: DeviceId("mock-cam-2".into()),
                name: "Garten Westen (Demo)".into(),
                model: "C7S".into(),
                kind: "camera".into(),
                online: true,
                battery_percent: Some(54),
                wifi_signal_percent: Some(54),
            },
            Device {
                id: DeviceId("mock-base".into()),
                name: "Basisstation (Demo)".into(),
                model: "H1L".into(),
                kind: "base_station".into(),
                online: true,
                battery_percent: None,
                wifi_signal_percent: None,
            },
        ];
        let events = vec![Event {
            device: DeviceId("mock-cam-1".into()),
            kind: EventKind::Motion,
            at: Timestamp::now(),
            thumbnail_url: None,
        }];
        Self {
            devices,
            events: Mutex::new(events),
        }
    }
}

impl Transport for MockTransport {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn discover(&self) -> Result<Vec<Device>> {
        Ok(self.devices.clone())
    }

    fn devices(&self) -> Result<Vec<Device>> {
        Ok(self.devices.clone())
    }

    fn events(&self, since: Timestamp) -> Result<Vec<Event>> {
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
        if self.devices.iter().any(|d| &d.id == device) {
            Ok(MediaStreamHandle {
                description: format!("mock live stream for {}", device.0),
            })
        } else {
            Err(crate::transport::TransportError::NotFound(device.0.clone()))
        }
    }

    fn talk(&self, device: &DeviceId) -> Result<AudioChannelHandle> {
        Ok(AudioChannelHandle {
            description: format!("mock audio channel for {}", device.0),
        })
    }

    fn recordings(&self, device: &DeviceId, range: TimeRange) -> Result<Vec<Recording>> {
        Ok(vec![Recording {
            device: device.clone(),
            started_at: range.from,
            duration_secs: 30,
            source: RecordingSource::LocalBaseStation,
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
    fn lists_three_demo_devices() {
        let t = MockTransport::default();
        assert_eq!(t.devices().unwrap().len(), 3);
    }

    #[test]
    fn live_fails_for_unknown_device() {
        let t = MockTransport::default();
        let err = t.live(&DeviceId("does-not-exist".into())).unwrap_err();
        assert!(matches!(err, crate::transport::TransportError::NotFound(_)));
    }

    #[test]
    fn ptz_is_reported_unsupported() {
        let t = MockTransport::default();
        assert_eq!(
            t.capability_report().get(Capability::Ptz),
            Support::Unsupported
        );
    }
}
