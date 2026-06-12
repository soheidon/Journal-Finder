use crate::core::deep_research::JournalCandidate;
use crate::core::report;
use crate::core::summary::SummaryResult;
use base64::Engine;
use std::fs;

#[tauri::command]
pub fn generate_report(
    summary: SummaryResult,
    journals: Vec<JournalCandidate>,
) -> Result<report::ReportData, String> {
    Ok(report::generate_report(&summary, &journals))
}

#[tauri::command]
pub fn export_report(path: String, content: String) -> Result<(), String> {
    if let Some(parent) = std::path::Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    fs::write(&path, &content).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn save_binary_file(path: String, base64_content: String) -> Result<(), String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&base64_content)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    if let Some(parent) = std::path::Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    fs::write(&path, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}
