//! Tauri commands: the only place the UI talks to `aosu-core`. Kept thin
//! on purpose -- all real logic lives in the core crate so it stays
//! testable with plain `cargo test`, no Tauri runtime required.

use aosu_core::{Device, DeviceId, Event, Timestamp, TransportConfig, TransportRegistry};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub transport: Mutex<Box<dyn aosu_core::Transport>>,
}

impl Default for AppState {
    fn default() -> Self {
        let registry = TransportRegistry::with_builtins();
        let transport = registry
            .build("mock", &TransportConfig::default())
            .expect("the built-in \"mock\" transport is always registered");
        Self {
            transport: Mutex::new(transport),
        }
    }
}

#[derive(Serialize)]
pub struct DeviceDto {
    id: String,
    name: String,
    model: String,
    kind: String,
    online: bool,
    battery_percent: Option<u8>,
    wifi_signal_percent: Option<u8>,
}

impl From<Device> for DeviceDto {
    fn from(d: Device) -> Self {
        Self {
            id: d.id.0,
            name: d.name,
            model: d.model,
            kind: d.kind,
            online: d.online,
            battery_percent: d.battery_percent,
            wifi_signal_percent: d.wifi_signal_percent,
        }
    }
}

#[tauri::command]
pub fn list_devices(state: State<AppState>) -> Result<Vec<DeviceDto>, String> {
    let transport = state.transport.lock().map_err(|e| e.to_string())?;
    transport
        .devices()
        .map(|ds| ds.into_iter().map(DeviceDto::from).collect())
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct EventDto {
    device: String,
    kind: String,
    at_epoch_seconds: u64,
}

impl From<Event> for EventDto {
    fn from(e: Event) -> Self {
        Self {
            device: e.device.0,
            kind: format!("{:?}", e.kind),
            at_epoch_seconds: e.at.0,
        }
    }
}

#[tauri::command]
pub fn recent_events(state: State<AppState>) -> Result<Vec<EventDto>, String> {
    let transport = state.transport.lock().map_err(|e| e.to_string())?;
    transport
        .events(Timestamp(0))
        .map(|es| es.into_iter().map(EventDto::from).collect())
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct PortCheckDto {
    port: u16,
    open: bool,
}

#[tauri::command]
pub fn run_network_diagnostics(host: String) -> Vec<PortCheckDto> {
    let result = aosu_core::diagnostics::scan_ports(
        &host,
        aosu_core::diagnostics::DEFAULT_PROBE_PORTS,
        std::time::Duration::from_secs(2),
    );
    result
        .ports
        .into_iter()
        .map(|p| PortCheckDto {
            port: p.port,
            open: p.open,
        })
        .collect()
}

#[allow(dead_code)]
fn _unused(_: DeviceId) {}
