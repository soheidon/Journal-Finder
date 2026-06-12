use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ApiFormat {
    #[serde(rename = "openai_compatible")]
    OpenAICompatible,
    Anthropic,
    Gemini,
    Ollama,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlmSlot {
    pub name: String,
    pub provider: String,
    pub api_format: ApiFormat,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    #[serde(default)]
    pub reasoning_enabled: bool,
    #[serde(default)]
    pub reasoning_mode: String,
    #[serde(default)]
    pub reasoning_budget: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlmSlotConfig {
    pub name: String,
    pub provider: String,
    #[serde(default = "default_api_format")]
    pub api_format: ApiFormat,
    pub base_url: String,
    pub model: String,
    #[serde(default, skip_serializing)]
    pub api_key: String,
    #[serde(default)]
    pub api_key_env: String,
    #[serde(default)]
    pub reasoning_enabled: bool,
    #[serde(default = "default_reasoning_mode")]
    pub reasoning_mode: String,
    #[serde(default)]
    pub reasoning_budget: Option<u32>,
    #[serde(default = "default_model_list_source")]
    pub model_list_source: String,
}

fn default_api_format() -> ApiFormat { ApiFormat::OpenAICompatible }
fn default_reasoning_mode() -> String { "off".to_string() }
fn default_model_list_source() -> String { "static".to_string() }

impl LlmSlotConfig {
    pub fn resolve_api_key(&self) -> Result<String, String> {
        if self.api_key_env.is_empty() {
            return Err(format!(
                "環境変数名が設定されていません。設定画面で {} スロットの環境変数名を指定してください。",
                self.name
            ));
        }
        match std::env::var(&self.api_key_env) {
            Ok(val) if !val.is_empty() => Ok(val),
            Ok(_) => Err(format!(
                "環境変数 {} は設定されていますが値が空です。ターミナルで `set {}=<your-key>` を実行してください。",
                self.api_key_env, self.api_key_env
            )),
            Err(_) => Err(format!(
                "環境変数 {} が設定されていません。ターミナルで `set {}=<your-key>` を実行してください。",
                self.api_key_env, self.api_key_env
            )),
        }
    }

    pub fn to_slot(&self, api_key: String) -> LlmSlot {
        LlmSlot {
            name: self.name.clone(),
            provider: self.provider.clone(),
            api_format: self.api_format.clone(),
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            api_key,
            reasoning_enabled: self.reasoning_enabled,
            reasoning_mode: self.reasoning_mode.clone(),
            reasoning_budget: self.reasoning_budget,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlmTestResult {
    pub ok: bool,
    pub message: String,
    pub latency_ms: u64,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub http_status: u16,
    #[serde(default)]
    pub model_used: String,
    #[serde(default)]
    pub prompt_tokens: u32,
    #[serde(default)]
    pub completion_tokens: u32,
    #[serde(default)]
    pub total_tokens: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelInfo {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelListResult {
    pub ok: bool,
    pub models: Vec<ModelInfo>,
    pub message: String,
}

pub struct LlmClient {
    http: reqwest::Client,
}

impl LlmClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self { http }
    }

    pub fn with_timeout(secs: u64) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(secs))
            .build()
            .expect("Failed to create HTTP client");
        Self { http }
    }

    pub async fn chat(&self, slot: &LlmSlot, messages: &[ChatMessage], max_tokens: u32) -> Result<String, String> {
        match slot.api_format {
            ApiFormat::OpenAICompatible | ApiFormat::Ollama => self.chat_openai_compatible(slot, messages, max_tokens).await,
            ApiFormat::Anthropic => self.chat_anthropic(slot, messages, max_tokens).await,
            ApiFormat::Gemini => self.chat_gemini(slot, messages, max_tokens).await,
        }
    }

    fn build_openai_body(&self, slot: &LlmSlot, messages: &[ChatMessage], max_tokens: u32) -> serde_json::Value {
        let msgs: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();

        let mut body = serde_json::json!({
            "model": slot.model,
            "messages": msgs,
            "max_tokens": max_tokens,
            "temperature": 0.3,
        });

        if slot.reasoning_enabled && slot.reasoning_mode != "off" {
            match slot.reasoning_mode.as_str() {
                "extended" | "max" => {
                    body["thinking"] = serde_json::json!({"type": "enabled"});
                    if let Some(budget) = slot.reasoning_budget {
                        body["thinking"]["budget_tokens"] = serde_json::json!(budget);
                    }
                }
                _ => {}
            }
        }

        body
    }

    async fn chat_openai_compatible(&self, slot: &LlmSlot, messages: &[ChatMessage], max_tokens: u32) -> Result<String, String> {
        let base = slot.base_url.trim_end_matches('/');
        let url = format!("{}/chat/completions", base);
        let body = self.build_openai_body(slot, messages, max_tokens);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", slot.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, truncate(&text, 300)));
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Invalid JSON response: {}", e))?;

        // Try standard content first, then reasoning_content for reasoning models
        let content = data
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content").or(m.get("reasoning_content")))
            .and_then(|c| c.as_str())
            .ok_or("No content in response")?;

        Ok(content.to_string())
    }

    async fn chat_anthropic(&self, slot: &LlmSlot, messages: &[ChatMessage], max_tokens: u32) -> Result<String, String> {
        let base = slot.base_url.trim_end_matches('/');
        let url = format!("{}/v1/messages", base);

        let mut system_text = String::new();
        let mut user_messages: Vec<serde_json::Value> = Vec::new();

        for msg in messages {
            match msg.role.as_str() {
                "system" => {
                    if !system_text.is_empty() { system_text.push_str("\n\n"); }
                    system_text.push_str(&msg.content);
                }
                _ => {
                    user_messages.push(serde_json::json!({"role": msg.role, "content": msg.content}));
                }
            }
        }

        let mut body = serde_json::json!({
            "model": slot.model,
            "max_tokens": max_tokens,
            "messages": user_messages,
        });

        if !system_text.is_empty() {
            body["system"] = serde_json::Value::String(system_text);
        }

        if slot.reasoning_enabled && slot.reasoning_mode != "off" {
            match slot.reasoning_mode.as_str() {
                "extended" | "max" => {
                    body["thinking"] = serde_json::json!({"type": "enabled", "budget_tokens": slot.reasoning_budget.unwrap_or(10000)});
                }
                "standard" => {
                    body["thinking"] = serde_json::json!({"type": "enabled", "budget_tokens": slot.reasoning_budget.unwrap_or(5000)});
                }
                _ => {}
            }
        }

        let resp = self
            .http
            .post(&url)
            .header("x-api-key", &slot.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, truncate(&text, 300)));
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Invalid JSON response: {}", e))?;

        let content = data
            .get("content")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .ok_or("No content in response")?;

        Ok(content.to_string())
    }

    async fn chat_gemini(&self, slot: &LlmSlot, messages: &[ChatMessage], max_tokens: u32) -> Result<String, String> {
        let base = slot.base_url.trim_end_matches('/');
        let url = format!("{}/v1beta/models/{}:generateContent?key={}", base, slot.model, slot.api_key);

        let contents: Vec<serde_json::Value> = messages.iter()
            .filter(|m| m.role != "system")
            .map(|m| serde_json::json!({
                "role": if m.role == "assistant" { "model" } else { "user" },
                "parts": [{"text": m.content}]
            }))
            .collect();

        let system_text: String = messages.iter()
            .filter(|m| m.role == "system")
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": max_tokens,
                "temperature": 0.3,
            }
        });

        if !system_text.is_empty() {
            body["systemInstruction"] = serde_json::json!({"parts": [{"text": system_text}]});
        }

        if slot.reasoning_enabled && slot.reasoning_mode != "off" {
            let thinking_config = match slot.reasoning_mode.as_str() {
                "extended" => serde_json::json!({"thinkingConfig": {"thinkingBudget": slot.reasoning_budget.unwrap_or(10000)}}),
                "max" => serde_json::json!({"thinkingConfig": {"thinkingBudget": slot.reasoning_budget.unwrap_or(32000)}}),
                _ => serde_json::json!({"thinkingConfig": {"thinkingBudget": slot.reasoning_budget.unwrap_or(5000)}}),
            };
            body["generationConfig"] = serde_json::json!({
                "maxOutputTokens": max_tokens,
                "temperature": 0.3,
            });
            body["generationConfig"]["thinkingConfig"] = thinking_config["thinkingConfig"].clone();
        }

        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, truncate(&text, 300)));
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Invalid JSON response: {}", e))?;

        let content = data
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or("No content in response")?;

        Ok(content.to_string())
    }

    // ── Connection test ──

    pub async fn test_connection(&self, slot: &LlmSlot) -> LlmTestResult {
        let start = std::time::Instant::now();

        let base = slot.base_url.trim_end_matches('/');
        let url = match slot.api_format {
            ApiFormat::OpenAICompatible | ApiFormat::Ollama => format!("{}/chat/completions", base),
            ApiFormat::Anthropic => format!("{}/v1/messages", base),
            ApiFormat::Gemini => format!("{}/v1beta/models/{}:generateContent?key=REDACTED", base, slot.model),
        };

        let result = match slot.api_format {
            ApiFormat::OpenAICompatible | ApiFormat::Ollama => self.test_openai_direct(slot, &format!("{}/chat/completions", base)).await,
            ApiFormat::Anthropic => self.test_anthropic_direct(slot, &format!("{}/v1/messages", base)).await,
            ApiFormat::Gemini => self.test_gemini_direct(slot).await,
        };

        let latency = start.elapsed().as_millis() as u64;

        match result {
            Ok(mut r) => { r.latency_ms = latency; r.url = url; r }
            Err(e) => LlmTestResult {
                ok: false, message: e, latency_ms: latency, url,
                http_status: 0, model_used: String::new(),
                prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
            },
        }
    }

    async fn test_openai_direct(&self, slot: &LlmSlot, url: &str) -> Result<LlmTestResult, String> {
        let body = serde_json::json!({
            "model": slot.model,
            "messages": [{"role": "user", "content": "Say OK"}],
            "max_tokens": 32,
        });

        let resp = self.http.post(url)
            .header("Authorization", format!("Bearer {}", slot.api_key))
            .header("Content-Type", "application/json")
            .json(&body).send().await
            .map_err(|e| format!("HTTPリクエスト失敗: {}", e))?;

        let status_code = resp.status().as_u16();
        let text = resp.text().await.map_err(|e| format!("レスポンス読み取り失敗: {}", e))?;

        if status_code == 0 || (status_code >= 400) {
            return Ok(self.build_error_result(status_code, &text, url));
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("JSON パース失敗: {}", e))?;

        let model_used = data.get("model").and_then(|m| m.as_str()).unwrap_or("").to_string();
        let (pt, ct, tt) = extract_openai_usage(&data);
        let has_choices = data.get("choices").and_then(|c| c.as_array()).map(|a| !a.is_empty()).unwrap_or(false);

        if !has_choices {
            return Ok(self.build_error_result(status_code, &text, url));
        }

        Ok(build_success_result(status_code, &model_used, pt, ct, tt))
    }

    async fn test_anthropic_direct(&self, slot: &LlmSlot, url: &str) -> Result<LlmTestResult, String> {
        let body = serde_json::json!({
            "model": slot.model, "max_tokens": 32,
            "messages": [{"role": "user", "content": "Say OK"}]
        });

        let resp = self.http.post(url)
            .header("x-api-key", &slot.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body).send().await
            .map_err(|e| format!("HTTPリクエスト失敗: {}", e))?;

        let status_code = resp.status().as_u16();
        let text = resp.text().await.map_err(|e| format!("レスポンス読み取り失敗: {}", e))?;

        if status_code >= 400 {
            return Ok(self.build_error_result(status_code, &text, url));
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("JSON パース失敗: {}", e))?;

        let model_used = data.get("model").and_then(|m| m.as_str()).unwrap_or("").to_string();
        let usage = data.get("usage");
        let pt = usage.and_then(|u| u.get("input_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let ct = usage.and_then(|u| u.get("output_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let has_content = data.get("content").and_then(|c| c.as_array()).map(|a| !a.is_empty()).unwrap_or(false);

        if !has_content {
            return Ok(self.build_error_result(status_code, &text, url));
        }

        Ok(build_success_result(status_code, &model_used, pt, ct, pt + ct))
    }

    async fn test_gemini_direct(&self, slot: &LlmSlot) -> Result<LlmTestResult, String> {
        let base = slot.base_url.trim_end_matches('/');
        let url = format!("{}/v1beta/models/{}:generateContent?key={}", base, slot.model, slot.api_key);

        let body = serde_json::json!({
            "contents": [{"role": "user", "parts": [{"text": "Say OK"}]}],
            "generationConfig": {"maxOutputTokens": 32}
        });

        let resp = self.http.post(&url)
            .header("Content-Type", "application/json")
            .json(&body).send().await
            .map_err(|e| format!("HTTPリクエスト失敗: {}", e))?;

        let status_code = resp.status().as_u16();
        let text = resp.text().await.map_err(|e| format!("レスポンス読み取り失敗: {}", e))?;

        if status_code >= 400 {
            let safe_url = url.split("?key=").next().unwrap_or(&url);
            return Ok(self.build_error_result(status_code, &text, safe_url));
        }

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("JSON パース失敗: {}", e))?;

        let has_content = data.get("candidates").and_then(|c| c.get(0)).is_some();
        let usage = data.get("usageMetadata");
        let pt = usage.and_then(|u| u.get("promptTokenCount")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let ct = usage.and_then(|u| u.get("candidatesTokenCount")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        if !has_content {
            let safe_url = url.split("?key=").next().unwrap_or(&url);
            return Ok(self.build_error_result(status_code, &text, safe_url));
        }

        Ok(build_success_result(status_code, &slot.model, pt, ct, pt + ct))
    }

    fn build_error_result(&self, status_code: u16, text: &str, _url: &str) -> LlmTestResult {
        LlmTestResult {
            ok: false,
            message: format!("HTTP {} レスポンス:\n{}", status_code, truncate(text, 500)),
            latency_ms: 0, url: String::new(), http_status: status_code,
            model_used: String::new(), prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
        }
    }

    // ── Model list fetching ──

    pub async fn fetch_models(&self, slot_config: &LlmSlotConfig) -> ModelListResult {
        let api_key = match slot_config.resolve_api_key() {
            Ok(k) => k,
            Err(e) => return ModelListResult { ok: false, models: vec![], message: e },
        };

        let base = slot_config.base_url.trim_end_matches('/');
        let result = match slot_config.api_format {
            ApiFormat::OpenAICompatible => self.fetch_openai_models(base, &api_key).await,
            ApiFormat::Anthropic => self.fetch_anthropic_models(base, &api_key).await,
            ApiFormat::Gemini => self.fetch_gemini_models(base, &api_key).await,
            ApiFormat::Ollama => self.fetch_ollama_models(base).await,
        };

        match result {
            Ok(models) => ModelListResult { ok: true, models, message: "OK".to_string() },
            Err(e) => ModelListResult { ok: false, models: vec![], message: e },
        }
    }

    async fn fetch_openai_models(&self, base: &str, api_key: &str) -> Result<Vec<ModelInfo>, String> {
        let url = format!("{}/models", base);
        let resp = self.http.get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send().await.map_err(|e| format!("HTTP error: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, truncate(&text, 200)));
        }

        let data: serde_json::Value = resp.json().await.map_err(|e| format!("JSON error: {}", e))?;
        let arr = data.get("data").and_then(|d| d.as_array()).ok_or("No data field")?;

        let mut models: Vec<ModelInfo> = arr.iter().filter_map(|m| {
            let id = m.get("id")?.as_str()?;
            Some(ModelInfo { id: id.to_string(), name: id.to_string(), description: String::new() })
        }).collect();

        models.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(models)
    }

    async fn fetch_anthropic_models(&self, _base: &str, _api_key: &str) -> Result<Vec<ModelInfo>, String> {
        Err("Anthropic does not support model listing via API. Use static presets.".to_string())
    }

    async fn fetch_gemini_models(&self, base: &str, api_key: &str) -> Result<Vec<ModelInfo>, String> {
        let url = format!("{}/v1beta/models?key={}", base, api_key);
        let resp = self.http.get(&url).send().await.map_err(|e| format!("HTTP error: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            let safe = text.chars().take(200).collect::<String>();
            return Err(format!("HTTP {}: {}", status, safe));
        }

        let data: serde_json::Value = resp.json().await.map_err(|e| format!("JSON error: {}", e))?;
        let arr = data.get("models").and_then(|m| m.as_array()).ok_or("No models field")?;

        let models: Vec<ModelInfo> = arr.iter().filter_map(|m| {
            let name = m.get("name")?.as_str()?;
            let id = name.strip_prefix("models/").unwrap_or(name);
            let display = m.get("displayName").and_then(|d| d.as_str()).unwrap_or(id);
            Some(ModelInfo { id: id.to_string(), name: display.to_string(), description: String::new() })
        }).collect();

        Ok(models)
    }

    async fn fetch_ollama_models(&self, base: &str) -> Result<Vec<ModelInfo>, String> {
        // Ollama /api/tags is at the root, not under /v1
        let root = base.trim_end_matches('/').trim_end_matches("/v1").trim_end_matches('/');
        let url = format!("{}/api/tags", root);
        let resp = self.http.get(&url).send().await.map_err(|e| format!("HTTP error: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, truncate(&text, 200)));
        }

        let data: serde_json::Value = resp.json().await.map_err(|e| format!("JSON error: {}", e))?;
        let arr = data.get("models").and_then(|m| m.as_array()).ok_or("No models field")?;

        let models: Vec<ModelInfo> = arr.iter().filter_map(|m| {
            let name = m.get("name")?.as_str()?;
            Some(ModelInfo { id: name.to_string(), name: name.to_string(), description: String::new() })
        }).collect();

        Ok(models)
    }
}

fn extract_openai_usage(data: &serde_json::Value) -> (u32, u32, u32) {
    let usage = data.get("usage");
    let pt = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let ct = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let tt = usage.and_then(|u| u.get("total_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    (pt, ct, tt)
}

fn build_success_result(status_code: u16, model_used: &str, pt: u32, ct: u32, tt: u32) -> LlmTestResult {
    let mut msg = format!("接続成功: model={}", model_used);
    if tt > 0 { msg += &format!(", tokens={}(prompt={}, completion={})", tt, pt, ct); }
    LlmTestResult {
        ok: true, message: msg, latency_ms: 0, url: String::new(),
        http_status: status_code, model_used: model_used.to_string(),
        prompt_tokens: pt, completion_tokens: ct, total_tokens: tt,
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}
