//! Cloud (Glazero/Aosu API) transport.
//!
//! Implements read-only login + device inventory using request shapes
//! observed in the official app. No secrets are persisted by this crate;
//! callers provide credentials and a local APK/split path at runtime.

pub mod client;

use self::client::{AosuCloudClient, CloudClientConfig, CloudCredentials, CloudSession};
use crate::capability::{BlockedReason, Capability, CapabilityReport, Support};
use crate::device::*;
use crate::transport::{Result, Transport, TransportError};
use std::sync::{Arc, Mutex};

pub const API_HOST: &str = "api-eu.aosulife.com";

#[derive(Default)]
pub struct CloudTransport {
    credentials: Option<CloudCredentials>,
    apk_material_path: Option<String>,
    client: Mutex<Option<Arc<AosuCloudClient>>>,
    session: Mutex<Option<CloudSession>>,
}

impl CloudTransport {
    pub fn with_config(
        credentials: Option<CloudCredentials>,
        apk_material_path: Option<String>,
    ) -> Self {
        Self {
            credentials,
            apk_material_path,
            client: Mutex::new(None),
            session: Mutex::new(None),
        }
    }

    fn ensure_client(&self) -> Result<Arc<AosuCloudClient>> {
        let mut guard = self
            .client
            .lock()
            .map_err(|e| TransportError::Other(e.to_string()))?;
        if let Some(c) = guard.as_ref() {
            return Ok(c.clone());
        }
        let apk = self
            .apk_material_path
            .as_ref()
            .ok_or(TransportError::CredentialsRequired)?;
        let cfg = CloudClientConfig::new(apk);
        let client = Arc::new(AosuCloudClient::from_apk(cfg)?);
        *guard = Some(client.clone());
        Ok(client)
    }

    fn ensure_session(&self, client: &AosuCloudClient) -> Result<CloudSession> {
        let mut guard = self
            .session
            .lock()
            .map_err(|e| TransportError::Other(e.to_string()))?;
        if let Some(s) = guard.as_ref() {
            return Ok(s.clone());
        }
        let creds = self
            .credentials
            .as_ref()
            .ok_or(TransportError::CredentialsRequired)?;
        let session = client.login(creds)?;
        *guard = Some(session.clone());
        Ok(session)
    }
}

impl Transport for CloudTransport {
    fn name(&self) -> &'static str {
        "cloud-glazero"
    }

    fn discover(&self) -> Result<Vec<Device>> {
        self.devices()
    }

    fn devices(&self) -> Result<Vec<Device>> {
        let client = self.ensure_client()?;
        let session = self.ensure_session(&client)?;
        client.get_device_list(&session)
    }

    fn events(&self, since: Timestamp) -> Result<Vec<Event>> {
        let client = self.ensure_client()?;
        let session = self.ensure_session(&client)?;
        let devices = client.get_device_list(&session)?;
        let serials: Vec<String> = devices.into_iter().map(|d| d.id.0).collect();
        client.get_events(&session, &serials, since, 100)
    }

    fn live(&self, _device: &DeviceId) -> Result<MediaStreamHandle> {
        Err(TransportError::Unsupported)
    }

    fn talk(&self, _device: &DeviceId) -> Result<AudioChannelHandle> {
        Err(TransportError::Unsupported)
    }

    fn recordings(&self, _device: &DeviceId, _range: TimeRange) -> Result<Vec<Recording>> {
        Err(TransportError::Unsupported)
    }

    fn command(&self, _device: &DeviceId, _cmd: Command) -> Result<()> {
        Err(TransportError::Unsupported)
    }

    fn capability_report(&self) -> CapabilityReport {
        let mut r = CapabilityReport::new();
        let configured = self.credentials.is_some() && self.apk_material_path.is_some();
        let inv = if configured {
            Support::Supported
        } else {
            Support::Blocked {
                reason: BlockedReason::CredentialsRequired,
            }
        };
        r.set(Capability::DeviceList, inv);
        r.set(Capability::DeviceStatus, inv);
        // Notifications are derived from events by the Transport trait.
        r.set(Capability::Events, inv);
        r.set(Capability::Notifications, inv);
        r.set(Capability::LiveVideo, Support::Unsupported);
        r.set(Capability::TwoWayAudio, Support::Unsupported);
        r.set(Capability::Recordings, Support::Unsupported);
        r.set(Capability::Commands, Support::Unsupported);
        r.set(
            Capability::Discover,
            if configured {
                Support::Supported
            } else {
                Support::Blocked {
                    reason: BlockedReason::CredentialsRequired,
                }
            },
        );
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
        assert!(matches!(
            t.devices(),
            Err(TransportError::CredentialsRequired)
        ));
    }

    #[test]
    fn configured_capabilities_report_supported_inventory() {
        let t = CloudTransport::with_config(
            Some(CloudCredentials {
                email: "a@b.c".into(),
                password: "x".into(),
            }),
            Some("C:/tmp/base.apk".into()),
        );
        let r = t.capability_report();
        assert_eq!(r.get(Capability::DeviceList), Support::Supported);
        assert_eq!(r.get(Capability::Events), Support::Supported);
        assert_eq!(r.get(Capability::LiveVideo), Support::Unsupported);
    }

    #[test]
    fn events_require_credentials() {
        let t = CloudTransport::default();
        assert!(matches!(
            t.events(crate::device::Timestamp(0)),
            Err(TransportError::CredentialsRequired)
        ));
    }
}
