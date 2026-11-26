mod commands;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::recent_events,
            commands::list_notifications,
            commands::acknowledge_notification,
            commands::run_network_diagnostics,
            commands::capability_report,
            commands::list_available_transports,
            commands::active_transport,
            commands::switch_transport,
            commands::export_diagnostics,
            commands::query_events,
            commands::device_activity,
            commands::event_analytics,
            commands::export_events,
            commands::apply_event_retention,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
