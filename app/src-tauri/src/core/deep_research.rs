use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer};

use crate::core::llm_client::{LlmClient, LlmSlot};
use crate::core::prompts;
use crate::core::summary::SummaryResult;

fn deserialize_match_score<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    struct MatchScoreVisitor;

    impl<'de> de::Visitor<'de> for MatchScoreVisitor {
        type Value = u8;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number 0-100 or a string like '96', '96%'")
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<u8, E> {
            Ok(v.min(100) as u8)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<u8, E> {
            Ok(v.clamp(0, 100) as u8)
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<u8, E> {
            Ok(v.clamp(0.0, 100.0) as u8)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<u8, E> {
            let cleaned = v.trim().trim_end_matches('%').trim();
            match cleaned.parse::<f64>() {
                Ok(n) => Ok(n.clamp(0.0, 100.0) as u8),
                Err(_) => Ok(0),
            }
        }

        fn visit_none<E: de::Error>(self) -> Result<u8, E> {
            Ok(0)
        }

        fn visit_unit<E: de::Error>(self) -> Result<u8, E> {
            Ok(0)
        }
    }

    deserializer.deserialize_any(MatchScoreVisitor)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JournalCandidate {
    pub journal_name: String,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub scope_fit: String,
    #[serde(default)]
    pub article_type_fit: String,
    #[serde(default)]
    pub similar_articles: String,
    #[serde(default)]
    pub impact_factor_or_metric: String,
    #[serde(default)]
    pub quartile_or_rank: String,
    #[serde(default)]
    pub metric_source: String,
    #[serde(default)]
    pub metric_year: String,
    #[serde(default)]
    pub apc: String,
    #[serde(default)]
    pub word_limit: String,
    #[serde(default)]
    pub open_access_policy: String,
    #[serde(default)]
    pub pros: String,
    #[serde(default)]
    pub cons: String,
    #[serde(default)]
    pub recommendation_level: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub source_evidence: String,
    #[serde(default, deserialize_with = "deserialize_match_score")]
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
                Err(e) => Ok(vec![JournalCandidate {
                    journal_name: format!("(JSON parse failed: {})", e),
                    publisher: String::new(),
                    scope_fit: String::new(),
                    article_type_fit: String::new(),
                    similar_articles: String::new(),
                    impact_factor_or_metric: String::new(),
                    quartile_or_rank: String::new(),
                    metric_source: String::new(),
                    metric_year: String::new(),
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

pub async fn extract_journal_names(
    journal_research: &str,
    slot: &LlmSlot,
) -> Result<Vec<String>, String> {
    let messages = prompts::build_extract_journal_names_messages(journal_research);
    let client = LlmClient::with_timeout(120);
    let raw = client.chat(slot, &messages, 4096).await?;

    // Try to parse as JSON array of strings
    let cleaned = try_extract_json_array(&raw);
    match serde_json::from_str::<Vec<String>>(&cleaned) {
        Ok(names) => Ok(names),
        Err(_) => {
            // Fallback: try to extract from markdown list
            let names: Vec<String> = raw.lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    // Match "1. Journal Name" or "- Journal Name" patterns
                    if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit() || c == '-' || c == '*') {
                        let name = rest.trim_start_matches(|c: char| c == '.' || c == ' ' || c == '\t').trim();
                        if !name.is_empty() && name.len() > 3 {
                            return Some(name.to_string());
                        }
                    }
                    None
                })
                .collect();
            if names.is_empty() {
                Err(format!("Failed to extract journal names. Raw response: {}", truncate_str(&raw, 500)))
            } else {
                Ok(names)
            }
        }
    }
}

pub async fn parse_single_journal(
    journal_name: &str,
    journal_research: &str,
    positioning_report: &str,
    summary: &SummaryResult,
    slot: &LlmSlot,
) -> Result<JournalCandidate, String> {
    let messages = prompts::build_single_journal_parse_messages(
        journal_name, journal_research, positioning_report, summary,
    );
    let client = LlmClient::with_timeout(120);
    let raw = client.chat(slot, &messages, 4096).await?;

    // Try to parse JSON object
    let cleaned = try_extract_json_object(&raw);
    match serde_json::from_str::<JournalCandidate>(&cleaned) {
        Ok(mut candidate) => {
            candidate.raw_response = raw;
            normalize_candidate(&mut candidate);
            Ok(candidate)
        }
        Err(e) => Err(format!("JSON parse failed for '{}': {}. Raw: {}", journal_name, e, truncate_str(&raw, 300))),
    }
}

fn try_extract_json_object(text: &str) -> String {
    // Try markdown code block first
    if let Some(extracted) = extract_from_code_block(text) {
        if extracted.starts_with('{') {
            return extracted;
        }
    }
    // Try finding { ... }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return text[start..=end].to_string();
            }
        }
    }
    text.to_string()
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
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
    // 1. Try direct parse
    if let Ok(candidates) = serde_json::from_str::<Vec<JournalCandidate>>(text) {
        return Ok(candidates);
    }

    // 2. Try extracting from markdown code block
    if let Some(extracted) = extract_from_code_block(text) {
        if let Ok(candidates) = serde_json::from_str::<Vec<JournalCandidate>>(&extracted) {
            return Ok(candidates);
        }
    }

    // 3. Try extracting JSON array from text
    let array_text = try_extract_json_array(text);
    serde_json::from_str(&array_text)
}

fn extract_from_code_block(text: &str) -> Option<String> {
    // Find ```json ... ``` or ``` ... ```
    let start_markers = ["```json\n", "```json\r\n", "```\n", "```\r\n"];
    for marker in &start_markers {
        if let Some(start_idx) = text.find(marker) {
            let content_start = start_idx + marker.len();
            if let Some(end_idx) = text[content_start..].find("```") {
                let content = &text[content_start..content_start + end_idx];
                let trimmed = content.trim();
                if trimmed.starts_with('[') || trimmed.starts_with('{') {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
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
