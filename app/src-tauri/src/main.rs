#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod commands;
pub mod core;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::healthcheck,
            commands::get_dummy_manuscript,
            commands::get_dummy_summary,
            commands::get_dummy_journals,
            commands::docx::extract_docx,
            commands::llm::test_llm_connection_from_config,
            commands::llm::fetch_models,
            commands::llm::detect_env_keys,
            commands::llm::load_presets,
            commands::llm::save_presets,
            commands::llm::save_settings,
            commands::llm::load_settings,
            commands::summary::generate_summary,
            commands::journal::get_positioning_prompt,
            commands::journal::get_journal_search_prompt,
            commands::journal::get_search_prompt,
            commands::journal::parse_external_results,
            commands::journal::run_openai_deep_research,
            commands::journal::extract_journal_names,
            commands::journal::parse_single_journal,
            commands::report::generate_report,
            commands::report::export_report,
            commands::report::save_binary_file,
            commands::project::create_project,
            commands::project::open_project,
            commands::project::save_project,
            commands::project::save_project_file,
            commands::project::load_project_file,
            commands::project::copy_to_project,
            commands::project::list_recent_projects,
            commands::project::add_recent_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
