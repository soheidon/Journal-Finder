# 論文投稿先アドバイザ

論文原稿を読み込み、投稿先ジャーナル候補を推薦するデスクトップアプリケーションです。

## 概要

- docx 形式の論文原稿を入力
- LLM による構造化要約を生成
- Deep Research 結果（外部 AI 貼り付け方式 / API 方式）を統合
- 投稿候補ジャーナルを推薦・ランク付け

## 技術スタック

| 層 | 技術 |
|---|---|
| GUI | Tauri v2 + React + TypeScript |
| Backend/Core | Rust |
| 通信 | Tauri commands |

## 動作環境

- **OS**: Windows 11（主対象）
- **Node.js**: 20+
- **Rust**: 1.70+

## セットアップ

### 1. リポジトリのクローン

```bash
git clone <repository-url>
cd Journal-Finder
```

### 2. フロントエンドの依存関係インストール

```bash
cd app
npm install
```

### 3. 環境変数の設定（API Key）

API Key は環境変数で管理します。設定ファイルには環境変数名のみ保存され、API Key の実値は保存されません。

ターミナルで以下を実行してからアプリを起動してください：

```bash
# OpenAI
set OPENAI_API_KEY=sk-your-key-here

# DeepSeek
set DEEPSEEK_API_KEY=sk-your-key-here

# Anthropic
set ANTHROPIC_API_KEY=sk-ant-your-key-here

# Gemini (Google AI Studio)
set GEMINI_API_KEY=your-key-here

# Kimi (Moonshot)
set MOONSHOT_API_KEY=your-key-here

# Xiaomi MiMo
set MIMO_API_KEY=your-key-here

# MiniMax
set MINIMAX_API_KEY=your-key-here

# OpenRouter
set OPENROUTER_API_KEY=sk-or-your-key-here
```

または、Windows の「システム環境変数」に設定すると、毎回の入力が不要になります。

**Ollama** はローカルで動作するため API Key 不要です。

### 4. 開発モードで起動

```bash
npm run tauri dev
```

初回起動時は Rust のコンパイルに時間がかかります（数分）。

### 5. ビルド（配布用実行ファイル作成）

```bash
npm run tauri build
```

ビルド成果物は `app/src-tauri/target/release/bundle/` に出力されます。

## 対応 LLM Provider

| Provider | API 形式 | 環境変数 | 推論モード | モデル一覧取得 |
|---|---|---|---|---|
| OpenAI | OpenAI Compatible | `OPENAI_API_KEY` | — | API (`/v1/models`) |
| DeepSeek | OpenAI Compatible | `DEEPSEEK_API_KEY` | — | 静的プリセット |
| Gemini | Gemini API | `GEMINI_API_KEY` | thinking / extended / max | API (`v1beta/models`) |
| Anthropic | Anthropic Messages | `ANTHROPIC_API_KEY` | extended thinking | 静的プリセット |
| Kimi (Moonshot) | OpenAI Compatible | `MOONSHOT_API_KEY` | thinking / extended | 静的プリセット |
| Xiaomi MiMo | OpenAI Compatible | `MIMO_API_KEY` | — | 静的プリセット |
| MiniMax | OpenAI Compatible | `MINIMAX_API_KEY` | — | 静的プリセット |
| OpenRouter | OpenAI Compatible | `OPENROUTER_API_KEY` | — | API (`/api/v1/models`) |
| Ollama | Ollama | 不要 | — | Local (`/api/tags`) |

## 推論モード設定

Gemini / Anthropic / Kimi では、thinking / reasoning モードを設定できます。

| モード | 説明 |
|---|---|
| Off | 推論無効 |
| Standard | 標準的な推論 |
| Extended | 拡張推論（より多くのトークンを使用） |
| Max | 最大推論 |
| Custom | カスタムトークン予算を指定 |

**注意**: DeepSeek / MiMo / MiniMax では推論モード設定は未対応またはモデル側で自動判定されます。

## Deep Research API の費用注意

Deep Research API は通常の LLM API より高額になる可能性があります。

| Provider | 費用構造 |
|---|---|
| Perplexity Sonar Deep Research | 入出力トークン + citation tokens + search queries + reasoning tokens |
| OpenAI Deep Research | Responses API + web_search tool call（検索回数分加算） |
| Gemini Grounding | トークン料金 + 検索クエリ数に応じた課金 |
| Claude Web Search | 通常トークン料金 + Web search $10/1,000 searches |

**推奨**: まずは外部 Deep Research 貼り付け方式（無料）で試し、必要に応じて API 方式を利用してください。

## プロジェクトフォルダ構成

各プロジェクトのフォルダ構成：

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
    journal_names.json          # 抽出された候補ジャーナル名リスト
    journals/
      001_child_abuse_neglect.json
      002_evolution_and_human_behavior.json
    journals_failed/
      004_child_maltreatment_error.json
    journals.json               # 統合統合済みジャーナル候補
```

- `positioning_research.md`: 先行研究上の立ち位置を調査した Deep Research の生結果
- `journal_research.md`: 投稿先ジャーナルを調査した Deep Research の生結果
- `journal_names.json`: Deep Research 結果から抽出した候補ジャーナル名リスト
- `journals/*.json`: 1誌ずつの詳細 JSON（マッチ度、APC、投稿戦略を含む）
- `journals_failed/*.json`: 解析に失敗した候補のエラー情報
- `journals.json`: 全候補を統合した結果（結果画面表示用）

## 開発

```bash
# フロントエンドのみ（Vite 開発サーバー）
npm run dev

# Tauri アプリ全体（フロントエンド + Rust バックエンド）
npm run tauri dev
```

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) を参照してください。
