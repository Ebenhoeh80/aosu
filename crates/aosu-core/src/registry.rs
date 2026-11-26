//! Transport registry: a real plugin mechanism, not a closed enum.
//!
//! Adding a future transport -- an official Aosu API if one is ever
//! granted, a Thingino-flashed local client, or a completely different
//! Tuya-derived brand's client -- means registering a named factory
//! function here (or from a third-party crate that depends on
//! `aosu-core`), not editing a hardcoded match statement anywhere else in
//! the UI or Home Assistant bridge. Those only ever see `dyn Transport`.
//!
//! This is intentionally *not* dynamic-library loading (no `.dll`/`.so`
//! plugin loading, no unsafe FFI boundary for untrusted native code).
//! Community "device adapters" are ordinary Rust crates implementing
//! `Transport` and registering a factory, compiled into the app -- the
//! same trust model as any other Rust dependency.

use crate::transport::Transport;
use crate::transports::cloud::CloudTransport;
use crate::transports::local::LocalTransport;
use crate::transports::mock::MockTransport;
use crate::transports::notification_log::NotificationLogTransport;
use crate::transports::tuya_cloud::{TuyaCloudTransport, TuyaConfig};
use crate::transports::vendor2_mock::Vendor2MockTransport;
use std::collections::HashMap;

pub type TransportFactory = Box<dyn Fn(&TransportConfig) -> Box<dyn Transport> + Send + Sync>;

/// Minimal, generic configuration passed to a factory. Deliberately not
/// Aosu-specific: a future brand's transport can read `host`/`options`
/// the same way `LocalTransport` does.
#[derive(Clone, Default)]
pub struct TransportConfig {
    pub host: Option<String>,
    pub options: HashMap<String, String>,
}

