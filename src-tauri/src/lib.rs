use tauri::{Manager, WindowEvent};

mod commands;
mod logger;

use commands::{
    clear_profile_log, clear_system_log, delete_profile_config, get_profile_log,
    get_running_profile, get_system_log, log_message, measure_ping, start_vpn, stop_vpn,
    VpnProcessState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(VpnProcessState(std::sync::Mutex::new(None)))
        .setup(|_app| {
            match logger::init_app_logger() {
                logger::InitResult::Ok { log_file_path } => {
                    log::info!(target: "app_lib", "Logger initialised. Log file: {}. Level: {}",
                        log_file_path.display(),
                        if cfg!(debug_assertions) { "DEBUG" } else { "INFO" }
                    );
                }
                logger::InitResult::FallbackToStdout { reason } => {
                    eprintln!(
                        "WARN: App_Logger falling back to stdout-only. Reason: {}",
                        reason
                    );
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_vpn,
            stop_vpn,
            clear_profile_log,
            clear_system_log,
            get_profile_log,
            get_system_log,
            get_running_profile,
            delete_profile_config,
            measure_ping,
            log_message,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::Destroyed = event {
                let state = window.state::<VpnProcessState>();

                match stop_vpn(state) {
                    Ok(msg) => println!("Auto-stop on exit: {}", msg),
                    Err(err) => println!("Auto-stop on exit log/error: {}", err),
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
