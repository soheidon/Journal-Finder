# SPEC.md

# Journal Finder / ジャーナルファインダー 仕様書

## 0. この仕様書の位置づけ

本仕様書は、**Journal Finder / ジャーナルファインダー** の全体仕様を定義する。

本ツールは、研究者が投稿先ジャーナルを効率的に探索・評価するためのデスクトップアプリケーションである。docx 形式の論文原稿を入力し、LLM による構造化要約、Deep Research 結果の統合、ジャーナル候補の推薦・ランク付けを行う。

実装は以下を前提とする。

```text
GUI:          Tauri v2 + React + TypeScript
Backend/Core: Rust（Tauri commands 経由で呼び出し）
対象OS:       Windows 11 を主対象とする
入力:         docx ファイル（将来: テキスト直接貼り付け）
出力:         Markdown / JSON
```

本ツールは **Peer Review Assistant の設計・プロンプト・UI方針を参考** にしているが、**コードはすべて Rust + TypeScript で新規実装** する。Python は使用しない。既存 Peer Review Assistant の Python コードを流用することはない。

---

## 1. 名称

### 1.1 正式名称

**Journal Finder**

### 1.2 日本語名

**ジャーナルファインダー**

### 1.3 説明文

Journal Finder は、研究者が学術論文の投稿先ジャーナルを探索・評価するためのツールである。論文原稿を読み込み LLM で構造化要約を生成し、Deep Research 結果（外部 AI 検索結果の貼り付け、または将来の API 連携）を統合して、投稿候補ジャーナルを推薦する。

---

## 2. アーキテクチャ

### 2.1 採用技術

```text
Tauri v2 + React TypeScript + Rust
```

| 層 | 技術 | 役割 |
|---|---|---|
| GUI | Tauri v2 + React + TypeScript | 画面表示、ファイル選択、状態管理 |
| Backend/Core | Rust | docx テキスト抽出、LLM API 呼び出し、プロンプト生成、レポート出力 |
| 通信 | Tauri commands (`#[tauri::command]`) | GUI から Rust 関数を直接呼び出し |
| docx 処理 | Rust (`zip` + `quick-xml`) | docx を zip として展開し `word/document.xml` からテキスト抽出 |
| LLM API | Rust (`reqwest`) | 各 LLM プロバイダーの REST API を直接呼び出し |

### 2.2 Tauri GUI の役割

- 画面表示（サイドバー + パネル切替）
- ファイル選択（docx）
- プログレスバー表示
- Deep Research 結果貼り付けモーダル
- ジャーナル候補テーブル表示
- LLM 設定画面
- ログ表示

### 2.3 Rust コアの役割

- docx テキスト抽出（zip + XML パース）
- セクション分割（見出しパターンベース）
- LLM API 呼び出し（プロバイダー抽象化）
- プロンプト生成・管理
- 論文構造化要約
- Deep Research 結果パース・マージ
- ジャーナル候補ランク付け
- レポート生成（Markdown / JSON）

### 2.4 通信プロトコル

GUI から Rust の `#[tauri::command]` 関数を直接呼び出す。非同期処理は `async fn` で実装し、Tauri の IPC 経由で結果を返す。

```text
React GUI
  ↓  invoke("command_name", { args })
Tauri IPC
  ↓
Rust command handler
  ↓  core/ モジュール呼び出し
結果を JSON で返却
  ↓
React が state に反映
```

### 2.5 UTF-8 エンコーディング

Rust はデフォルトで UTF-8 を使用するため、特別なエンコーディング対策は不要。

---

## 3. プロジェクト管理

### 3.1 プロジェクトフォルダ構成

作業フォルダはリポジトリ外にユーザーが任意の場所を指定する。`project.json` の存在によりプロジェクトフォルダであることを識別する。

```text
project_folder/
  project.json
  manuscript_text.txt           # 抽出済みテキスト
  summary.json                  # 構造化要約
  search_prompt.txt             # 外部AI検索プロンプト（貼り付け方式用）
  external_results_a.txt        # 外部AI A の検索結果
  external_results_b.txt        # 外部AI B の検索結果
  journals.json                 # ジャーナル候補リスト
  journals.md                   # ジャーナル候補テーブル（Markdown）
  assessment.md                 # 適合性評価
  report.md                     # 最終レポート
  report.json                   # 最終レポート（JSON）
  source/
    manuscript.docx             # 元のdocxファイル
```

### 3.2 project.json スキーマ

