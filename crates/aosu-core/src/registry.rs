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
use crate::transports::vendor2_mock::Vendor2MockTransport;
use std::collections::HashMap;

pub type TransportFactory = Box<dyn Fn(&TransportConfig) -> Box<dyn Transport> + Send + Sync>;

/// Minimal, generic configuration passed to a factory. Deliberately not
/// Aosu-specific: a future brand's transport can read `host`/`options`
/// the same way `LocalTransport` does.
#[derive(Debug, Clone, Default)]
pub struct TransportConfig {
    pub host: Option<String>,
    pub options: HashMap<String, String>,
}

impl TransportConfig {
    pub fn with_host(host: impl Into<String>) -> Self {
        Self {
            host: Some(host.into()),
            options: HashMap::new(),
        }
    }
}

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
        r.register("cloud", |_cfg| {
            Box::new(CloudTransport::default()) as Box<dyn Transport>
        });
        // Fictional second vendor, registered the same way a real
        // third-party crate would -- proves the registry genuinely
        // supports more than one vendor, not just "mock" by name. See
        // transports::vendor2_mock for why it's deliberately fictional.
        r.register("kestrel-mock", |_cfg| {
            Box::new(Vendor2MockTransport::default()) as Box<dyn Transport>
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
        assert_eq!(names, vec!["cloud", "kestrel-mock", "local", "mock"]);
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
}
