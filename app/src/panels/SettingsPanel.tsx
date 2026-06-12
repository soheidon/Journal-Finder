import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { LlmSlotConfig, LlmTestResult, LlmProvider, ApiFormat, ReasoningMode, ModelListResult } from "../App";

interface SettingsPanelProps {
  addLog: (message: string) => void;
  showStatus: (type: "success" | "error" | "info", text: string) => void;
  onLlmStatusChange: (configured: boolean, testOk: boolean) => void;
  testResults: Record<string, LlmTestResult>;
  onTestResultsChange: (results: Record<string, LlmTestResult>) => void;
}

interface Preset {
  label: string;
  provider: LlmProvider;
  api_format: ApiFormat;
  base_url: string;
  model: string;
  api_key_env: string;
  reasoning_enabled: boolean;
  reasoning_mode: ReasoningMode;
  model_list_source: "static" | "api" | "local";
}

const providerMeta: Record<LlmProvider, { label: string; api_format: ApiFormat; base_url: string; env_var: string; model_list: "static" | "api" | "local" }> = {
  openai: { label: "OpenAI", api_format: "openai_compatible", base_url: "https://api.openai.com/v1", env_var: "OPENAI_API_KEY", model_list: "api" },
  anthropic: { label: "Anthropic", api_format: "anthropic", base_url: "https://api.anthropic.com", env_var: "ANTHROPIC_API_KEY", model_list: "static" },
  deepseek: { label: "DeepSeek", api_format: "openai_compatible", base_url: "https://api.deepseek.com", env_var: "DEEPSEEK_API_KEY", model_list: "static" },
  openrouter: { label: "OpenRouter", api_format: "openai_compatible", base_url: "https://openrouter.ai/api/v1", env_var: "OPENROUTER_API_KEY", model_list: "api" },
  ollama: { label: "Ollama", api_format: "ollama", base_url: "http://localhost:11434/v1", env_var: "", model_list: "local" },
  gemini: { label: "Gemini", api_format: "gemini", base_url: "https://generativelanguage.googleapis.com", env_var: "GEMINI_API_KEY", model_list: "api" },
  kimi: { label: "Kimi", api_format: "openai_compatible", base_url: "https://api.moonshot.cn/v1", env_var: "MOONSHOT_API_KEY", model_list: "static" },
  mimo: { label: "Xiaomi MiMo", api_format: "openai_compatible", base_url: "https://api.mimo.xiaomi.com/v1", env_var: "MIMO_API_KEY", model_list: "static" },
  minimax: { label: "MiniMax", api_format: "openai_compatible", base_url: "https://api.minimax.chat/v1", env_var: "MINIMAX_API_KEY", model_list: "static" },
  custom: { label: "Custom", api_format: "openai_compatible", base_url: "", env_var: "", model_list: "static" },
};

const apiFormatOptions: { value: ApiFormat; label: string }[] = [
  { value: "openai_compatible", label: "OpenAI Compatible" },
  { value: "anthropic", label: "Anthropic" },
  { value: "gemini", label: "Gemini" },
  { value: "ollama", label: "Ollama" },
];

const reasoningModeOptions: { value: ReasoningMode; label: string }[] = [
  { value: "off", label: "Off" },
  { value: "standard", label: "Standard" },
  { value: "extended", label: "Extended" },
  { value: "max", label: "Max" },
  { value: "custom", label: "Custom Budget" },
];

