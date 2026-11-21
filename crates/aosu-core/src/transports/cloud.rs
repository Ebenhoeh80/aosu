//! Cloud (Glazero API) transport skeleton. Models the observed hostnames
//! and request field *names* only (see `docs/apk-findings.md`). Does not
//! implement the `Gz-Sign` signature itself -- that stays an open,
//! intentionally undone TODO, see ADR 0002.

use crate::capability::{Capability, CapabilityReport, Support, BlockedReason};
use crate::device::*;
use crate::transport::{Result, Transport, TransportError};

pub const API_HOST: &str = "api.aosulife.com";
pub const H5_HOST: &str = "home.aosulife.com";

#[derive(Debug, Clone, Default)]
pub struct CloudCredentials {
    pub email: String,
    pub password: String,
}

/// Request shape matching observed field *names* (`uuid`, `t`) used by the
/// real app's signing interceptor. The actual signature value is never
/// computed here.
#[derive(Debug, Clone)]
pub struct SignedRequestShape {
    pub uuid: String,
    pub t: String,
    pub sign_header_name: &'static str,
}

impl SignedRequestShape {
    pub fn new(uuid: impl Into<String>, t: impl Into<String>) -> Self {
        Self {
            uuid: uuid.into(),
            t: t.into(),
            sign_header_name: "Gz-Sign",
        }
    }
}

pub struct CloudTransport {
    pub credentials: Option<CloudCredentials>,
}

impl Default for CloudTransport {
    fn default() -> Self {
        Self { credentials: None }
    }
}

impl Transport for CloudTransport {
    fn name(&self) -> &'static str {
        "cloud-glazero"
    }

    fn discover(&self) -> Result<Vec<Device>> {
        Err(TransportError::CredentialsRequired)
    }

    fn devices(&self) -> Result<Vec<Device>> {
        Err(TransportError::CredentialsRequired)
    }

    fn events(&self, _since: Timestamp) -> Result<Vec<Event>> {
        Err(TransportError::CredentialsRequired)
    }

    fn live(&self, _device: &DeviceId) -> Result<MediaStreamHandle> {
        Err(TransportError::CredentialsRequired)
    }

    fn talk(&self, _device: &DeviceId) -> Result<AudioChannelHandle> {
        Err(TransportError::CredentialsRequired)
    }

    fn recordings(&self, _device: &DeviceId, _range: TimeRange) -> Result<Vec<Recording>> {
        Err(TransportError::CredentialsRequired)
    }

    fn command(&self, _device: &DeviceId, _cmd: Command) -> Result<()> {
        Err(TransportError::CredentialsRequired)
    }

    fn capability_report(&self) -> CapabilityReport {
        let mut r = CapabilityReport::new();
        let blocked = Support::Blocked {
            reason: BlockedReason::CredentialsRequired,
        };
        for cap in [
            Capability::DeviceList,
            Capability::DeviceStatus,
            Capability::Events,
            Capability::LiveVideo,
            Capability::TwoWayAudio,
            Capability::Recordings,
            Capability::Commands,
        ] {
            r.set(cap, blocked);
        }
        r.set(Capability::Discover, Support::Unsupported);
        r.set(Capability::Ptz, Support::Unsupported);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devices_require_credentials() {
        let t = CloudTransport::default();
        assert!(matches!(t.devices(), Err(TransportError::CredentialsRequired)));
    }

    #[test]
    fn signed_request_shape_names_the_real_header_without_a_value() {
        let shape = SignedRequestShape::new("u-1", "1700000000");
        assert_eq!(shape.sign_header_name, "Gz-Sign");
    }
}
