//! Plugin registry: picks which `Transport` implementation backs the app.
//! Adding a future transport (e.g. an official Aosu API, or a
//! Thingino-flashed local client) means adding one enum variant here, not
//! touching the UI or the Home Assistant bridge.

use crate::transport::Transport;
use crate::transports::cloud::CloudTransport;
use crate::transports::local::LocalTransport;
use crate::transports::mock::MockTransport;

pub enum TransportKind {
    Mock,
    Local { host: String },
    Cloud,
}

pub fn build(kind: TransportKind) -> Box<dyn Transport> {
    match kind {
        TransportKind::Mock => Box::new(MockTransport::default()),
        TransportKind::Local { host } => Box::new(LocalTransport::new(host)),
        TransportKind::Cloud => Box::new(CloudTransport::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_mock_transport_by_default() {
        let t = build(TransportKind::Mock);
        assert_eq!(t.name(), "mock");
        assert!(!t.devices().unwrap().is_empty());
    }
}
