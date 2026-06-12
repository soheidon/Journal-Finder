use serde::{Deserialize, Serialize};

use crate::core::llm_client::{LlmClient, LlmSlot};
use crate::core::prompts;
use crate::core::summary::SummaryResult;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JournalCandidate {
    pub journal_name: String,
    pub publisher: String,
    pub scope_fit: String,
    pub article_type_fit: String,
    pub similar_articles: String,
    pub impact_factor_or_metric: String,
    pub apc: String,
    pub word_limit: String,
    pub open_access_policy: String,
    pub pros: String,
    pub cons: String,
    pub recommendation_level: String,
    pub reason: String,
    pub source_evidence: String,
    #[serde(default)]
    pub match_score: u8,
    #[serde(default)]
    pub publication_route: String,
    #[serde(default)]
    pub apc_required: String,
    #[serde(default)]
    pub apc_avoidance: String,
    #[serde(default)]
    pub recommended_submission_strategy: String,
    #[serde(default)]
    pub waiver_or_discount_info: String,
    #[serde(default)]
    pub cost_risk_level: String,
    #[serde(default)]
    pub raw_response: String,
}

pub fn generate_search_prompt(summary: &SummaryResult) -> String {
    prompts::build_deep_research_prompt(summary)
}

pub async fn parse_external_results(
    summary: &SummaryResult,
    external_a: &str,
    external_b: Option<&str>,
    slot: &LlmSlot,
) -> Result<Vec<JournalCandidate>, String> {
    let messages = prompts::build_parse_external_messages(summary, external_a, external_b);
    let client = LlmClient::with_timeout(180);
    let raw = client.chat(slot, &messages, 8192).await?;

    match parse_journal_candidates_json(&raw) {
        Ok(mut candidates) => {
            for c in &mut candidates {
                c.raw_response = raw.clone();
                normalize_candidate(c);
            }
            Ok(candidates)
        }
        Err(_) => {
            let cleaned = try_extract_json_array(&raw);
            match parse_journal_candidates_json(&cleaned) {
                Ok(mut candidates) => {
                    for c in &mut candidates {
                        c.raw_response = raw.clone();
                        normalize_candidate(c);
                    }
                    Ok(candidates)
                }
                Err(_) => Ok(vec![JournalCandidate {
                    journal_name: "(JSON parse failed — see raw response)".to_string(),
                    publisher: String::new(),
                    scope_fit: String::new(),
                    article_type_fit: String::new(),
                    similar_articles: String::new(),
                    impact_factor_or_metric: String::new(),
                    apc: String::new(),
                    word_limit: String::new(),
                    open_access_policy: String::new(),
                    pros: String::new(),
                    cons: String::new(),
                    recommendation_level: String::new(),
                    reason: String::new(),
                    source_evidence: String::new(),
                    match_score: 0,
                    publication_route: String::new(),
                    apc_required: String::new(),
                    apc_avoidance: String::new(),
                    recommended_submission_strategy: String::new(),
                    waiver_or_discount_info: String::new(),
                    cost_risk_level: String::new(),
                    raw_response: raw,
                }]),
            }
        }
    }
}

fn normalize_candidate(c: &mut JournalCandidate) {
    if c.match_score == 0 {
        if let Some(num) = parse_match_score(&c.recommendation_level) {
            c.match_score = num;
        }
    }
    if c.apc_required.is_empty() {
        c.apc_required = infer_apc_required(&c.apc, &c.open_access_policy);
    }
    if c.cost_risk_level.is_empty() {
        c.cost_risk_level = infer_cost_risk(&c.apc_required, &c.apc);
    }
}

fn parse_match_score(rec: &str) -> Option<u8> {
    let lower = rec.to_lowercase();
    if lower.contains("strong") { Some(85) }
    else if lower.contains("moderate") { Some(60) }
    else if lower.contains("weak") { Some(35) }
    else { None }
}

fn infer_apc_required(apc: &str, oa_policy: &str) -> String {
    let apc_lower = apc.to_lowercase();
    let oa_lower = oa_policy.to_lowercase();
    if apc_lower.contains("free") || apc_lower == "0" || apc_lower.contains("no apc") {
        "no_apc".to_string()
    } else if oa_lower.contains("gold") && !oa_lower.contains("hybrid") {
        "required".to_string()
    } else if oa_lower.contains("hybrid") {
        "optional".to_string()
    } else if oa_lower.contains("subscription") {
        "no_apc".to_string()
    } else {
        "unknown".to_string()
    }
}

fn infer_cost_risk(apc_required: &str, apc: &str) -> String {
    match apc_required {
        "no_apc" => "low".to_string(),
        "optional" => "low".to_string(),
        "required" => {
            if apc.to_lowercase().contains("waiver") {
                "medium".to_string()
            } else {
                "high".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

fn parse_journal_candidates_json(text: &str) -> Result<Vec<JournalCandidate>, serde_json::Error> {
    serde_json::from_str(text)
}

fn try_extract_json_array(text: &str) -> String {
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            if end > start {
                return text[start..=end].to_string();
            }
        }
    }
    text.to_string()
}