const presets: Preset[] = [
  // OpenAI
  { label: "OpenAI — gpt-5.5", provider: "openai", api_format: "openai_compatible", base_url: "https://api.openai.com/v1", model: "gpt-5.5", api_key_env: "OPENAI_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "api" },
  { label: "OpenAI — gpt-5.4", provider: "openai", api_format: "openai_compatible", base_url: "https://api.openai.com/v1", model: "gpt-5.4", api_key_env: "OPENAI_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "api" },
  // DeepSeek
  { label: "DeepSeek — v4-pro", provider: "deepseek", api_format: "openai_compatible", base_url: "https://api.deepseek.com", model: "deepseek-v4-pro", api_key_env: "DEEPSEEK_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  { label: "DeepSeek — v4-flash", provider: "deepseek", api_format: "openai_compatible", base_url: "https://api.deepseek.com", model: "deepseek-v4-flash", api_key_env: "DEEPSEEK_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  // Gemini
  { label: "Gemini — 3.5 Flash", provider: "gemini", api_format: "gemini", base_url: "https://generativelanguage.googleapis.com", model: "gemini-3.5-flash", api_key_env: "GEMINI_API_KEY", reasoning_enabled: true, reasoning_mode: "standard", model_list_source: "api" },
  { label: "Gemini — 3.5 Flash Extended", provider: "gemini", api_format: "gemini", base_url: "https://generativelanguage.googleapis.com", model: "gemini-3.5-flash", api_key_env: "GEMINI_API_KEY", reasoning_enabled: true, reasoning_mode: "extended", model_list_source: "api" },
  { label: "Gemini — 3.5 Flash Max", provider: "gemini", api_format: "gemini", base_url: "https://generativelanguage.googleapis.com", model: "gemini-3.5-flash", api_key_env: "GEMINI_API_KEY", reasoning_enabled: true, reasoning_mode: "max", model_list_source: "api" },
  { label: "Gemini — 2.5 Pro", provider: "gemini", api_format: "gemini", base_url: "https://generativelanguage.googleapis.com", model: "gemini-2.5-pro", api_key_env: "GEMINI_API_KEY", reasoning_enabled: true, reasoning_mode: "standard", model_list_source: "api" },
  // Kimi
  { label: "Kimi K2.6 — Non-thinking", provider: "kimi", api_format: "openai_compatible", base_url: "https://api.moonshot.cn/v1", model: "kimi-k2.6", api_key_env: "MOONSHOT_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  { label: "Kimi K2.6 — Thinking", provider: "kimi", api_format: "openai_compatible", base_url: "https://api.moonshot.cn/v1", model: "kimi-k2.6", api_key_env: "MOONSHOT_API_KEY", reasoning_enabled: true, reasoning_mode: "standard", model_list_source: "static" },
  { label: "Kimi K2.6 — Extended", provider: "kimi", api_format: "openai_compatible", base_url: "https://api.moonshot.cn/v1", model: "kimi-k2.6", api_key_env: "MOONSHOT_API_KEY", reasoning_enabled: true, reasoning_mode: "extended", model_list_source: "static" },
  { label: "Kimi K2.5 — Non-thinking", provider: "kimi", api_format: "openai_compatible", base_url: "https://api.moonshot.cn/v1", model: "kimi-k2.5", api_key_env: "MOONSHOT_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  { label: "Kimi K2.5 — Thinking", provider: "kimi", api_format: "openai_compatible", base_url: "https://api.moonshot.cn/v1", model: "kimi-k2.5", api_key_env: "MOONSHOT_API_KEY", reasoning_enabled: true, reasoning_mode: "standard", model_list_source: "static" },
  // Xiaomi MiMo
  { label: "MiMo — v2.5-pro", provider: "mimo", api_format: "openai_compatible", base_url: "https://api.mimo.xiaomi.com/v1", model: "mimo-v2.5-pro", api_key_env: "MIMO_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  { label: "MiMo — v2.5", provider: "mimo", api_format: "openai_compatible", base_url: "https://api.mimo.xiaomi.com/v1", model: "mimo-v2.5", api_key_env: "MIMO_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  // MiniMax
  { label: "MiniMax — M3", provider: "minimax", api_format: "openai_compatible", base_url: "https://api.minimax.chat/v1", model: "MiniMax-M3", api_key_env: "MINIMAX_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  // Anthropic
  { label: "Claude — Fable 5", provider: "anthropic", api_format: "anthropic", base_url: "https://api.anthropic.com", model: "claude-fable-5", api_key_env: "ANTHROPIC_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  { label: "Claude — Opus 4.8", provider: "anthropic", api_format: "anthropic", base_url: "https://api.anthropic.com", model: "claude-opus-4-8", api_key_env: "ANTHROPIC_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  { label: "Claude — Sonnet 4.6", provider: "anthropic", api_format: "anthropic", base_url: "https://api.anthropic.com", model: "claude-sonnet-4-6", api_key_env: "ANTHROPIC_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  { label: "Claude — Haiku 4.5", provider: "anthropic", api_format: "anthropic", base_url: "https://api.anthropic.com", model: "claude-haiku-4-5", api_key_env: "ANTHROPIC_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "static" },
  // OpenRouter
  { label: "OpenRouter — DeepSeek v4-pro", provider: "openrouter", api_format: "openai_compatible", base_url: "https://openrouter.ai/api/v1", model: "deepseek/deepseek-v4-pro", api_key_env: "OPENROUTER_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "api" },
  { label: "OpenRouter — MiMo v2.5-pro", provider: "openrouter", api_format: "openai_compatible", base_url: "https://openrouter.ai/api/v1", model: "xiaomi/mimo-v2.5-pro", api_key_env: "OPENROUTER_API_KEY", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "api" },
  // Ollama
  { label: "Ollama — llama3.3", provider: "ollama", api_format: "ollama", base_url: "http://localhost:11434/v1", model: "llama3.3", api_key_env: "", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "local" },
  { label: "Ollama — qwen3", provider: "ollama", api_format: "ollama", base_url: "http://localhost:11434/v1", model: "qwen3", api_key_env: "", reasoning_enabled: false, reasoning_mode: "off", model_list_source: "local" },
];

function makeDefault(name: string): LlmSlotConfig {
  return {
    name,
    provider: "openai",
    api_format: "openai_compatible",
    base_url: "https://api.openai.com/v1",
    model: "gpt-5.4",
    api_key_env: "OPENAI_API_KEY",
    reasoning_enabled: false,
    reasoning_mode: "off",
    reasoning_budget: null,
    model_list_source: "api",
  };
}

function SettingsPanel({ addLog, showStatus, onLlmStatusChange, testResults, onTestResultsChange }: SettingsPanelProps) {
  const [slots, setSlots] = useState<LlmSlotConfig[]>([makeDefault("summarizer"), makeDefault("journal_assessor")]);
  const [testing, setTesting] = useState<Record<string, boolean>>({});
  const [loading, setLoading] = useState(true);
  const [modelListResults, setModelListResults] = useState<Record<string, ModelListResult>>({});
  const [fetchingModels, setFetchingModels] = useState<Record<string, boolean>>({});
  const [customPresets, setCustomPresets] = useState<Preset[]>([]);
  const initialLoadDone = useRef(false);
  const suppressAutoSave = useRef(false);

  const allPresets = useMemo(() => [...presets, ...customPresets], [customPresets]);

  useEffect(() => {
    const load = async () => {
      suppressAutoSave.current = true;
      try {
        const loaded = await invoke<LlmSlotConfig[]>("load_settings");
        if (loaded.length > 0) {
          // Validate model-provider consistency
          const fixed = loaded.map(s => {
            const availablePresets = allPresets.filter(p => p.provider === s.provider);
            if (availablePresets.length > 0 && !availablePresets.some(p => p.model === s.model)) {
              // Model doesn't match provider — reset to first preset
              const first = availablePresets[0];
              addLog(`[${s.name}] モデル '${s.model}' は ${s.provider} のプリセットにないため、${first.model} に修正しました`);
              return { ...makeDefault(s.name), ...s, model: first.model, base_url: first.base_url };
            }
            return { ...makeDefault(s.name), ...s };
          });
          setSlots(fixed);
          addLog("設定を読み込みました");
        } else {
          // No saved settings — auto-detect from environment variables
          const envKeys = await invoke<{ key_name: string; provider: string; is_set: boolean }[]>("detect_env_keys");
          const setKeys = envKeys.filter(k => k.is_set);
          if (setKeys.length > 0) {
            const detected = setKeys[0];
            const meta = providerMeta[detected.provider as LlmProvider];
            const firstPreset = allPresets.find(p => p.provider === detected.provider);
            if (meta && firstPreset) {
              setSlots(prev => {
                const next = [...prev];
                next[0] = { ...next[0], provider: detected.provider as LlmProvider, api_format: meta.api_format, base_url: firstPreset.base_url, model: firstPreset.model, api_key_env: detected.key_name, reasoning_enabled: firstPreset.reasoning_enabled, reasoning_mode: firstPreset.reasoning_mode, model_list_source: meta.model_list };
                return next;
              });
              addLog(`環境変数 ${detected.key_name} を検出しました。summarizer を ${meta.label} に自動設定しました`);
            }
          }
        }
      } catch (e) { addLog(`設定読み込みエラー: ${e}`); }
      finally {
        setLoading(false);
        initialLoadDone.current = true;
        setTimeout(() => { suppressAutoSave.current = false; }, 500);
      }
    };
    load();
  }, [addLog]);

  // Load custom presets
  useEffect(() => {
    const loadPresets = async () => {
      try {
        const json = await invoke<string>("load_presets");
        const parsed = JSON.parse(json);
        if (Array.isArray(parsed)) {
          setCustomPresets(parsed);
        }
      } catch (_) { /* no custom presets yet */ }
    };
    loadPresets();
  }, []);

  // Auto-save when slots change (after initial load, with debounce)
  useEffect(() => {
    if (!initialLoadDone.current || suppressAutoSave.current) return;
    const timer = setTimeout(async () => {
      try {
        await invoke("save_settings", { slots });
      } catch (e) {
        addLog(`自動保存エラー: ${e}`);
      }
    }, 1000);
    return () => clearTimeout(timer);
  }, [slots, addLog]);

  // Notify App.tsx about LLM status
  useEffect(() => {
    if (!initialLoadDone.current) return;
    const configured = slots.some(s => !!s.api_key_env);
    const anyTestOk = Object.values(testResults).some(r => r.ok);
    onLlmStatusChange(configured, anyTestOk);
  }, [slots, testResults, onLlmStatusChange]);

  const updateSlot = useCallback((index: number, field: keyof LlmSlotConfig, value: unknown) => {
    setSlots(prev => { const next = [...prev]; next[index] = { ...next[index], [field]: value }; return next; });
  }, []);

  const applyPreset = useCallback((index: number, preset: Preset) => {
    setSlots(prev => {
      const next = [...prev];
      next[index] = {
        ...next[index],
        provider: preset.provider,
        api_format: preset.api_format,
        base_url: preset.base_url,
        model: preset.model,
        api_key_env: preset.api_key_env || next[index].api_key_env,
        reasoning_enabled: preset.reasoning_enabled,
        reasoning_mode: preset.reasoning_mode,
        model_list_source: preset.model_list_source,
      };
      return next;
    });
  }, []);

  const handleProviderChange = useCallback((index: number, provider: LlmProvider) => {
    const meta = providerMeta[provider];
    const firstPreset = allPresets.find(p => p.provider === provider);
    setSlots(prev => {
      const next = [...prev];
      next[index] = {
        ...next[index],
        provider,
        api_format: meta.api_format,
        base_url: firstPreset?.base_url || meta.base_url,
        model: firstPreset?.model || "",
        api_key_env: next[index].api_key_env || meta.env_var,
        reasoning_enabled: firstPreset?.reasoning_enabled || false,
        reasoning_mode: firstPreset?.reasoning_mode || "off",
        model_list_source: meta.model_list,
      };
      return next;
    });
  }, []);

  const handleTest = useCallback(async (index: number) => {
    const slot = slots[index];
    if (!slot.api_key_env && slot.provider !== "ollama") {
      showStatus("error", "環境変数名を入力してください");
      return;
    }
    setTesting(p => ({ ...p, [slot.name]: true }));
    addLog(`[${slot.name}] 接続テスト開始: ${slot.provider} / ${slot.model}`);
    try {
      const result = await invoke<LlmTestResult>("test_llm_connection_from_config", { config: slot });
      onTestResultsChange({ ...testResults, [slot.name]: result });
      if (result.ok) {
        addLog(`[${slot.name}] 接続成功 (${result.latency_ms}ms) ${result.message}`);
        showStatus("success", `${slot.name}: 接続成功 (${result.latency_ms}ms)`);
      } else {
        addLog(`[${slot.name}] 接続失敗: ${result.message}`);
        showStatus("error", `${slot.name}: ${result.message}`);
      }
    } catch (e) {
      onTestResultsChange({ ...testResults, [slot.name]: { ok: false, message: `${e}`, latency_ms: 0, url: "", http_status: 0, model_used: "", prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 } });
      showStatus("error", `${slot.name}: エラー`);
    } finally { setTesting(p => ({ ...p, [slot.name]: false })); }
  }, [slots, testResults, onTestResultsChange, addLog, showStatus]);

  const handleFetchModels = useCallback(async (index: number) => {
    const slot = slots[index];
    setFetchingModels(p => ({ ...p, [slot.name]: true }));
    addLog(`[${slot.name}] モデル一覧取得開始`);
    try {
      const result = await invoke<ModelListResult>("fetch_models", { config: slot });
      setModelListResults(p => ({ ...p, [slot.name]: result }));
      if (result.ok) {
        addLog(`[${slot.name}] モデル一覧取得成功: ${result.models.length} 件`);
      } else {
        addLog(`[${slot.name}] モデル一覧取得失敗: ${result.message}`);
      }
    } catch (e) {
      addLog(`[${slot.name}] モデル一覧取得エラー: ${e}`);
    } finally { setFetchingModels(p => ({ ...p, [slot.name]: false })); }
  }, [slots, addLog]);

  const handleSaveAsPreset = useCallback(async (index: number) => {
    const slot = slots[index];
    const newPreset: Preset = {
      label: `${providerMeta[slot.provider]?.label || slot.provider} — ${slot.model}`,
      provider: slot.provider,
      api_format: slot.api_format,
      base_url: slot.base_url,
      model: slot.model,
      api_key_env: slot.api_key_env,
      reasoning_enabled: slot.reasoning_enabled,
      reasoning_mode: slot.reasoning_mode,
      model_list_source: slot.model_list_source,
    };
    const exists = customPresets.some(p => p.model === newPreset.model && p.provider === newPreset.provider);
    const updated = exists ? customPresets.map(p => p.model === newPreset.model && p.provider === newPreset.provider ? newPreset : p) : [...customPresets, newPreset];
    try {
      await invoke("save_presets", { json: JSON.stringify(updated, null, 2) });
      setCustomPresets(updated);
      addLog(`プリセット保存: ${newPreset.label}`);
      showStatus("success", `プリセット「${newPreset.label}」を保存しました`);
    } catch (e) {
      addLog(`プリセット保存エラー: ${e}`);
    }
  }, [slots, customPresets, addLog, showStatus]);

  if (loading) return <div className="panel settings-panel"><p className="hint-text">読み込み中...</p></div>;

  const providerPresets = (provider: LlmProvider) => allPresets.filter(p => p.provider === provider);

  return (
    <div className="panel settings-panel">
      <div className="input-section info-card">
        <h3>API 設定について</h3>
        <p>
          Journal Finder は LLM（大規模言語モデル）を使って論文の要約とジャーナル推薦を行います。
          2 つのスロットを設定してください。
        </p>
        <ul>
          <li><strong>summarizer</strong>: 論文の構造化要約を生成します</li>
          <li><strong>journal_assessor</strong>: Deep Research 結果を解析し、投稿先候補を推薦します</li>
        </ul>
        <p>
          API Key は環境変数で管理します。設定ファイルには環境変数名のみ保存され、キーの実値は保存されません。
        </p>
      </div>

      <div className="input-section">
        <h3>LLM スロット設定</h3>
        <p className="settings-description">API Key は環境変数で管理します。設定ファイルには環境変数名のみ保存されます。</p>

        {slots.map((slot, index) => {
          const testResult = testResults[slot.name];
          const isTesting = testing[slot.name];
          const modelList = modelListResults[slot.name];
          const isFetching = fetchingModels[slot.name];
          const meta = providerMeta[slot.provider];
          const showReasoning = ["gemini", "anthropic", "kimi"].includes(slot.provider);
          const availablePresets = providerPresets(slot.provider);

          return (
            <div key={slot.name} className="llm-slot">
              <h4>{slot.name === "summarizer" ? "summarizer（論文要約）" : "journal_assessor（ジャーナル評価）"}</h4>
              <div className="slot-layout">

                {/* Left: Settings */}
                <div className="slot-settings">
                  <div className="form-grid">
                    <label>Provider</label>
                    <select value={slot.provider} onChange={e => handleProviderChange(index, e.target.value as LlmProvider)}>
                      {Object.entries(providerMeta).map(([k, v]) => <option key={k} value={k}>{v.label}</option>)}
                    </select>

                    <label>モデル</label>
                    <div className="model-select-group">
                      {availablePresets.length > 0 ? (
                        <select value={availablePresets.some(p => p.model === slot.model) ? slot.model : (slot.model ? "__custom_with_value" : "__custom__")} onChange={e => {
                          if (e.target.value === "__custom__" || e.target.value === "__custom_with_value") {
                            updateSlot(index, "model", "");
                          } else {
                            const preset = availablePresets.find(p => p.model === e.target.value);
                            if (preset) applyPreset(index, preset);
                            else updateSlot(index, "model", e.target.value);
                          }
                        }}>
                          {availablePresets.map(p => <option key={p.model} value={p.model}>{p.label} ({p.model})</option>)}
                          <option value="__custom__">手動入力...</option>
                          {!availablePresets.some(p => p.model === slot.model) && slot.model && (
                            <option value="__custom_with_value">{slot.model} (カスタム)</option>
                          )}
                        </select>
                      ) : (
                        <input type="text" value={slot.model} onChange={e => updateSlot(index, "model", e.target.value)} placeholder="model-id を入力" />
                      )}
                    </div>

                    {availablePresets.length > 0 && !availablePresets.some(p => p.model === slot.model) && (
                      <>
                        <label>Model ID</label>
                        <input type="text" value={slot.model} onChange={e => updateSlot(index, "model", e.target.value)} placeholder="model-id を直接入力" />
                      </>
                    )}

                    <label>Base URL</label>
                    <input type="text" value={slot.base_url} onChange={e => updateSlot(index, "base_url", e.target.value)} />

                    <label>環境変数名</label>
                    <input type="text" value={slot.api_key_env} onChange={e => updateSlot(index, "api_key_env", e.target.value)} placeholder={meta.env_var || "KEY_NAME"} />

                    {meta.model_list !== "static" && (
                      <>
                        <label></label>
                        <div className="model-row">
                          <button className="fetch-btn" onClick={() => handleFetchModels(index)} disabled={isFetching}>
                            {isFetching ? "取得中..." : "モデル一覧を取得"}
                          </button>
                          {modelList?.ok && <span className="hint-text">{modelList.models.length} 件取得済み</span>}
                        </div>
                      </>
                    )}

                    {modelList?.ok && modelList.models.length > 0 && (
                      <>
                        <label>一覧から選択</label>
                        <select value="" onChange={e => { if (e.target.value) updateSlot(index, "model", e.target.value); }}>
                          <option value="">選択してください...</option>
                          {modelList.models.map(m => <option key={m.id} value={m.id}>{m.id}</option>)}
                        </select>
                      </>
                    )}

                    {showReasoning && (
                      <>
                        <label>Reasoning</label>
                        <div className="reasoning-row">
                          <select value={slot.reasoning_mode} onChange={e => {
                            const mode = e.target.value as ReasoningMode;
                            updateSlot(index, "reasoning_mode", mode);
                            updateSlot(index, "reasoning_enabled", mode !== "off");
                          }}>
                            {reasoningModeOptions.map(o => <option key={o.value} value={o.value}>{o.label}</option>)}
                          </select>
                          {slot.reasoning_mode === "custom" && (
                            <input type="number" value={slot.reasoning_budget ?? ""} onChange={e => updateSlot(index, "reasoning_budget", e.target.value ? Number(e.target.value) : null)} placeholder="tokens" className="budget-input" />
                          )}
                        </div>
                        {slot.provider === "kimi" && (
                          <p className="hint-text" style={{ color: "#ffd54f" }}>
                            Kimi の thinking mode では、一部ツール呼び出しと互換性がない場合があります。
                          </p>
                        )}
                      </>
                    )}
                  </div>
                </div>

                {/* Right: Actions */}
                <div className="slot-actions">
                  <button className="test-button" onClick={() => handleTest(index)} disabled={isTesting || (!slot.api_key_env && slot.provider !== "ollama")}>
                    {isTesting ? "テスト中..." : "接続テスト"}
                  </button>
                  {testResult && <span className={`status-chip ${testResult.ok ? "ok" : "err"}`}>{testResult.ok ? `成功 (${testResult.latency_ms}ms)` : "失敗"}</span>}

                  <button className="preset-save-btn" onClick={() => handleSaveAsPreset(index)}>
                    プリセット保存
                  </button>
                </div>

              </div>

              {/* Test details — full width below both columns */}
              {testResult && (
                <div className={`test-detail ${testResult.ok ? "test-ok" : "test-err"}`}>
                  <div className="test-detail-row"><span className="test-detail-label">URL:</span><span className="test-detail-value">{testResult.url}</span></div>
                  <div className="test-detail-row"><span className="test-detail-label">HTTP:</span><span className="test-detail-value">{testResult.http_status}</span></div>
                  {testResult.ok && testResult.model_used && <div className="test-detail-row"><span className="test-detail-label">Model:</span><span className="test-detail-value">{testResult.model_used}</span></div>}
                  {testResult.ok && testResult.total_tokens > 0 && <div className="test-detail-row"><span className="test-detail-label">Tokens:</span><span className="test-detail-value">{testResult.total_tokens} (p:{testResult.prompt_tokens} c:{testResult.completion_tokens})</span></div>}
                  {!testResult.ok && <div className="test-detail-row"><span className="test-detail-label">Error:</span><span className="test-detail-value">{testResult.message}</span></div>}
                </div>
              )}
            </div>
          );
        })}
        <p className="hint-text" style={{ marginTop: 8 }}>設定は変更から1秒後に自動保存されます。</p>
      </div>

    </div>
  );
}

export default SettingsPanel;