```json
{
  "project_id": "string",
  "created_at": "ISO 8601",
  "updated_at": "ISO 8601",
  "source": {
    "docx_path": "source/manuscript.docx",
    "docx_sha256": "string",
    "docx_size_bytes": 0
  },
  "pipeline": {
    "extract_status": "not_started",
    "summary_status": "not_started",
    "search_status": "not_started",
    "report_status": "not_started"
  },
  "settings": {
    "target_journal": ""
  }
}
```

ステータス値: `not_started` / `in_progress` / `done` / `failed`

---

## 4. 入力ファイル

### 4.1 入力モード

| モード | 入力 | 説明 |
|---|---|---|
| docx（主モード） | docx ファイル | テキスト抽出は Rust 内部で自動実行 |
| テキスト貼り付け（将来） | プレーンテキスト | docx 処理をスキップしテキスト直接入力 |

内部処理はすべて `ManuscriptText` 構造体を受け取る設計とし、入力手段に依存しない。

### 4.2 docx テキスト抽出

`source/manuscript.docx` から全文テキストとセクションを抽出する。

```text
docx ファイル
  → zip として展開
  → word/document.xml を取得
  → XML パース（quick-xml）
  → <w:p> 要素を段落として取得
  → <w:t> ノードからテキスト結合
  → 見出しパターンでセクション分割
  → ManuscriptText を返す
```

セクション分割は見出しスタイル（`<w:pStyle w:val="Heading1">` 等）を優先し、見出しが検出できない場合は段落テキストの先頭パターン（"Abstract", "Introduction", "Methods" 等）で推定する。

---

## 5. パイプライン

### 5.1 概要

```text
1. テキスト抽出   docx → ManuscriptText
2. 論文要約       ManuscriptText → SummaryResult
3. Deep Research   貼り付け方式（優先） / API方式（将来）
4. ジャーナル推薦  SummaryResult + DR結果 → JournalCandidate[]
5. レポート出力    全結果 → Markdown / JSON
```

### 5.2 ステップ 1: テキスト抽出

```rust
// commands/docx.rs
#[tauri::command]
async fn extract_docx(path: String) -> Result<ManuscriptText, String>;
```

docx ファイルを zip として開き、`word/document.xml` からテキストを抽出する。最小実装として、段落分割とセクション推定のみ行う。

### 5.3 ステップ 2: 論文要約

```rust
// commands/summary.rs
#[tauri::command]
async fn generate_summary(manuscript: ManuscriptText) -> Result<SummaryResult, String>;
```

`summarizer` スロットの LLM を使用し、ManuscriptText から構造化要約を生成する。

**注意**: この段階では投稿先ジャーナルは未定であるため、`journal_name` 引数は受け取らない。ジャーナル情報は後の適合性評価で使用する。

出力 `SummaryResult`:

```rust
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
}
```

### 5.4 ステップ 3: Deep Research

Deep Research には 2 つの方式を想定する。

#### 方式 A: 貼り付け方式（初期実装・優先）

1. GUI が `SEARCH_EXTERNAL_PROMPT`（SummaryResult を埋め込み済み）を表示
2. ユーザーが ChatGPT / Perplexity / Claude 等の Web チャットに貼り付け
3. 得られた結果を PasteModal に貼り戻し（A/B 2 スロット対応）
4. `journal_assessor` スロットの LLM がパース・マージ・ランク付け

```rust
// commands/journal.rs
#[tauri::command]
async fn get_search_prompt(summary: SummaryResult) -> Result<String, String>;

#[tauri::command]
async fn parse_external_results(
    summary: SummaryResult,
    external_a: String,
    external_b: String,
) -> Result<Vec<JournalCandidate>, String>;
```

#### 方式 B: API 方式（将来実装）

将来的には ChatGPT API / Perplexity API / ブラウザ検索系 API 等と連携し、根拠リンク付きの Deep Research 相当の検索を実現する。

**初期版では mock または LLM-only candidate generation として実装する。** これは真の Deep Research ではなく、LLM の学習知識に基づく候補生成である。Web 検索による根拠リンク付きの調査は将来実装とする。

```rust
// commands/journal.rs（初期版: mock）
#[tauri::command]
async fn search_journals_api(
    summary: SummaryResult,
) -> Result<Vec<JournalCandidate>, String>;
```

### 5.5 ステップ 4: ジャーナル候補ランク付け

```rust
// core/ranking.rs
pub fn merge_and_rank(
    internal_results: Option<Vec<JournalCandidate>>,
    external_a: Option<Vec<JournalCandidate>>,
    external_b: Option<Vec<JournalCandidate>>,
) -> Vec<JournalCandidate>;
```

- ジャーナル名の正規化による重複排除
- `match_rate` 降順ソート
- 上位 10 件に絞り込み

### 5.6 ステップ 5: レポート出力

