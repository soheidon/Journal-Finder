use crate::core::docx_extract;

#[tauri::command]
pub fn extract_docx(path: String) -> Result<docx_extract::ManuscriptText, String> {
    docx_extract::extract_docx(&path)
}
