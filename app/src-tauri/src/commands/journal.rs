use crate::commands::llm::load_settings_inner;
use crate::core::deep_research;
use crate::core::summary::SummaryResult;

fn get_assessor_slot() -> Result<crate::core::llm_client::LlmSlot, String> {
    let slots = load_settings_inner();
    let config = slots
        .iter()
        .find(|s| s.name == "journal_assessor")
        .ok_or("journal_assessor スロットが設定されていません。設定画面で LLM を設定してください。")?;

    let api_key = config.resolve_api_key()?;
    Ok(config.to_slot(api_key))
}

#[tauri::command]
pub fn get_search_prompt(summary: SummaryResult) -> Result<String, String> {
    Ok(deep_research::generate_search_prompt(&summary))
}

#[tauri::command]
pub async fn parse_external_results(
    summary: SummaryResult,
    external_a: String,
    external_b: Option<String>,
) -> Result<Vec<deep_research::JournalCandidate>, String> {
    let slot = get_assessor_slot()?;
    deep_research::parse_external_results(&summary, &external_a, external_b.as_deref(), &slot).await
}
