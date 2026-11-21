//! Tauri commands: the only place the UI talks to `aosu-core`. Kept thin
//! on purpose -- all real logic lives in the core crate so it stays
//! testable with plain `cargo test`, no Tauri runtime required.

use aosu_core::{
    build_bundle, scan_ports, Device, Event, Notification, Timestamp, TransportConfig,
    TransportRegistry, DEFAULT_PROBE_PORTS,
};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;

pub struct AppState {
    pub registry: TransportRegistry,
    pub transport: Mutex<Box<dyn aosu_core::Transport>>,
    pub active_transport_name: Mutex<String>,
    /// Notification acknowledgement is an app-level concern, not
    /// something any transport tracks -- see docs/architecture.md.
    pub acknowledged_notifications: Mutex<HashSet<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        let registry = TransportRegistry::with_builtins();
        let transport = registry
            .build("mock", &TransportConfig::default())
            .expect("the built-in \"mock\" transport is always registered");
        Self {
            registry,
            transport: Mutex::new(transport),
            active_transport_name: Mutex::new("mock".to_string()),
            acknowledged_notifications: Mutex::new(HashSet::new()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDto {
    id: String,
    name: String,
    vendor: String,
    model: String,
    kind: String,
    online: bool,
    health: String,
    battery_percent: Option<u8>,
    wifi_signal_percent: Option<u8>,
    firmware_version: Option<String>,
}

impl From<Device> for DeviceDto {
    fn from(d: Device) -> Self {
        Self {
            id: d.id.0,
            name: d.name,
            vendor: d.vendor,
            model: d.model,
            kind: d.kind,
            online: d.online,
            health: format!("{:?}", d.health),
            battery_percent: d.battery_percent,
            wifi_signal_percent: d.wifi_signal_percent,
            firmware_version: d.firmware_version,
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
#[serde(rename_all = "camelCase")]
pub struct EventDto {
    device: String,
    kind: String,
    severity: String,
    at_epoch_seconds: u64,
}

impl From<Event> for EventDto {
    fn from(e: Event) -> Self {
        Self {
            device: e.device.0,
            kind: format!("{:?}", e.kind),
            severity: format!("{:?}", e.severity),
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
#[serde(rename_all = "camelCase")]
pub struct NotificationDto {
    id: String,
    device: Option<String>,
    severity: String,
    title: String,
    body: String,
    at_epoch_seconds: u64,
    acknowledged: bool,
}

fn to_notification_dto(n: Notification, acked: &HashSet<String>) -> NotificationDto {
    let acknowledged = n.acknowledged || acked.contains(&n.id);
    NotificationDto {
        id: n.id,
        device: n.device.map(|d| d.0),
        severity: format!("{:?}", n.severity),
        title: n.title,
        body: n.body,
        at_epoch_seconds: n.at.0,
        acknowledged,
    }
}

#[tauri::command]
pub fn list_notifications(state: State<AppState>) -> Result<Vec<NotificationDto>, String> {
    let transport = state.transport.lock().map_err(|e| e.to_string())?;
    let acked = state
        .acknowledged_notifications
        .lock()
        .map_err(|e| e.to_string())?;
    transport
        .notifications()
        .map(|ns| {
            ns.into_iter()
                .map(|n| to_notification_dto(n, &acked))
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn acknowledge_notification(state: State<AppState>, id: String) -> Result<(), String> {
    state
        .acknowledged_notifications
        .lock()
        .map_err(|e| e.to_string())?
        .insert(id);
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortCheckDto {
    port: u16,
    open: bool,
}

#[tauri::command]
pub fn run_network_diagnostics(host: String) -> Vec<PortCheckDto> {
    let result = scan_ports(&host, DEFAULT_PROBE_PORTS, Duration::from_secs(2));
    result
        .ports
        .into_iter()
        .map(|p| PortCheckDto {
            port: p.port,
            open: p.open,
        })
        .collect()
}

#[tauri::command]
pub fn list_available_transports(state: State<AppState>) -> Vec<String> {
    let mut names = state.registry.names();
    names.sort();
    names.into_iter().map(str::to_string).collect()
}

#[tauri::command]
pub fn active_transport(state: State<AppState>) -> Result<String, String> {
    state
        .active_transport_name
        .lock()
        .map(|n| n.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn switch_transport(
    state: State<AppState>,
    name: String,
    host: Option<String>,
) -> Result<(), String> {
    let config = match host {
        Some(h) => TransportConfig::with_host(h),
        None => TransportConfig::default(),
    };
    let built = state
        .registry
        .build(&name, &config)
        .ok_or_else(|| format!("unknown transport \"{name}\" (see list_available_transports)"))?;
    *state.transport.lock().map_err(|e| e.to_string())? = built;
    *state
        .active_transport_name
        .lock()
        .map_err(|e| e.to_string())? = name;
    Ok(())
}

/// Export everything the Diagnostics Center shows, as one JSON document
/// -- the whole point being a user can attach this to a bug report
/// without us ever seeing their screen. `network_host`, if given, also
/// runs the same read-only port scan as `run_network_diagnostics`.
#[tauri::command]
pub fn export_diagnostics(
    state: State<AppState>,
    network_host: Option<String>,
) -> Result<String, String> {
    let transport = state.transport.lock().map_err(|e| e.to_string())?;
    let active_name = state
        .active_transport_name
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let network = network_host.map(|h| scan_ports(&h, DEFAULT_PROBE_PORTS, Duration::from_secs(2)));
    let bundle = build_bundle(&active_name, transport.capability_report(), network);
    serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())
}
