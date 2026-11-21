mod commands;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::recent_events,
            commands::run_network_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
