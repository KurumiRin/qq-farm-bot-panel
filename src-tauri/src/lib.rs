#![allow(dead_code)]

mod auth;
mod code_receiver;
mod commands;
mod config;
mod error;
mod network;
mod proto;
mod services;
mod state;

use std::sync::Arc;

use commands::TauriState;
use network::NetworkManager;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let app_state = AppState::new();
    let network = NetworkManager::new(Arc::clone(&app_state));

    let tauri_state = TauriState {
        app_state,
        network,
        engine: Arc::new(tokio::sync::Mutex::new(None)),
    };

    // Start HTTP code receiver on port 7788 (uses tauri::async_runtime)
    code_receiver::start(
        Arc::clone(&tauri_state.network),
        Arc::clone(&tauri_state.app_state),
        Arc::clone(&tauri_state.engine),
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(tauri_state)
        .setup(|app| {
            state::set_app_handle(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::request_qr_code,
            commands::check_qr_status,
            commands::request_mp_login_code,
            commands::connect_and_login,
            commands::disconnect,
            commands::get_status,
            commands::get_automation_config,
            commands::set_automation_config,
            commands::get_all_lands,
            commands::harvest,
            commands::water_lands,
            commands::weed_out_lands,
            commands::insecticide_lands,
            commands::remove_dead_plants,
            commands::auto_plant_empty,
            commands::plant_seeds,
            commands::get_friends,
            commands::get_bag,
            commands::sell_all_fruits,
            commands::get_tasks,
            commands::claim_all_tasks,
            commands::get_shop_info,
            commands::get_logs,
            commands::restart_code_receiver,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