```rust
// commands/report.rs
#[tauri::command]
async fn generate_report(
    summary: SummaryResult,
    journals: Vec<JournalCandidate>,
    format: String,
) -> Result<String, String>;
```

出力形式: `md` / `json` / `all`

---

## 6. LLM 設定

### 6.1 LLM スロット

| スロット | 用途 |
|---|---|
| `summarizer` | 論文構造化要約の生成 |
| `journal_assessor` | Deep Research 結果のパース、マージ、推薦理由生成 |

2 スロットは同一プロバイダー・モデルでも異なる設定でもよい。

### 6.2 プロバイダー対応

| プロバイダー | API 形式 |
|---|---|
| OpenAI | OpenAI Chat Completions API |
| Anthropic | Anthropic Messages API（ヘッダー差異対応） |
| DeepSeek | OpenAI Compatible |
| OpenRouter | OpenAI Compatible |
| Ollama | OpenAI Compatible |
| Custom | OpenAI Compatible |

### 6.3 設定項目

各スロットに以下を設定する:

- プロバイダー
- Base URL
- モデル名
- API Key（OS の資格情報ストアに保存、平文保存禁止）

接続テストはスロットごとに実行可能。

### 6.4 API Key 管理

- OS の資格情報ストア（Windows Credential Manager / DPAPI）に保存
- `project.json` や平文設定ファイルには保存しない
- Rust 側では環境変数または Tauri の secure storage 経由で取得

---

## 7. データ構造

### 7.1 ManuscriptText

```rust
pub struct ManuscriptText {
    pub raw_text: String,
    pub sections: Vec<Section>,
    pub paragraph_count: usize,
    pub char_count: usize,
}

pub struct Section {
    pub heading: String,
    pub body: String,
}
```

### 7.2 SummaryResult

```rust
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
}
```

### 7.3 JournalCandidate

```rust
pub struct JournalCandidate {
    pub journal_name: String,
    pub publisher: String,
    pub impact_factor: String,
    pub submission_fee: String,
    pub match_rate: String,
    pub reason: String,
}
```

### 7.4 LlmSlot

```rust
pub struct LlmSlot {
    pub name: String,
    pub provider: LlmProvider,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

pub enum LlmProvider {
    OpenAI,
    Anthropic,
    DeepSeek,
    OpenRouter,
    Ollama,
    Custom,
}
```

---

## 8. GUI アーキテクチャ

### 8.1 画面構成

```
┌──────────┬──────────────────────────────────────┐
│ Sidebar  │ Progress Bar (ステップ表示)           │
│          ├──────────────────────────────────────┤
│ ホーム   │                                      │
│ 入力     │ Active Panel                         │
│ 検索     │ (home/input/journal/results/settings)│
│ 結果     │                                      │
│ 設定     │                                      │
│          ├──────────────────────────────────────┤
│          │ ▼ ログ (collapsible bottom pane)     │
└──────────┴──────────────────────────────────────┘
```

### 8.2 サイドバー

5 メニュー項目。アクティブ項目は青ハイライト + 左ボーダー。

| 項目 | キー | アイコン |
|---|---|---|
| ホーム | home | 🏠 |
| 入力 | input | 📄 |
| 検索 | journal | 🔍 |
| 結果 | results | 📊 |
| 設定 | settings | ⚙️ |

### 8.3 プログレスバー

4 ステップの水平ステップインジケーター。

```text
①テキスト抽出 → ②論文要約 → ③ジャーナル検索 → ④結果出力
```

各ステップは丸 + ラベルで表示。完了ステップは緑、現在ステップは青、未着手は灰色。クリックで該当パネルに遷移可能。

### 8.4 パネル一覧

| パネル | コンポーネント | 主な機能 |
|---|---|---|
| HomePanel | `panels/HomePanel.tsx` | LLM 接続テスト、進捗サマリー、次ステップ提案 |
| InputPanel | `panels/InputPanel.tsx` | docx 選択、テキスト抽出実行、抽出結果プレビュー（全文 / セクション表示） |
| JournalPanel | `panels/JournalPanel.tsx` | 論文要約生成、Deep Research モード選択（貼り付け / API）、ジャーナル検索実行、候補テーブル表示 |
| ResultsPanel | `panels/ResultsPanel.tsx` | 最終レポート表示（Markdown）、JSON エクスポート、ファイル保存 |
| SettingsPanel | `panels/SettingsPanel.tsx` | LLM スロット設定（summarizer / journal_assessor）、プロバイダー選択、モデル設定、API Key、接続テスト |

### 8.5 PasteModal（貼り付けモーダル）

Deep Research 結果貼り付け用のモーダルダイアログ。

