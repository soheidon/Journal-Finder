use serde::{Deserialize, Serialize};

use crate::core::docx_extract::ManuscriptText;
use crate::core::llm_client::{LlmClient, LlmSlot};
use crate::core::prompts;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SummaryResult {
    pub research_topic: String,
    pub objective: String,
    pub sample_summary: String,
    pub design: String,
    pub methods_summary: String,
    pub measures: String,
    pub statistics: String,
    pub findings: String,
    pub claimed_contributions: String,
    pub keywords_for_search: Vec<String>,
    #[serde(default)]
    pub raw_response: String,
}

pub async fn generate_summary(
    manuscript: &ManuscriptText,
    slot: &LlmSlot,
) -> Result<SummaryResult, String> {
    let messages = prompts::build_summary_messages(manuscript);
    let client = LlmClient::with_timeout(120);
    let raw = client.chat(slot, &messages, 4096).await?;

    match parse_summary_json(&raw) {
        Ok(mut result) => {
            result.raw_response = raw;
            Ok(result)
        }
        Err(_) => {
            let cleaned = try_extract_json_block(&raw);
            match parse_summary_json(&cleaned) {
                Ok(mut result) => {
                    result.raw_response = raw;
                    Ok(result)
                }
                Err(_) => Ok(SummaryResult {
                    research_topic: "(JSON parse failed — see raw response)".to_string(),
                    objective: String::new(),
                    sample_summary: String::new(),
                    design: String::new(),
                    methods_summary: String::new(),
                    measures: String::new(),
                    statistics: String::new(),
                    findings: String::new(),
                    claimed_contributions: String::new(),
                    keywords_for_search: Vec::new(),
                    raw_response: raw,
                }),
            }
        }
    }
}

fn parse_summary_json(text: &str) -> Result<SummaryResult, serde_json::Error> {
    serde_json::from_str(text)
}

fn try_extract_json_block(text: &str) -> String {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return text[start..=end].to_string();
            }
        }
    }
    text.to_string()
}
