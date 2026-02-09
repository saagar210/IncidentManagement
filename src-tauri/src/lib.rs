pub mod ai;
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

            // Initialize Ollama AI state with health check
            let ollama = ai::OllamaState::default();
            let ollama_clone = ollama.clone();
            tauri::async_runtime::spawn(async move {
                ai::client::update_health(&ollama_clone).await;
            });
            app.manage(ollama);

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
            commands::incidents::bulk_delete_incidents,
            // Action items
            commands::incidents::create_action_item,
            commands::incidents::update_action_item,
            commands::incidents::delete_action_item,
            commands::incidents::list_action_items,
            // Services
            commands::services::create_service,
            commands::services::update_service,
            commands::services::delete_service,
            commands::services::get_service,
            commands::services::list_services,
            commands::services::list_active_services,
            commands::services::add_service_dependency,
            commands::services::remove_service_dependency,
            commands::services::list_service_dependencies,
            commands::services::list_service_dependents,
            // Settings
            commands::settings::get_quarter_configs,
            commands::settings::upsert_quarter_config,
            commands::settings::delete_quarter_config,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::export_all_data,
            commands::settings::import_backup,
            // Tags
            commands::incidents::get_incident_tags,
            commands::incidents::set_incident_tags,
            commands::incidents::get_all_tags,
            // Trash / Soft Delete
            commands::incidents::list_deleted_incidents,
            commands::incidents::restore_incident,
            commands::incidents::permanent_delete_incident,
            commands::incidents::count_deleted_incidents,
            commands::incidents::count_overdue_action_items,
            // Custom Fields
            commands::custom_fields::list_custom_fields,
            commands::custom_fields::create_custom_field,
            commands::custom_fields::update_custom_field,
            commands::custom_fields::delete_custom_field,
            commands::custom_fields::get_incident_custom_fields,
            commands::custom_fields::set_incident_custom_fields,
            // Attachments
            commands::attachments::upload_attachment,
            commands::attachments::list_attachments,
            commands::attachments::delete_attachment,
            // Metrics
            commands::metrics::get_dashboard_data,
            commands::metrics::get_incident_heatmap,
            commands::metrics::get_incident_by_hour,
            commands::metrics::get_backlog_aging,
            commands::metrics::get_service_reliability,
            commands::metrics::get_escalation_funnel,
            // Saved Filters
            commands::saved_filters::list_saved_filters,
            commands::saved_filters::create_saved_filter,
            commands::saved_filters::update_saved_filter,
            commands::saved_filters::delete_saved_filter,
            // Reports
            commands::reports::generate_report,
            commands::reports::save_report,
            commands::reports::generate_discussion_points,
            commands::reports::list_report_history,
            commands::reports::delete_report_history_entry,
            commands::reports::generate_narrative,
            // Roles
            commands::roles::assign_role,
            commands::roles::unassign_role,
            commands::roles::list_incident_roles,
            // Checklists
            commands::checklists::create_checklist_template,
            commands::checklists::update_checklist_template,
            commands::checklists::delete_checklist_template,
            commands::checklists::list_checklist_templates,
            commands::checklists::create_incident_checklist,
            commands::checklists::list_incident_checklists,
            commands::checklists::delete_incident_checklist,
            commands::checklists::toggle_checklist_item,
            // Audit & Notifications
            commands::audit::list_audit_entries,
            commands::audit::get_notification_summary,
            // SLA
            commands::sla::list_sla_definitions,
            commands::sla::create_sla_definition,
            commands::sla::update_sla_definition,
            commands::sla::delete_sla_definition,
            commands::sla::compute_sla_status,
            // Import (Phase 4)
            commands::import::parse_csv_headers,
            commands::import::preview_csv_import,
            commands::import::execute_csv_import,
            commands::import::list_import_templates,
            commands::import::save_import_template,
            commands::import::delete_import_template,
            // Post-mortems
            commands::postmortems::list_contributing_factors,
            commands::postmortems::create_contributing_factor,
            commands::postmortems::delete_contributing_factor,
            commands::postmortems::list_postmortem_templates,
            commands::postmortems::get_postmortem_by_incident,
            commands::postmortems::create_postmortem,
            commands::postmortems::update_postmortem,
            commands::postmortems::delete_postmortem,
            commands::postmortems::list_postmortems,
            commands::postmortems::get_postmortem_readiness,
            // AI
            commands::ai::get_ai_status,
            commands::ai::check_ai_health,
            commands::ai::ai_summarize_incident,
            commands::ai::ai_stakeholder_update,
            commands::ai::ai_postmortem_draft,
            commands::ai::find_similar_incidents,
            commands::ai::ai_suggest_root_causes,
            commands::ai::check_duplicate_incidents,
            commands::ai::detect_service_trends,
            // Stakeholder Updates
            commands::stakeholder_updates::list_stakeholder_updates,
            commands::stakeholder_updates::create_stakeholder_update,
            commands::stakeholder_updates::delete_stakeholder_update,
            // Shift Handoffs
            commands::shift_handoffs::list_shift_handoffs,
            commands::shift_handoffs::create_shift_handoff,
            commands::shift_handoffs::delete_shift_handoff,
            // Export
            commands::export::export_incidents_csv,
            commands::export::export_incidents_json,
            // Backup
            commands::backup::create_backup,
            commands::backup::list_backups,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