- モーダル上部に検索プロンプト（コピー可能）を表示
- A / B 2 スロットのテキストエリア（外部 AI の結果を貼り付け）
- B は任意（1 つの AI 結果のみでも可）
- 「解析実行」ボタンで `parse_external_results` を呼び出し

### 8.6 ボトムログペイン

常時表示の折りたたみ可能なログ領域。

- 展開時: 高さ 150px、全ログ行を表示、「ログを消去」ボタン
- 折りたたみ時: 高さ 28px、ログ件数バッジを表示
- ヘッダークリックで展開 / 折りたたみ切替

### 8.7 状態管理

全状態は `App.tsx` に `useState` で集約。React Context 不使用。各パネルは純粋なプレゼンテーションコンポーネントで、必要な props のみ受け取る。

主な状態カテゴリ:

- **ナビゲーション**: `activeView`
- **プロジェクト**: `projectPath`, `docxPath`
- **パイプライン状態**: `extractStatus`, `summaryStatus`, `searchStatus`
- **データ**: `manuscriptText`, `summaryResult`, `journals`, `reportContent`
- **LLM 設定**: `llmSlots`（2 スロット）
- **フィードバック**: `statusMessage`（8 秒で自動消去）
- **ログ**: `logs`, `logExpanded`

### 8.8 ステータス表示パターン

各操作行は以下の統一パターンで表示する:

```
[操作名]  [実行ボタン]  [状態チップ]  [補足情報]
```

ステータスチップ:

| 状態 | ラベル | CSS class |
|---|---|---|
| 未実行 | 未実行 | `status-chip.unrun`（灰） |
| 実行中 | 実行中... | `status-chip.running`（青） |
| 完了 | 完了 | `status-chip.ok`（緑） |
| エラー | エラー | `status-chip.err`（赤） |

ステータスメッセージバナー: 各パネル上部に表示。成功（緑）/ エラー（赤）/ 情報（青）。8 秒で自動消去。

---

## 9. Tauri Commands 一覧

```rust
// commands/docx.rs
#[tauri::command]
async fn extract_docx(path: String) -> Result<ManuscriptText, String>;

// commands/summary.rs
#[tauri::command]
async fn generate_summary(manuscript: ManuscriptText) -> Result<SummaryResult, String>;

// commands/journal.rs
#[tauri::command]
async fn get_search_prompt(summary: SummaryResult) -> Result<String, String>;

#[tauri::command]
async fn search_journals_api(summary: SummaryResult) -> Result<Vec<JournalCandidate>, String>;

#[tauri::command]
async fn parse_external_results(
    summary: SummaryResult,
    external_a: String,
    external_b: String,
) -> Result<Vec<JournalCandidate>, String>;

// commands/llm.rs
#[tauri::command]
async fn test_llm_connection(slot: LlmSlot) -> Result<bool, String>;

// commands/report.rs
#[tauri::command]
async fn generate_report(
    summary: SummaryResult,
    journals: Vec<JournalCandidate>,
    format: String,
) -> Result<String, String>;
```

---

## 10. Rust モジュール構成

```text
src-tauri/src/
  main.rs                         # Tauri 起動 + コマンド登録
  commands/
    mod.rs
    docx.rs                       # extract_docx コマンドハンドラ
    llm.rs                        # test_llm_connection コマンドハンドラ
    summary.rs                    # generate_summary コマンドハンドラ
    journal.rs                    # get_search_prompt / search_journals_api / parse_external_results
    report.rs                     # generate_report コマンドハンドラ
  core/
    mod.rs
    manuscript.rs                 # ManuscriptText / Section / SummaryResult / JournalCandidate 構造体
    docx_extract.rs               # zip 展開 + XML パース + テキスト抽出
    prompts.rs                    # 全 LLM プロンプト定数
    llm_client.rs                 # LLM API 呼び出しクライアント（プロバイダー抽象化）
    deep_research.rs              # DR 貼り付け方式パース + API 方式（mock）
    journal_profile.rs            # ジャーナルプロファイル JSON スキーマ定義
    ranking.rs                    # ジャーナル候補マージ・ランク付け
    summary.rs                    # 要約ロジック（LLM 呼び出し + レスポンスパース）
    report.rs                     # Markdown / JSON レポート生成
```

---

## 11. セキュリティ

### 11.1 基本方針

論文原稿は機密情報である可能性がある。

### 11.2 API Key 管理

- OS の資格情報ストアに保存
- `project.json` や設定ファイルへの平文保存禁止
- 設定画面ではマスク表示（`••••••••`）

### 11.3 データ送信

