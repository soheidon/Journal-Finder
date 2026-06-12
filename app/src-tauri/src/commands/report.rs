use crate::core::deep_research::JournalCandidate;
use crate::core::report;
use crate::core::summary::SummaryResult;
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
