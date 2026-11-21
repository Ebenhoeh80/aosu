//! Passive, read-only network diagnostics -- the same checks already
//! reviewed and run manually as `tools/lan_probe.py`, productionized here
//! with tests. Never sends a control command, never authenticates.

use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Fields that must never be logged or serialized into a diagnostics
/// bundle. Centralized here so every log call site can check against one
/// list instead of relying on reviewers to remember it.
pub const NEVER_LOG_FIELDS: &[&str] = &[
    "password",
    "local_key",
    "token",
    "session",
    "gz-sign",
    "app_secret",
];

pub fn contains_sensitive_field(key: &str) -> bool {
    let lower = key.to_lowercase();
    NEVER_LOG_FIELDS.iter().any(|f| lower.contains(f))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortCheckResult {
    pub port: u16,
    pub open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDiagnostics {
    pub host: String,
    pub ports: Vec<PortCheckResult>,
}

/// Check a fixed list of ports with a short connect timeout. Read-only:
/// only establishes (and immediately drops) a TCP connection, sends
/// nothing.
pub fn scan_ports(host: &str, ports: &[u16], timeout: Duration) -> NetworkDiagnostics {
    let mut results = Vec::with_capacity(ports.len());
    for &port in ports {
        let open = resolve(host, port)
            .map(|addr| TcpStream::connect_timeout(&addr, timeout).is_ok())
            .unwrap_or(false);
        results.push(PortCheckResult { port, open });
    }
    NetworkDiagnostics {
        host: host.to_string(),
        ports: results,
    }
}

fn resolve(host: &str, port: u16) -> Option<SocketAddr> {
    (host, port).to_socket_addrs().ok()?.next()
}

pub const DEFAULT_PROBE_PORTS: &[u16] = &[80, 443, 554, 6668, 8000, 8080, 8554, 8883, 8899];

/// Everything the "Diagnostics Center" exports in one shot: capability
/// report, an optional network scan, and basic runtime/environment
/// facts. Deliberately contains nothing from `NEVER_LOG_FIELDS` -- there
/// is no field here that could, since this crate never holds a
/// credential or key in the first place (see ADR 0002).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsBundle {
    pub generated_at_epoch_seconds: u64,
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub active_transport: String,
    pub capability_report: crate::capability::CapabilityReport,
    pub network: Option<NetworkDiagnostics>,
}

pub fn build_bundle(
    active_transport: &str,
    capability_report: crate::capability::CapabilityReport,
    network: Option<NetworkDiagnostics>,
) -> DiagnosticsBundle {
    DiagnosticsBundle {
        generated_at_epoch_seconds: crate::device::Timestamp::now().0,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        active_transport: active_transport.to_string(),
        capability_report,
        network,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn detects_sensitive_field_names_case_insensitively() {
        assert!(contains_sensitive_field("Local_Key"));
        assert!(contains_sensitive_field("Gz-Sign"));
        assert!(!contains_sensitive_field("device_name"));
    }

    #[test]
    fn finds_a_real_open_port_on_loopback() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1];
                let _ = stream.read(&mut buf);
            }
        });
        let result = scan_ports("127.0.0.1", &[port], Duration::from_millis(500));
        assert!(result.ports[0].open);
        drop(TcpStream::connect(("127.0.0.1", port)));
        let _ = handle.join();
    }

    #[test]
    fn reports_closed_for_unused_port() {
        // Port 1 is reserved/unlikely to be listening in any test env.
        let result = scan_ports("127.0.0.1", &[1], Duration::from_millis(200));
        assert!(!result.ports[0].open);
    }

    #[test]
    fn bundle_round_trips_through_json_without_a_network_scan() {
        let bundle = build_bundle("mock", crate::capability::CapabilityReport::new(), None);
        let json = serde_json::to_string(&bundle).unwrap();
        assert!(json.contains("\"activeTransport\":\"mock\""));
        let back: DiagnosticsBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(back.active_transport, "mock");
    }
}
