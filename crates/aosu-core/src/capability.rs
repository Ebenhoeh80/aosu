//! Per-feature capability reporting. A Transport never silently pretends to
//! support something it hasn't validated; the UI reads this report to decide
//! what to show as available.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(tag = "type", rename_all = "camelCase")]
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
    /// Cloud-storage playback needs a callback URL Tuya can reach to fetch
    /// the short-lived decryption key (documented HLS API).
    CallbackUrlRequired,
    /// Implementation exists, but no successful call has proved it yet.
    LiveVerificationRequired,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Locks down the exact wire shape the frontend (app/src/types.ts)
    /// is written against -- camelCase keys, `{"type": "...", ...}`
    /// tagging for `Support`. If this test ever needs to change, the
    /// TypeScript types need the matching change in the same commit.
    #[test]
    fn json_shape_matches_what_the_frontend_expects() {
        let mut r = CapabilityReport::new();
        r.set(Capability::Discover, Support::Supported);
        r.set(
            Capability::LiveVideo,
            Support::Blocked {
                reason: BlockedReason::LocalKeyRequired,
            },
        );
        let json = serde_json::to_value(&r).unwrap();
        assert_eq!(json["discover"], serde_json::json!({"type": "supported"}));
        assert_eq!(
            json["liveVideo"],
            serde_json::json!({"type": "blocked", "reason": "LocalKeyRequired"})
        );
    }
}
