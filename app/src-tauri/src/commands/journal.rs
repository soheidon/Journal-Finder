use crate::commands::llm::load_settings_inner;
use crate::core::deep_research;
use crate::core::deep_research_api;
use crate::core::llm_client::LlmSlot;
use crate::core::prompts;
use crate::core::summary::SummaryResult;

fn get_assessor_slot() -> Result<LlmSlot, String> {
    let slots = load_settings_inner();
    let config = slots
        .iter()
        .find(|s| s.name == "analysis_llm")
        .ok_or("analysis_llm スロットが設定されていません。設定画面で LLM を設定してください。")?;

    let api_key = config.resolve_api_key()?;
    Ok(config.to_slot(api_key))
}

fn get_deep_research_slot() -> Result<LlmSlot, String> {
    let slots = load_settings_inner();
    let config = slots
        .iter()
        .find(|s| s.name == "deep_research_provider")
        .ok_or("deep_research_provider スロットが設定されていません。設定画面で Deep Research API を設定してください。")?;

    let api_key = config.resolve_api_key()?;
    Ok(config.to_slot(api_key))
}

#[tauri::command]
pub fn get_positioning_prompt(summary: SummaryResult) -> Result<String, String> {
    Ok(prompts::build_positioning_prompt(&summary))
}

#[tauri::command]
pub fn get_journal_search_prompt(summary: SummaryResult, positioning: String) -> Result<String, String> {
    Ok(prompts::build_journal_search_prompt(&summary, &positioning))
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

#[tauri::command]
pub async fn run_openai_deep_research(
    summary: SummaryResult,
    positioning_report: String,
    task_type: String,
) -> Result<deep_research_api::DeepResearchApiResult, String> {
    let slot = get_deep_research_slot()?;
    let prompt = deep_research_api::build_prompt(&task_type, &summary, &positioning_report);
    Ok(deep_research_api::run_openai_deep_research(&slot.api_key, &slot.model, &prompt).await)
}