/// Debug output exposes option *names* only. Values can be passwords, tokens
/// or client secrets, so they are never printed even in diagnostics.
impl std::fmt::Debug for TransportConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportConfig")
            .field("host", &self.host.as_ref().map(|_| "<set>"))
            .field("options", &self.options.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl TransportConfig {
    pub fn with_host(host: impl Into<String>) -> Self {
        Self {
            host: Some(host.into()),
            options: HashMap::new(),
        }
    }

    pub fn with_option(key: impl Into<String>, value: impl Into<String>) -> Self {
        let mut options = HashMap::new();
        options.insert(key.into(), value.into());
        Self {
            host: None,
            options,
        }
    }
}

/// Pure mapping of session-only UI options into the Tuya adapter config.
fn tuya_config(cfg: &TransportConfig) -> TuyaConfig {
    let get = |key: &str| cfg.options.get(key).cloned().unwrap_or_default();
    let region = get("region");
    TuyaConfig {
        client_id: get("clientId"),
        client_secret: get("clientSecret"),
        region: if region.is_empty() {
            "eu".to_string()
        } else {
            region
        },
        uid: cfg.options.get("uid").cloned().filter(|s| !s.is_empty()),
        go2rtc_base: cfg
            .options
            .get("go2rtcBase")
            .cloned()
            .filter(|s| !s.is_empty()),
        go2rtc_stream: cfg
            .options
            .get("go2rtcStream")
            .cloned()
            .filter(|s| !s.is_empty()),
        callback_base: cfg
            .options
            .get("callbackBase")
            .cloned()
            .filter(|s| !s.is_empty()),
        ptz_enabled: cfg
            .options
            .get("ptzEnabled")
            .is_some_and(|v| v.eq_ignore_ascii_case("true")),
    }
}

/// Where `push-log` looks for the JSONL file if the caller didn't
/// configure `options["path"]` -- see docs/event-pipeline.md.
pub const DEFAULT_PUSH_LOG_PATH: &str = "aosu-events.jsonl";

pub struct TransportRegistry {
    factories: HashMap<String, TransportFactory>,
}

impl TransportRegistry {
    /// A registry with the transports this project ships pre-registered.
    /// Third-party code can start from `TransportRegistry::empty()`
    /// instead and register only what it needs.
    pub fn with_builtins() -> Self {
        let mut r = Self::empty();
        r.register("mock", |_cfg| {
            Box::new(MockTransport::default()) as Box<dyn Transport>
        });
        r.register("local", |cfg| {
            let host = cfg.host.clone().unwrap_or_default();
            Box::new(LocalTransport::new(host)) as Box<dyn Transport>
        });
        r.register("cloud", |cfg| {
            let email = cfg.options.get("email").cloned();
            let password = cfg.options.get("password").cloned();
            let apk_path = cfg
                .options
                .get("apkPath")
                .cloned()
                .or_else(|| cfg.options.get("apk_path").cloned());
            let creds = match (email, password) {
                (Some(email), Some(password)) if !email.is_empty() && !password.is_empty() => {
                    Some(crate::transports::cloud::client::CloudCredentials { email, password })
                }
                _ => None,
            };
            Box::new(CloudTransport::with_config(creds, apk_path)) as Box<dyn Transport>
        });
        // Fictional second vendor, registered the same way a real
        // third-party crate would -- proves the registry genuinely
        // supports more than one vendor, not just "mock" by name. See
        // transports::vendor2_mock for why it's deliberately fictional.
        r.register("kestrel-mock", |_cfg| {
            Box::new(Vendor2MockTransport::default()) as Box<dyn Transport>
        });
        r.register("push-log", |cfg| {
            let path = cfg
                .options
                .get("path")
                .cloned()
                .unwrap_or_else(|| DEFAULT_PUSH_LOG_PATH.to_string());
            Box::new(NotificationLogTransport::new(path)) as Box<dyn Transport>
        });
        // Official Tuya Cloud adapter. All values are RAM-only and must be
        // supplied by the user from their own IoT project.
        r.register("tuya-cloud", |cfg| {
            let config = tuya_config(cfg);
            Box::new(
                TuyaCloudTransport::new(config)
                    .expect("TuyaCloudTransport construction never fails without credentials"),
            ) as Box<dyn Transport>
        });
        r
    }

    pub fn empty() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        name: impl Into<String>,
        factory: impl Fn(&TransportConfig) -> Box<dyn Transport> + Send + Sync + 'static,
    ) {
        self.factories.insert(name.into(), Box::new(factory));
    }

    pub fn names(&self) -> Vec<&str> {
        self.factories.keys().map(String::as_str).collect()
    }

    pub fn build(&self, name: &str, config: &TransportConfig) -> Option<Box<dyn Transport>> {
        self.factories.get(name).map(|f| f(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_include_mock_local_cloud() {
        let r = TransportRegistry::with_builtins();
        let mut names = r.names();
        names.sort();
        assert_eq!(
            names,
            vec![
                "cloud",
                "kestrel-mock",
                "local",
                "mock",
                "push-log",
                "tuya-cloud"
            ]
        );
    }

    /// The UI blends features from `capability_report()`. Every built-in
    /// transport must therefore answer all ten capabilities explicitly;
    /// `Unknown` would silently hide a feature instead of showing its state.
    #[test]
    fn every_builtin_reports_all_capabilities() {
        use crate::capability::{Capability, Support};
        let all = [
            Capability::Discover,
            Capability::DeviceList,
            Capability::DeviceStatus,
            Capability::Events,
            Capability::Notifications,
            Capability::LiveVideo,
            Capability::TwoWayAudio,
            Capability::Ptz,
            Capability::Recordings,
            Capability::Commands,
        ];
        let r = TransportRegistry::with_builtins();
        for name in r.names() {
            let t = r
                .build(name, &TransportConfig::default())
                .unwrap_or_else(|| panic!("missing builtin {name}"));
            let report = t.capability_report();
            for cap in all {
                assert_ne!(
                    report.get(cap),
                    Support::Unknown,
                    "{name} leaves {cap:?} unknown"
                );
            }
        }
    }

    #[test]
    fn builds_mock_transport_by_name() {
        let r = TransportRegistry::with_builtins();
        let t = r.build("mock", &TransportConfig::default()).unwrap();
        assert_eq!(t.name(), "mock");
        assert!(!t.devices().unwrap().is_empty());
    }

    #[test]
    fn unknown_name_returns_none() {
        let r = TransportRegistry::with_builtins();
        assert!(r
            .build("does-not-exist", &TransportConfig::default())
            .is_none());
    }

    #[test]
    fn third_party_code_can_register_its_own_transport_without_touching_builtins() {
        let mut r = TransportRegistry::empty();
        r.register("community-brand-x", |_cfg| {
            Box::new(MockTransport::default()) as Box<dyn Transport>
        });
        assert!(r
            .build("community-brand-x", &TransportConfig::default())
            .is_some());
        assert!(r.build("mock", &TransportConfig::default()).is_none());
    }

    #[test]
    fn local_factory_passes_the_configured_host_through() {
        let r = TransportRegistry::with_builtins();
        let cfg = TransportConfig::with_host("192.0.2.1");
        let t = r.build("local", &cfg).unwrap();
        assert_eq!(t.name(), "local-h1l");
    }

    #[test]
    fn tuya_options_map_to_session_only_config_and_debug_stays_redacted() {
        let cfg = TransportConfig {
            host: None,
            options: HashMap::from([
                ("clientId".to_string(), "cid".to_string()),
                (
                    "clientSecret".to_string(),
                    "redacted-test-value".to_string(),
                ),
                ("region".to_string(), "us".to_string()),
                ("go2rtcStream".to_string(), "tuya_webrtc".to_string()),
                ("ptzEnabled".to_string(), "false".to_string()),
                ("email".to_string(), String::new()),
            ]),
        };
        let tuya = tuya_config(&cfg);
        assert_eq!(tuya.client_id, "cid");
        assert_eq!(tuya.region, "us");
        assert_eq!(tuya.go2rtc_stream.as_deref(), Some("tuya_webrtc"));
        assert!(!tuya.ptz_enabled);
        assert_eq!(tuya_config(&TransportConfig::default()).region, "eu");
        let debug = format!("{cfg:?}");
        assert!(!debug.contains("redacted-test-value"));
        assert!(debug.contains("clientSecret"));
    }
}
