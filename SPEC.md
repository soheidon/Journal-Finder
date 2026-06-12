# 論文投稿先アドバイザ — 技術仕様書

## バージョン

v0.1.0

## 技術スタック

| 層 | 技術 | バージョン |
|---|---|---|
| GUI | Tauri v2 + React + TypeScript | Tauri 2.x |
| Backend/Core | Rust | 1.70+ |
| 通信 | Tauri commands | — |
| docx 処理 | zip + quick-xml | — |
| LLM API | reqwest (HTTP client) | — |
| Word 出力 | docx-rs | 0.4.x |

## アーキテクチャ

```
React (TypeScript)
  ↕ Tauri commands
Rust backend
  ├── commands/     Tauri command handlers
  ├── core/         Business logic
  │   ├── llm_client.rs       LLM API client (OpenAI Compatible, Anthropic, Gemini, Ollama)
  │   ├── docx_extract.rs     docx text extraction (zip + XML)
  │   ├── prompts.rs           LLM prompt templates
  │   ├── summary.rs           Paper summarization
  │   ├── deep_research.rs     Deep Research result parsing
  │   ├── deep_research_api.rs OpenAI Deep Research API (Responses API)
  │   ├── report.rs            Report generation (Markdown, JSON, Word)
  │   └── ranking.rs           Journal candidate ranking
  └── src-tauri/    Tauri configuration
```

## LLM 設計

### スロット構成

| スロット | 用途 |
|---|---|
| `analysis_llm` | 論文要約、Deep Research 結果の解析、ジャーナル候補の評価 |
| `deep_research_provider` | API 経由の Deep Research / Web 検索（将来実装） |

### 対応 API 形式

| 形式 | 対象 |
|---|---|
| OpenAI Compatible | OpenAI, DeepSeek, OpenRouter, Kimi, MiMo, MiniMax, Custom |
| Anthropic Messages | Anthropic Claude |
| Gemini API | Google Gemini |
| Ollama | ローカル LLM |

### API キー管理

- 環境変数から読み込み（`std::env::var`）
- 設定ファイルには環境変数名のみ保存
- API キーの実値は保存・表示・ログ出力しない

### 推論モード

Gemini / Anthropic / Kimi で thinking / reasoning モードをサポート：

| モード | 説明 |
|---|---|
| off | 推論無効 |
| standard | 標準推論 |
| extended | 拡張推論 |
| max | 最大推論 |
| custom | カスタムトークン予算 |

## Deep Research 設計

### 貼り付け方式（初期実装・推奨）

1. アプリがプロンプトを生成
2. ユーザーが外部 AI（ChatGPT / Perplexity 等）に貼り付け
3. 結果をアプリに貼り戻し
4. `analysis_llm` で解析・構造化

### API 方式（OpenAI Deep Research）

- Responses API (`POST /v1/responses`) を使用
- `tools: [{"type": "web_search_preview"}]`
- `max_output_tokens: 16384`
- タイムアウト: 600 秒

### 2段階 Deep Research

1. **立ち位置調査**: 先行研究・新規性・研究領域を調査
2. **ジャーナル調査**: 立ち位置レポートを入力として投稿先候補を探索

## データ構造

### JournalCandidate

```rust
struct JournalCandidate {
    journal_name: String,
    publisher: String,
    scope_fit: String,
    article_type_fit: String,
    similar_articles: String,
    impact_factor_or_metric: String,
    quartile_or_rank: String,
    metric_source: String,
    metric_year: String,
    apc: String,
    word_limit: String,
    open_access_policy: String,
    pros: String,
    cons: String,
    recommendation_level: String,
    reason: String,
    source_evidence: String,
    match_score: u8,               // 0-100
    publication_route: String,
    apc_required: String,          // required / optional / no_apc / unknown
    apc_avoidance: String,
    recommended_submission_strategy: String,
    waiver_or_discount_info: String,
    cost_risk_level: String,       // low / medium / high / unknown
}
```

### LlmSlotConfig

```rust
struct LlmSlotConfig {
    name: String,                  // "analysis_llm" | "deep_research_provider"
    provider: String,              // "openai" | "deepseek" | "gemini" | ...
    api_format: ApiFormat,         // openai_compatible | anthropic | gemini | ollama
    base_url: String,
    model: String,
    api_key_env: String,           // 環境変数名のみ保存
    reasoning_enabled: bool,
    reasoning_mode: String,        // off | standard | extended | max | custom
    reasoning_budget: Option<u32>,
    model_list_source: String,     // static | api | local
}
```

## プロジェクトフォルダ構成

```
プロジェクト/
  project.json                  # プロジェクト情報
  source/
    manuscript.docx             # 元の docx ファイル
  data/
    manuscript_text.txt         # 抽出テキスト（全文）
    manuscript_text.json        # 抽出テキスト（構造化 JSON）
    summary.json                # 論文要約
    positioning_research.md     # 立ち位置調査結果（Deep Research 生データ）
    journal_research.md         # ジャーナル調査結果（Deep Research 生データ）
    journal_names.json          # 候補ジャーナル名リスト
    journals/
      001_journal_name.json     # 1誌ずつの詳細 JSON
    journals_failed/
      002_failed_journal.json   # 解析失敗した候補
    journals.json               # 統合済み候補
```

## 画面構成

```
🏠 ホーム         プロジェクト作成・選択
⚙️ API設定        LLM スロット設定（解析 LLM / Deep Research API）
📄 入力           docx テキスト抽出
📝 論文要約       構造化要約の生成・表示
🔬 立ち位置調査   先行研究上の立ち位置を調査（貼り付け / API）
🔍 ジャーナル調査 投稿先候補を探索（貼り付け / API）
📊 結果           レポート生成・エクスポート（Markdown / Word / JSON）
```

## ライセンス

MIT License
