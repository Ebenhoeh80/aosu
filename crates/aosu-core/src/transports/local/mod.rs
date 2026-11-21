//! Local (H1L) transport skeleton. Talks only generic, publicly documented
//! Tuya LAN framing (see the brand-agnostic `tuya_lan_protocol` crate).
//! Never sends a device key or secret -- there isn't one available yet
//! (see ADR 0002) -- so every capability here legitimately reports
//! `Blocked { reason: LocalKeyRequired }` until that changes.

use crate::capability::{BlockedReason, Capability, CapabilityReport, Support};
use crate::device::*;
use crate::transport::{Result, Transport, TransportError};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tuya_lan_protocol::{build_frame, parse_header, Command as TuyaCommand, FrameHeader};

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

    /// Read-only protocol fingerprint: sends one unauthenticated, generic
    /// Tuya LAN frame (no device key, no control command) and reports
    /// what came back. Mirrors the manual probe already reviewed in
    /// ADR 0002 -- this just productionizes it with tests. Never used to
    /// authenticate or control anything.
    pub fn fingerprint(&self, command: TuyaCommand) -> FingerprintResult {
        let addr = match format!("{}:{}", self.host, self.port).parse() {
            Ok(a) => a,
            Err(_) => return FingerprintResult::connect_failed("invalid host/port"),
        };
        let mut stream = match TcpStream::connect_timeout(&addr, self.connect_timeout) {
            Ok(s) => s,
            Err(e) => return FingerprintResult::connect_failed(&e.to_string()),
        };
        let _ = stream.set_read_timeout(Some(self.connect_timeout));
        let frame = build_frame(1, command, &[0u8; 16]);
        if stream.write_all(&frame).is_err() {
            return FingerprintResult::connect_failed("send failed");
        }
        // A peer that closes before reading our frame typically resets
        // the connection (RST) rather than ending it cleanly (FIN) --
        // from a fingerprinting point of view both mean the same thing:
        // no usable response came back. We don't try to tell network
        // errors apart beyond that; `closed_without_response` is the
        // signal callers actually need (see ADR 0002).
        let mut buf = [0u8; 512];
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => FingerprintResult {
                connected: true,
                closed_without_response: true,
                header: None,
            },
            Ok(n) => FingerprintResult {
                connected: true,
                closed_without_response: false,
                header: parse_header(&buf[..n]),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FingerprintResult {
    pub connected: bool,
    pub closed_without_response: bool,
    pub header: Option<FrameHeader>,
}

impl FingerprintResult {
    fn connect_failed(_reason: &str) -> Self {
        Self {
            connected: false,
            closed_without_response: false,
            header: None,
        }
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
    use std::net::TcpListener;
    use std::thread;

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

    #[test]
    fn fingerprint_detects_a_server_that_closes_without_responding() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                // Drain the client's frame first, like the real H1L does
                // (observed in ADR 0002: it reads then closes cleanly,
                // not a reset) -- a close that skips this step produces
                // a connection reset instead, which is a separate case.
                let mut buf = [0u8; 64];
                let _ = stream.read(&mut buf);
                drop(stream);
            }
        });
        let t = LocalTransport {
            host: "127.0.0.1".into(),
            port,
            connect_timeout: Duration::from_millis(500),
        };
        let result = t.fingerprint(TuyaCommand::DpQuery);
        assert!(result.connected);
        assert!(result.closed_without_response);
        let _ = handle.join();
    }

    #[test]
    fn fingerprint_treats_a_connection_reset_the_same_as_a_clean_close() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                // Close without reading: on most stacks this produces a
                // connection reset (RST) rather than a clean end (FIN).
                drop(stream);
            }
        });
        let t = LocalTransport {
            host: "127.0.0.1".into(),
            port,
            connect_timeout: Duration::from_millis(500),
        };
        let result = t.fingerprint(TuyaCommand::DpQuery);
        assert!(result.connected);
        assert!(result.closed_without_response);
        let _ = handle.join();
    }

    #[test]
    fn fingerprint_parses_a_well_formed_echo_response() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 512];
                if let Ok(n) = stream.read(&mut buf) {
                    let _ = stream.write_all(&buf[..n]); // simple echo
                }
            }
        });
        let t = LocalTransport {
            host: "127.0.0.1".into(),
            port,
            connect_timeout: Duration::from_millis(500),
        };
        let result = t.fingerprint(TuyaCommand::SessKeyNegStart);
        assert!(result.connected);
        assert!(!result.closed_without_response);
        assert!(result.header.unwrap().prefix_ok);
        let _ = handle.join();
    }
}
