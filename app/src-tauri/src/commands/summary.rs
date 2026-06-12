use crate::commands::llm::load_settings_inner;
use crate::core::docx_extract::ManuscriptText;
use crate::core::summary;

#[tauri::command]
pub async fn generate_summary(manuscript: ManuscriptText) -> Result<summary::SummaryResult, String> {
    let slots = load_settings_inner();
    let config = slots
        .iter()
        .find(|s| s.name == "analysis_llm")
        .ok_or("analysis_llm スロットが設定されていません。設定画面で LLM を設定してください。")?;

    let api_key = config.resolve_api_key()?;
    let slot = config.to_slot(api_key);

    summary::generate_summary(&manuscript, &slot).await
}
