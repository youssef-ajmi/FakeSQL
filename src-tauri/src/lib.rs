pub mod commands;
pub mod engine;
pub mod state;

use state::WorkspaceState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(WorkspaceState::default())
        .invoke_handler(tauri::generate_handler![
            commands::open_workspace,
            commands::get_workspace,
            commands::list_tables,
            commands::run_query,
            commands::workspace_issues,
            commands::validate_query,
            commands::create_table,
            commands::delete_table,
            commands::export_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FakeSQL Studio");
}
