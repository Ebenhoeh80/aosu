//! The core abstraction: every way of talking to Aosu/H1L hardware
//! implements this trait. The UI and the Home Assistant bridge depend only
//! on this, never on a concrete transport.

use crate::capability::CapabilityReport;
use crate::device::{
    AudioChannelHandle, Command, Device, DeviceId, Event, MediaStreamHandle, Notification,
    Recording, TimeRange, Timestamp,
};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransportError {
    #[error("not supported by this transport")]
    Unsupported,
    #[error("credentials required")]
    CredentialsRequired,
    #[error("local key required")]
    LocalKeyRequired,
    #[error("device not found: {0}")]
    NotFound(String),
    #[error("transport-level error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, TransportError>;

pub trait Transport: Send + Sync {
    fn name(&self) -> &'static str;

    fn discover(&self) -> Result<Vec<Device>>;
    fn devices(&self) -> Result<Vec<Device>>;
    fn events(&self, since: Timestamp) -> Result<Vec<Event>>;
    /// Default: derives notifications from `events()` at Warning+
    /// severity. A transport with a dedicated notification channel
    /// (e.g. the push-notification bridge) can override this.
    fn notifications(&self) -> Result<Vec<Notification>> {
        Ok(self
            .events(Timestamp(0))?
            .into_iter()
            .filter(|e| e.severity >= crate::device::Severity::Warning)
            .enumerate()
            .map(|(i, e)| Notification {
                id: format!("{}-{}", e.device.0, i),
                device: Some(e.device.clone()),
                severity: e.severity,
                title: format!("{:?}", e.kind),
                body: format!("{:?} at device {}", e.kind, e.device.0),
                at: e.at,
                acknowledged: false,
            })
            .collect())
    }
    fn live(&self, device: &DeviceId) -> Result<MediaStreamHandle>;
    fn talk(&self, device: &DeviceId) -> Result<AudioChannelHandle>;
    fn recordings(&self, device: &DeviceId, range: TimeRange) -> Result<Vec<Recording>>;
    fn command(&self, device: &DeviceId, cmd: Command) -> Result<()>;

    fn capability_report(&self) -> CapabilityReport;
}
