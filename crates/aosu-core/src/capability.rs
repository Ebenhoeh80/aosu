//! Per-feature capability reporting. A Transport never silently pretends to
//! support something it hasn't validated; the UI reads this report to decide
//! what to show as available.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Capability {
    Discover,
    DeviceList,
    DeviceStatus,
    Events,
    Notifications,
    LiveVideo,
    TwoWayAudio,
    Ptz,
    Recordings,
    Commands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Support {
    /// Observed working against the real target.
    Supported,
    /// Reachable/attempted but blocked on something concrete (e.g. a
    /// missing credential or key) -- not a permanent "no".
    Blocked { reason: BlockedReason },
    /// Not implemented by this transport at all.
    Unsupported,
    /// Not yet determined.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockedReason {
    CredentialsRequired,
    LocalKeyRequired,
    PinningUnresolved,
    NotImplementedYet,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityReport {
    entries: BTreeMap<Capability, Support>,
}

impl CapabilityReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, cap: Capability, support: Support) -> &mut Self {
        self.entries.insert(cap, support);
        self
    }

    pub fn get(&self, cap: Capability) -> Support {
        self.entries.get(&cap).copied().unwrap_or(Support::Unknown)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Capability, &Support)> {
        self.entries.iter()
    }
}
