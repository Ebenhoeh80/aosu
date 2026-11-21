//! Core, UI-independent library for the Aosu/H1L desktop client.
//! See `docs/architecture.md` for the layer design this crate implements.

pub mod capability;
pub mod device;
pub mod diagnostics;
pub mod registry;
pub mod transport;
pub mod transports;

pub use capability::{BlockedReason, Capability, CapabilityReport, Support};
pub use device::*;
pub use registry::{TransportConfig, TransportRegistry};
pub use transport::{Result, Transport, TransportError};
