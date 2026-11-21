//! Local (H1L) transport skeleton. Talks only generic, publicly documented
//! Tuya LAN framing (see `tuya_frame`). Never sends a device key or secret
//! -- there isn't one available yet (see ADR 0002) -- so every capability
//! here legitimately reports `Blocked { reason: LocalKeyRequired }` until
//! that changes.

pub mod tuya_frame;

use crate::capability::{Capability, CapabilityReport, Support, BlockedReason};
use crate::device::*;
use crate::transport::{Result, Transport, TransportError};
use std::net::TcpStream;
use std::time::Duration;

pub struct LocalTransport {
    pub host: String,
    pub port: u16,
    pub connect_timeout: Duration,
}

impl LocalTransport {
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: 6668,
            connect_timeout: Duration::from_secs(3),
        }
    }

    /// Read-only reachability probe: does the standard Tuya local-control
    /// port accept a TCP connection at all? Does not send any frame.
    pub fn port_reachable(&self) -> bool {
        let addr = format!("{}:{}", self.host, self.port);
        addr.parse()
            .ok()
            .and_then(|a| TcpStream::connect_timeout(&a, self.connect_timeout).ok())
            .is_some()
    }
}

impl Transport for LocalTransport {
    fn name(&self) -> &'static str {
        "local-h1l"
    }

    fn discover(&self) -> Result<Vec<Device>> {
        Err(TransportError::LocalKeyRequired)
    }

    fn devices(&self) -> Result<Vec<Device>> {
        Err(TransportError::LocalKeyRequired)
    }

    fn events(&self, _since: Timestamp) -> Result<Vec<Event>> {
        Err(TransportError::Unsupported)
    }

    fn live(&self, _device: &DeviceId) -> Result<MediaStreamHandle> {
        Err(TransportError::LocalKeyRequired)
    }

    fn talk(&self, _device: &DeviceId) -> Result<AudioChannelHandle> {
        Err(TransportError::Unsupported)
    }

    fn recordings(&self, _device: &DeviceId, _range: TimeRange) -> Result<Vec<Recording>> {
        Err(TransportError::Unsupported)
    }

    fn command(&self, _device: &DeviceId, _cmd: Command) -> Result<()> {
        Err(TransportError::LocalKeyRequired)
    }

    fn capability_report(&self) -> CapabilityReport {
        let mut r = CapabilityReport::new();
        let blocked = Support::Blocked {
            reason: BlockedReason::LocalKeyRequired,
        };
        for cap in [
            Capability::DeviceList,
            Capability::DeviceStatus,
            Capability::LiveVideo,
            Capability::Commands,
        ] {
            r.set(cap, blocked);
        }
        r.set(Capability::Discover, Support::Unknown);
        r.set(Capability::Events, Support::Unsupported);
        r.set(Capability::TwoWayAudio, Support::Unsupported);
        r.set(Capability::Recordings, Support::Unsupported);
        r.set(Capability::Ptz, Support::Unsupported);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_local_key_required_for_device_list() {
        let t = LocalTransport::new("192.0.2.1"); // TEST-NET-1, never reachable
        assert!(matches!(t.devices(), Err(TransportError::LocalKeyRequired)));
    }

    #[test]
    fn capability_report_marks_core_features_blocked() {
        let t = LocalTransport::new("192.0.2.1");
        let report = t.capability_report();
        assert_eq!(
            report.get(Capability::LiveVideo),
            Support::Blocked {
                reason: BlockedReason::LocalKeyRequired
            }
        );
    }

    #[test]
    fn unreachable_host_reports_not_reachable() {
        let t = LocalTransport {
            host: "192.0.2.1".into(),
            port: 6668,
            connect_timeout: Duration::from_millis(200),
        };
        assert!(!t.port_reachable());
    }
}
