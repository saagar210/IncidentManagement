mod commands;
mod db;
mod error;
mod import;
mod models;
mod reports;

#[cfg(test)]
mod security_tests;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            let pool = tauri::async_runtime::block_on(db::init_db(app_data_dir))
                .expect("Failed to initialize database");
            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Incidents
            commands::incidents::create_incident,
            commands::incidents::update_incident,
            commands::incidents::delete_incident,
            commands::incidents::get_incident,
            commands::incidents::list_incidents,
            commands::incidents::search_incidents,
            commands::incidents::bulk_update_status,
            // Action items
            commands::incidents::create_action_item,
            commands::incidents::update_action_item,
            commands::incidents::delete_action_item,
            commands::incidents::list_action_items,
            // Services
            commands::services::create_service,
            commands::services::update_service,
            commands::services::delete_service,
            commands::services::list_services,
            commands::services::list_active_services,
            // Settings
            commands::settings::get_quarter_configs,
            commands::settings::upsert_quarter_config,
            commands::settings::delete_quarter_config,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::export_all_data,
            commands::settings::import_backup,
            // Metrics
            commands::metrics::get_dashboard_data,
            // Reports (Phase 3)
            commands::reports::generate_report,
            commands::reports::save_report,
            commands::reports::generate_discussion_points,
            // Import (Phase 4)
            commands::import::parse_csv_headers,
            commands::import::preview_csv_import,
            commands::import::execute_csv_import,
            commands::import::list_import_templates,
            commands::import::save_import_template,
            commands::import::delete_import_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
