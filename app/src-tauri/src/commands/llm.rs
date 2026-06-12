use crate::core::llm_client::{LlmClient, LlmTestResult, LlmSlotConfig, ModelListResult};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

const KNOWN_ENV_KEYS: &[(&str, &str)] = &[
    ("OPENAI_API_KEY", "openai"),
    ("DEEPSEEK_API_KEY", "deepseek"),
    ("ANTHROPIC_API_KEY", "anthropic"),
    ("GEMINI_API_KEY", "gemini"),
    ("GOOGLE_API_KEY", "gemini"),
    ("MOONSHOT_API_KEY", "kimi"),
    ("MIMO_API_KEY", "mimo"),
    ("MINIMAX_API_KEY", "minimax"),
    ("OPENROUTER_API_KEY", "openrouter"),
];

#[derive(Serialize)]
pub struct EnvKeyInfo {
    pub key_name: String,
    pub provider: String,
    pub is_set: bool,
}

#[tauri::command]
pub fn detect_env_keys() -> Vec<EnvKeyInfo> {
    KNOWN_ENV_KEYS.iter().map(|(key_name, provider)| {
        EnvKeyInfo {
            key_name: key_name.to_string(),
            provider: provider.to_string(),
            is_set: std::env::var(key_name).map(|v| !v.is_empty()).unwrap_or(false),
        }
    }).collect()
}

fn settings_path() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("journal-finder");
    fs::create_dir_all(&dir).ok();
    dir.push("settings.json");
    dir
}

#[tauri::command]
pub async fn test_llm_connection_from_config(config: LlmSlotConfig) -> Result<LlmTestResult, String> {
    let api_key = match config.resolve_api_key() {
        Ok(key) => key,
        Err(e) => {
            return Ok(LlmTestResult {
                ok: false, message: e, latency_ms: 0,
                url: String::new(), http_status: 0, model_used: String::new(),
                prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
            });
        }
    };
    let slot = config.to_slot(api_key);
    let client = LlmClient::new();
    Ok(client.test_connection(&slot).await)
}

#[tauri::command]
pub async fn fetch_models(config: LlmSlotConfig) -> Result<ModelListResult, String> {
    let client = LlmClient::new();
    Ok(client.fetch_models(&config).await)
}

#[tauri::command]
pub async fn save_settings(slots: Vec<LlmSlotConfig>) -> Result<(), String> {
    let path = settings_path();
    let safe_slots: Vec<serde_json::Value> = slots.iter().map(|s| {
        serde_json::json!({
            "name": s.name,
            "provider": s.provider,
            "api_format": s.api_format,
            "base_url": s.base_url,
            "model": s.model,
            "api_key_env": s.api_key_env,
            "reasoning_enabled": s.reasoning_enabled,
            "reasoning_mode": s.reasoning_mode,
            "reasoning_budget": s.reasoning_budget,
            "model_list_source": s.model_list_source,
        })
    }).collect();
    let json = serde_json::to_string_pretty(&safe_slots)
        .map_err(|e| format!("Serialization error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write settings: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn load_settings() -> Result<Vec<LlmSlotConfig>, String> {
    Ok(load_settings_inner())
}

pub fn load_settings_inner() -> Vec<LlmSlotConfig> {
    let path = settings_path();
    if !path.exists() {
        return default_slots();
    }
    match fs::read_to_string(&path) {
        Ok(data) => {
            let mut slots: Vec<LlmSlotConfig> = serde_json::from_str(&data)
                .unwrap_or_else(|_| default_slots());
            for s in &mut slots { s.api_key.clear(); }
            slots
        }
        Err(_) => default_slots(),
    }
}

fn default_slots() -> Vec<LlmSlotConfig> {
    vec![
        LlmSlotConfig {
            name: "summarizer".to_string(),
            provider: "openai".to_string(),
            api_format: crate::core::llm_client::ApiFormat::OpenAICompatible,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-5.4".to_string(),
            api_key: String::new(),
            api_key_env: "OPENAI_API_KEY".to_string(),
            reasoning_enabled: false,
            reasoning_mode: "off".to_string(),
            reasoning_budget: None,
            model_list_source: "api".to_string(),
        },
        LlmSlotConfig {
            name: "journal_assessor".to_string(),
            provider: "openai".to_string(),
            api_format: crate::core::llm_client::ApiFormat::OpenAICompatible,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-5.4".to_string(),
            api_key: String::new(),
            api_key_env: "OPENAI_API_KEY".to_string(),
            reasoning_enabled: false,
            reasoning_mode: "off".to_string(),
            reasoning_budget: None,
            model_list_source: "api".to_string(),
        },
    ]
}

fn presets_path() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("journal-finder");
    fs::create_dir_all(&dir).ok();
    dir.push("presets.json");
    dir
}

#[tauri::command]
pub fn load_presets() -> Result<String, String> {
    let path = presets_path();
    if !path.exists() {
        return Ok("[]".to_string());
    }
    fs::read_to_string(&path).map_err(|e| format!("Failed to read presets: {}", e))
}

#[tauri::command]
pub fn save_presets(json: String) -> Result<(), String> {
    // Validate JSON
    serde_json::from_str::<serde_json::Value>(&json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    let path = presets_path();
    fs::write(&path, &json).map_err(|e| format!("Failed to write presets: {}", e))?;
    Ok(())
}