- LLM API への原稿テキスト送信時、ユーザーに確認を求める
- ローカル LLM（Ollama 等）の利用オプションを提供

---

## 12. エラー処理

### 12.1 基本方針

1. 1 つの処理が失敗してもプロジェクト全体を破棄しない
2. 成功した出力は保存し再利用可能にする
3. 失敗した処理だけを再実行可能にする

### 12.2 リトライ

LLM API 呼び出しは最大 3 回の自動リトライ（指数バックオフ: 2 秒 → 5 秒 → 10 秒）。3 回失敗で `failed` として記録。

---

## 13. 既存 Peer Review Assistant からの借用

本ツールは Peer Review Assistant の設計を参考にするが、**コードはすべて新規実装** する。以下の要素を借用する:

| 借用元 | 借用内容 | 形式 |
|---|---|---|
| `novelty/prompts.py` | プロンプト文面 | Rust 文字列定数に移植 |
| `journal_profile.py` | ジャーナルプロファイル JSON スキーマ構造 | Rust 構造体に移植 |
| `NoveltyCheckPanel.tsx` | 5 フェーズ UI の考え方 | 4 フェーズに簡略化して参考 |
| `DeepResearchModal.tsx` | 貼り付けモーダルの UI 設計 | PasteModal として Rust + React に移植 |
| `SettingsPanel.tsx` | LLM スロット設定の UI パターン | 2 スロット構成に簡略化 |
| SPEC.md | 画面構成、ステータス表示パターン、エラー処理方針 | 設計方針として継承 |

**借用しないもの**: Python コード、Click CLI、subprocess 通信、NDJSON プロトコル

---

## 14. 実装順序

| 順 | 内容 | 依存 |
|---|---|---|
| 1 | Tauri プロジェクト初期化 + 基本 GUI 骨格（Sidebar + ProgressBar + パネル切替） | なし |
| 2 | SettingsPanel + settings.json の最小実装（LLM スロット定義、保存・読込） | なし |
| 3 | `llm_client.rs` + LLM 接続テストコマンド | 2 |
| 4 | `docx_extract.rs` + `extract_docx` コマンド + InputPanel | なし |
| 5 | `prompts.rs` — プロンプト定数定義 | なし |
| 6 | `summary.rs` + `generate_summary` コマンド + JournalPanel 要約部分 | 3, 5 |
| 7 | PasteModal + `deep_research.rs` 貼り付け方式 + `parse_external_results` | 3, 5 |
| 8 | `ranking.rs` — 候補マージ・ランク付け + JournalPanel 候補表示 | 7 |
| 9 | ResultsPanel + `report.rs` — レポート出力 | 6, 8 |
| 10 | `deep_research.rs` API 方式（mock / LLM-only 候補生成） | 3, 5 |

---

## 15. 推奨プロジェクト構成

```text
journal-finder/
  README.md
  SPEC.md
  LICENSE

  app/
    package.json
    vite.config.ts
    tsconfig.json
    src/
      main.tsx
      App.tsx
      App.css
      Sidebar.tsx
      ProgressBar.tsx
      panels/
        HomePanel.tsx
        InputPanel.tsx
        JournalPanel.tsx
        ResultsPanel.tsx
        SettingsPanel.tsx
        LogPanel.tsx
      components/
        PasteModal.tsx
        JournalTable.tsx
        LlmSlotConfig.tsx
    src-tauri/
      Cargo.toml
      tauri.conf.json
      build.rs
      src/
        main.rs
        commands/
          mod.rs
          docx.rs
          llm.rs
          summary.rs
          journal.rs
          report.rs
        core/
          mod.rs
          manuscript.rs
          docx_extract.rs
          prompts.rs
          llm_client.rs
          deep_research.rs
          journal_profile.rs
          ranking.rs
          summary.rs
          report.rs
```

---

## 16. 設計上の重要リスク

1. **docx テキスト抽出は不完全になりうる** — 複雑なレイアウト（2 段組み、図表配置）では正しく抽出できない場合がある。最小実装では見出しスタイルベースのセクション分割に限定し、失敗時は全文テキストとして処理を継続する。
2. **LLM 出力のフォーマットが不安定** — JSON パース失敗時はリトライし、最終的に失敗した場合はエラーメッセージを表示する。
3. **Deep Research 貼り付け方式はユーザー依存** — 外部 AI の出力品質に左右される。パース・マージ処理で可能な限り正規化する。
4. **API 方式は初期版では真の Deep Research ではない** — LLM の学習知識に基づく候補生成であり、Web 検索による根拠リンク付きの調査ではないことを UI 上で明示する。
5. **API Key の平文保存禁止** — OS 資格情報ストアを使用する。
