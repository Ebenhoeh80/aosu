//! Tauri commands: the only place the UI talks to `aosu-core`. Kept thin
//! on purpose -- all real logic lives in the core crate so it stays
//! testable with plain `cargo test`, no Tauri runtime required.

use aosu_core::{
    build_bundle, scan_ports, Device, DeviceActivity, Event, EventAnalytics, EventQuery,
    Notification, StoredEvent, Timestamp, TransportConfig, TransportRegistry, DEFAULT_PROBE_PORTS,
};
use serde::{Deserialize, Serialize};
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
            kind: e.kind.wire_name(),
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
pub fn capability_report(state: State<AppState>) -> Result<aosu_core::CapabilityReport, String> {
    state
        .transport
        .lock()
        .map(|t| t.capability_report())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn active_transport(state: State<AppState>) -> Result<String, String> {
    state
        .active_transport_name
        .lock()
        .map(|n| n.clone())
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SwitchTransportConfigDto {
    pub host: Option<String>,
    pub path: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub apk_path: Option<String>,
}

impl SwitchTransportConfigDto {
    /// Pure DTO mapping, kept separate so it can be tested without a Tauri
    /// runtime. Empty strings are treated as "not configured" so switching
    /// transports does not carry stale values from another transport's form.
    fn into_transport_config(self) -> TransportConfig {
        let mut cfg = TransportConfig::default();
        if let Some(h) = self.host {
            if !h.is_empty() {
                cfg.host = Some(h);
            }
        }
        if let Some(p) = self.path {
            if !p.is_empty() {
                cfg.options.insert("path".to_string(), p);
            }
        }
        if let Some(v) = self.email {
            if !v.is_empty() {
                cfg.options.insert("email".to_string(), v);
            }
        }
        if let Some(v) = self.password {
            if !v.is_empty() {
                cfg.options.insert("password".to_string(), v);
            }
        }
        if let Some(v) = self.apk_path {
            if !v.is_empty() {
                cfg.options.insert("apkPath".to_string(), v);
            }
        }
        cfg
    }
}

#[tauri::command]
pub fn switch_transport(
    state: State<AppState>,
    name: String,
    config: Option<SwitchTransportConfigDto>,
) -> Result<(), String> {
    let cfg = config.unwrap_or_default().into_transport_config();

    let built = state
        .registry
        .build(&name, &cfg)
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

// -- Event Platform: persistent storage/query/analytics/export/retention.
// Only the active transport's own support matters here (today: only
// "push-log"); every other transport's TransportError::Unsupported is
// surfaced as a plain string, the UI already has an ErrorNotice pattern
// for that (see docs/event-pipeline.md, docs/ROADMAP.md).

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventQueryDto {
    pub camera_name: Option<String>,
    pub event_type: Option<String>,
    pub since_ms: Option<i64>,
    pub until_ms: Option<i64>,
    pub search_text: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

impl From<EventQueryDto> for EventQuery {
    fn from(d: EventQueryDto) -> Self {
        Self {
            camera_name: d.camera_name,
            event_type: d.event_type,
            since_ms: d.since_ms,
            until_ms: d.until_ms,
            search_text: d.search_text,
            limit: d.limit,
            offset: d.offset,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredEventDto {
    id: String,
    timestamp_ms: i64,
    event_type: String,
    channel: String,
    camera_name: Option<String>,
    confidence: String,
    classification_note: Option<String>,
    raw_json: String,
}

impl From<StoredEvent> for StoredEventDto {
    fn from(e: StoredEvent) -> Self {
        Self {
            id: e.id,
            timestamp_ms: e.timestamp_ms,
            event_type: e.event_type,
            channel: e.channel,
            camera_name: e.camera_name,
            confidence: e.confidence,
            classification_note: e.classification_note,
            raw_json: e.raw_json,
        }
    }
}

#[tauri::command]
pub fn query_events(
    state: State<AppState>,
    query: EventQueryDto,
) -> Result<Vec<StoredEventDto>, String> {
    let transport = state.transport.lock().map_err(|e| e.to_string())?;
    transport
        .query_events(&query.into())
        .map(|rows| rows.into_iter().map(StoredEventDto::from).collect())
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceActivityDto {
    camera_name: String,
    event_count: i64,
    distinct_event_types: Vec<String>,
    last_seen_ms: i64,
}

impl From<DeviceActivity> for DeviceActivityDto {
    fn from(a: DeviceActivity) -> Self {
        Self {
            camera_name: a.camera_name,
            event_count: a.event_count,
            distinct_event_types: a.distinct_event_types,
            last_seen_ms: a.last_seen_ms,
        }
    }
}

#[tauri::command]
pub fn device_activity(state: State<AppState>) -> Result<Vec<DeviceActivityDto>, String> {
    let transport = state.transport.lock().map_err(|e| e.to_string())?;
    transport
        .device_activity()
        .map(|rows| rows.into_iter().map(DeviceActivityDto::from).collect())
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventAnalyticsDto {
    events_today: i64,
    by_camera: Vec<(String, i64)>,
    by_type: Vec<(String, i64)>,
    daily_counts_last_14_days: Vec<(String, i64)>,
}

impl From<EventAnalytics> for EventAnalyticsDto {
    fn from(a: EventAnalytics) -> Self {
        Self {
            events_today: a.events_today,
            by_camera: a.by_camera,
            by_type: a.by_type,
            daily_counts_last_14_days: a.daily_counts_last_14_days,
        }
    }
}

#[tauri::command]
pub fn event_analytics(state: State<AppState>) -> Result<EventAnalyticsDto, String> {
    let transport = state.transport.lock().map_err(|e| e.to_string())?;
    transport
        .event_analytics(now_ms())
        .map(EventAnalyticsDto::from)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_events(
    state: State<AppState>,
    query: EventQueryDto,
    format: String,
) -> Result<String, String> {
    let fmt = match format.as_str() {
        "csv" => aosu_core::ExportFormat::Csv,
        _ => aosu_core::ExportFormat::Json,
    };
    let transport = state.transport.lock().map_err(|e| e.to_string())?;
    transport
        .export_events(&query.into(), fmt)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_event_retention(
    state: State<AppState>,
    max_age_days: Option<u32>,
    max_rows: Option<i64>,
) -> Result<i64, String> {
    let transport = state.transport.lock().map_err(|e| e.to_string())?;
    transport
        .apply_event_retention(now_ms(), max_age_days, max_rows)
        .map_err(|e| e.to_string())
}
