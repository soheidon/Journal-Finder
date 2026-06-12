# 論文投稿先アドバイザ

学術論文の投稿先ジャーナルを、費用面も含めて総合的に推薦するデスクトップアプリケーションです。

## このアプリでできること

- **docx ファイルを読み込み**、論文の構造化要約を自動生成
- **先行研究上の立ち位置**を Deep Research で調査
- **投稿先ジャーナル候補**を Deep Research で探索
- 各候補の **APC（論文処理費）、掲載方法、waiver 情報、低コスト投稿戦略** を評価
- **Markdown / Word / JSON** 形式でレポートを出力

## インストール

### Windows

[Releases](https://github.com/soheidon/Journal-Finder/releases) ページから最新の `.msi` ファイルをダウンロードしてインストールしてください。

### ソースコードからのビルド

```bash
git clone https://github.com/soheidon/Journal-Finder.git
cd Journal-Finder/app
npm install
npm run tauri build
```

## 使い方

### 1. API キーの設定

本アプリは LLM API を使用します。API キーは環境変数で管理します（設定ファイルには保存されません）。

**Windows の場合**、PowerShell で以下を実行してからアプリを起動してください：

```powershell
$env:DEEPSEEK_API_KEY="sk-your-key"
$env:OPENAI_API_KEY="sk-your-key"
```

または、Windows の「システムの詳細設定」→「環境変数」に設定すると、毎回の入力が不要になります。

対応 Provider と環境変数名：

| Provider | 環境変数 |
|---|---|
| OpenAI | `OPENAI_API_KEY` |
| DeepSeek | `DEEPSEEK_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Gemini | `GEMINI_API_KEY` |
| Kimi (Moonshot) | `MOONSHOT_API_KEY` |
| Xiaomi MiMo | `MIMO_API_KEY` |
| MiniMax | `MINIMAX_API_KEY` |
| OpenRouter | `OPENROUTER_API_KEY` |
| Perplexity | `PERPLEXITY_API_KEY` |
| Ollama | 不要（ローカル実行） |

### 2. プロジェクトの作成

ホーム画面で「新規プロジェクト」をクリックし、プロジェクト名と保存先フォルダを選択します。

### 3. 論文の読み込み

「入力」タブで docx ファイルを選択し、「テキスト抽出」を実行します。

### 4. 論文要約

「論文要約」タブで「論文要約を生成」をクリックします。LLM が論文を読み込み、研究テーマ・目的・方法・結果などを構造化要約として生成します。

### 5. 立ち位置調査

「立ち位置調査」タブで、先行研究上の立ち位置を調べます。

**貼り付け方式（推奨・無料）**:
1. 「プロンプトを生成」→「コピー」
2. ChatGPT / Perplexity / Gemini / Claude 等の Deep Research に貼り付け
3. 結果をテキストエリアに貼り戻し →「保存」

**API 方式（上級者向け・有料）**:
- OpenAI Deep Research API を使用（Responses API）
- 実行には数分〜数十分、費用がかかる場合があります

### 6. ジャーナル調査

「ジャーナル調査」タブで、投稿先候補を探索します。立ち位置調査の結果を使って、より精度の高い候補を取得できます。

**貼り付け方式（推奨・無料）**:
1. 「プロンプトを生成」→「コピー」
2. 外部 AI の Deep Research に貼り付け
3. 結果を貼り戻し →「保存」→「解析」

**API 方式（上級者向け・有料）**:
- OpenAI Deep Research API を使用

解析後、候補ジャーナルの一覧が表示されます。各行をクリックすると詳細（APC、掲載方法、waiver 情報、投稿戦略）が表示されます。

### 7. 結果の確認・出力

「結果」タブで、レポートを Markdown / Word / JSON 形式で保存できます。

## Deep Research API の費用について

本アプリの貼り付け方式は無料ですが、API 方式は費用が発生します。

| Provider | 費用の目安 |
|---|---|
| OpenAI Deep Research | トークン料金 + web_search tool call の回数分 |
| Perplexity Sonar Deep Research | 入出力トークン + citation tokens + search queries + reasoning tokens |
| Gemini Grounding | トークン料金 + 検索クエリ数に応じた課金 |
| Claude Web Search | トークン料金 + Web search $10/1,000 searches |

**推奨**: まずは貼り付け方式（無料）で試し、必要に応じて API 方式を利用してください。

## 費用を抑える投稿戦略

本アプリは「できるだけ安価に適切なジャーナルへ投稿すること」を目的としています。各候補について以下を評価します：

- **APC が不要な候補**: Subscription journal / 非 OA 投稿
- **APC が回避可能な候補**: Hybrid OA で非 OA 選択、waiver / discount
- **APC が必須の候補**: Gold OA、費用リスクが高い
- **所属機関の Read & Publish 契約**: 大学図書館や研究支援部門に確認

## ライセンス

MIT License - [LICENSE](LICENSE) を参照してください。
