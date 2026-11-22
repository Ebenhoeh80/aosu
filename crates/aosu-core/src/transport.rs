//! The core abstraction: every way of talking to Aosu/H1L hardware
//! implements this trait. The UI and the Home Assistant bridge depend only
//! on this, never on a concrete transport.

use crate::capability::CapabilityReport;
use crate::device::{
    AudioChannelHandle, Command, Device, DeviceId, Event, MediaStreamHandle, Notification,
    Recording, TimeRange, Timestamp,
};
use crate::event_store::{DeviceActivity, EventAnalytics, EventQuery, StoredEvent};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
}

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

    // -- Event Platform: persistent storage/query/analytics/export/
    // retention. Defaults to `Unsupported` for every transport that
    // doesn't have a durable event store (most of them) -- only
    // `NotificationLogTransport` overrides these today. Kept as trait
    // methods (not a separate "if let Some(store) = ..." API) so the
    // Mutex-guarded access each concrete transport needs stays fully
    // encapsulated; callers never see a lock guard or a concrete store
    // type, only `dyn Transport`.

    fn query_events(&self, _query: &EventQuery) -> Result<Vec<StoredEvent>> {
        Err(TransportError::Unsupported)
    }

    fn event_analytics(&self, _now_ms: i64) -> Result<EventAnalytics> {
        Err(TransportError::Unsupported)
    }

    fn device_activity(&self) -> Result<Vec<DeviceActivity>> {
        Err(TransportError::Unsupported)
    }

    fn export_events(&self, _query: &EventQuery, _format: ExportFormat) -> Result<String> {
        Err(TransportError::Unsupported)
    }

    /// Returns the number of rows deleted.
    fn apply_event_retention(
        &self,
        _now_ms: i64,
        _max_age_days: Option<u32>,
        _max_rows: Option<i64>,
    ) -> Result<i64> {
        Err(TransportError::Unsupported)
    }
}
