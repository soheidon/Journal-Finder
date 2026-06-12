use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeepResearchApiResult {
    pub ok: bool,
    pub raw_report: String,
    pub model_used: String,
    pub output_tokens: u32,
    pub message: String,
}

// TODO: Future improvements for long-running Deep Research:
// - Background mode: submit request, poll for completion
// - Streaming: SSE-based real-time output
// - Cancellation support
// For now, synchronous execution with 600s timeout.

pub fn build_prompt(
    task_type: &str,
    summary: &crate::core::summary::SummaryResult,
    positioning_report: &str,
) -> String {
    match task_type {
        "positioning" => crate::core::prompts::build_positioning_api_prompt(summary),
        "journal_search" => crate::core::prompts::build_deep_research_api_prompt(summary, positioning_report),
        _ => crate::core::prompts::build_deep_research_api_prompt(summary, positioning_report),
    }
}

pub async fn run_openai_deep_research(
    api_key: &str,
    model: &str,
    prompt: &str,
) -> DeepResearchApiResult {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
    {
        Ok(c) => c,
        Err(e) => return DeepResearchApiResult {
            ok: false, raw_report: String::new(), model_used: model.to_string(),
            output_tokens: 0, message: format!("HTTP client error: {}", e),
        },
    };

    let url = "https://api.openai.com/v1/responses";

    let body = serde_json::json!({
        "model": model,
        "input": prompt,
        "tools": [{"type": "web_search_preview"}],
        "max_output_tokens": 16384,
    });

    let resp = match client.post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return DeepResearchApiResult {
            ok: false, raw_report: String::new(), model_used: model.to_string(),
            output_tokens: 0, message: format!("HTTP request failed: {}", e),
        },
    };

    let status = resp.status();
    let status_code = status.as_u16();
    let text = match resp.text().await {
        Ok(t) => t,
        Err(e) => return DeepResearchApiResult {
            ok: false, raw_report: String::new(), model_used: model.to_string(),
            output_tokens: 0, message: format!("Failed to read response: {}", e),
        },
    };

    if !status.is_success() {
        return DeepResearchApiResult {
            ok: false, raw_report: text.clone(), model_used: model.to_string(),
            output_tokens: 0,
            message: format!("HTTP {}: {}", status_code, truncate(&text, 500)),
        };
    }

    // Parse Responses API output
    let data: serde_json::Value = match serde_json::from_str(&text) {
        Ok(d) => d,
        Err(e) => return DeepResearchApiResult {
            ok: false, raw_report: text.clone(), model_used: model.to_string(),
            output_tokens: 0, message: format!("JSON parse error: {}", e),
        },
    };

    let model_used = data.get("model").and_then(|m| m.as_str()).unwrap_or(model).to_string();

    let output_tokens = data.get("usage")
        .and_then(|u| u.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    // Extract text from Responses API output
    let raw_report = extract_responses_text(&data);

    if raw_report.is_empty() {
        // Show response structure for debugging (without API key)
        let debug_text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| text.clone())
            .chars().take(2000)
            .collect::<String>();
        DeepResearchApiResult {
            ok: false, raw_report: debug_text, model_used,
            output_tokens, message: "No text content in response. See raw response for structure.".to_string(),
        }
    } else {
        DeepResearchApiResult {
            ok: true, raw_report, model_used,
            output_tokens, message: "Deep Research completed".to_string(),
        }
    }
}

fn extract_responses_text(data: &serde_json::Value) -> String {
    // 1. Try output_text (simplest Responses API format)
    if let Some(text) = data.get("output_text").and_then(|t| t.as_str()) {
        if !text.is_empty() {
            return text.to_string();
        }
    }

    // 2. Try output array with message/content blocks
    if let Some(output) = data.get("output").and_then(|o| o.as_array()) {
        let mut texts = Vec::new();
        for item in output {
            // Try message.content[].text
            if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
                for block in content {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        texts.push(text.to_string());
                    }
                }
            }
            // Try message.content as string
            if texts.is_empty() {
                if let Some(content) = item.get("content").and_then(|c| c.as_str()) {
                    if !content.is_empty() {
                        texts.push(content.to_string());
                    }
                }
            }
        }
        if !texts.is_empty() {
            return texts.join("\n\n");
        }
    }

    // 3. Fallback: try choices format (chat/completions compatibility)
    if let Some(choices) = data.get("choices").and_then(|c| c.as_array()) {
        if let Some(first) = choices.first() {
            if let Some(text) = first.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                return text.to_string();
            }
        }
    }

    // 4. Nothing found — return empty (caller will show debug info)
    String::new()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}
