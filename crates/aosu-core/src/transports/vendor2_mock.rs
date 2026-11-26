//! Multi-vendor architecture validation.
//!
//! "Kestrel Labs" is an **entirely fictional** camera brand, invented
//! only to prove that `Transport`/`Device`/`CapabilityReport` hold up for
//! a second, structurally different vendor -- not a stand-in for any
//! real company. No real vendor was available to validate this with, so
//! per `docs/architecture.md` ("avoid speculative implementation") this
//! stays clearly fictional rather than guessing at a real brand's
//! behavior.
//!
//! Deliberately implemented independently of `mock.rs` (no shared base
//! struct) so the test suite proves the trait boundary itself works, not
//! just that one implementation happens to satisfy it.

use crate::capability::{Capability, CapabilityReport, Support};
use crate::device::*;
use crate::transport::{Result, Transport, TransportError};

pub struct Vendor2MockTransport {
    devices: Vec<Device>,
}

impl Default for Vendor2MockTransport {
    fn default() -> Self {
        Self {
            devices: vec![
                Device {
                    id: DeviceId("kestrel-ptz-1".into()),
                    name: "Kestrel PTZ Dome (Demo)".into(),
                    vendor: "Kestrel Labs (fiktiv, Demo)".into(),
                    model: "K2-PTZ".into(),
                    kind: "camera".into(),
                    online: true,
                    health: HealthStatus::Healthy,
                    // Mains-powered in this fictional product line.
                    battery_percent: None,
                    wifi_signal_percent: Some(90),
                    firmware_version: Some("1.0.0-fictional".into()),
                },
                Device {
                    id: DeviceId("kestrel-doorbell-1".into()),
                    name: "Kestrel Doorbell (Demo)".into(),
                    vendor: "Kestrel Labs (fiktiv, Demo)".into(),
                    model: "K-Bell".into(),
                    kind: "doorbell".into(),
                    online: true,
                    health: HealthStatus::Healthy,
                    battery_percent: Some(95),
                    wifi_signal_percent: Some(60),
                    firmware_version: Some("1.0.0-fictional".into()),
                },
            ],
        }
    }
}

impl Transport for Vendor2MockTransport {
    fn name(&self) -> &'static str {
        "kestrel-mock"
    }

    fn discover(&self) -> Result<Vec<Device>> {
        Ok(self.devices.clone())
    }

    fn devices(&self) -> Result<Vec<Device>> {
        Ok(self.devices.clone())
    }

    fn events(&self, _since: Timestamp) -> Result<Vec<Event>> {
        Ok(vec![])
    }

    fn live(&self, device: &DeviceId) -> Result<MediaStreamHandle> {
        if self.devices.iter().any(|d| &d.id == device) {
            Ok(MediaStreamHandle {
                description: format!("fictional Kestrel stream for {}", device.0),
                ..Default::default()
            })
        } else {
            Err(TransportError::NotFound(device.0.clone()))
        }
    }

    fn talk(&self, _device: &DeviceId) -> Result<AudioChannelHandle> {
        Err(TransportError::Unsupported)
    }

    fn recordings(&self, _device: &DeviceId, _range: TimeRange) -> Result<Vec<Recording>> {
        Ok(vec![])
    }

    fn command(&self, device: &DeviceId, _cmd: Command) -> Result<()> {
        if self.devices.iter().any(|d| &d.id == device) {
            Ok(())
        } else {
            Err(TransportError::NotFound(device.0.clone()))
        }
    }

    fn capability_report(&self) -> CapabilityReport {
        let mut r = CapabilityReport::new();
        for cap in [
            Capability::Discover,
            Capability::DeviceList,
            Capability::DeviceStatus,
            Capability::LiveVideo,
            Capability::Commands,
            // Unlike the Aosu demo fixtures, this fictional vendor's PTZ
            // dome genuinely supports PTZ -- showing the capability
            // model varies per vendor, not hardcoded "no PTZ anywhere".
            Capability::Ptz,
        ] {
            r.set(cap, Support::Supported);
        }
        r.set(Capability::Events, Support::Unsupported);
        r.set(Capability::Notifications, Support::Unsupported);
        r.set(Capability::TwoWayAudio, Support::Unsupported);
        r.set(Capability::Recordings, Support::Unsupported);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_two_fictional_devices_of_a_different_vendor() {
        let t = Vendor2MockTransport::default();
        let devices = t.devices().unwrap();
        assert_eq!(devices.len(), 2);
        assert!(devices.iter().all(|d| d.vendor.starts_with("Kestrel")));
    }

    #[test]
    fn unlike_aosu_demo_devices_this_vendor_reports_ptz_supported() {
        let t = Vendor2MockTransport::default();
        assert_eq!(
            t.capability_report().get(Capability::Ptz),
            Support::Supported
        );
    }

    #[test]
    fn command_fails_for_unknown_device() {
        let t = Vendor2MockTransport::default();
        let err = t
            .command(
                &DeviceId("not-a-real-device".into()),
                Command::SetLight(true),
            )
            .unwrap_err();
        assert!(matches!(err, TransportError::NotFound(_)));
    }
}
