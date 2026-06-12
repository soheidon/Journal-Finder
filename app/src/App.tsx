import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./Sidebar";
import ProgressBar from "./ProgressBar";
import HomePanel from "./panels/HomePanel";
import InputPanel from "./panels/InputPanel";
import JournalPanel from "./panels/JournalPanel";
import ResultsPanel from "./panels/ResultsPanel";
import SettingsPanel from "./panels/SettingsPanel";
import LogPanel from "./panels/LogPanel";

export type ActiveView = "home" | "input" | "journal" | "results" | "settings";

export interface ManuscriptText {
  raw_text: string;
  sections: { heading: string; body: string }[];
  paragraphs: string[];
  paragraph_count: number;
  char_count: number;
}

export interface SummaryResult {
  research_topic: string;
  objective: string;
  sample_summary: string;
  design: string;
  methods_summary: string;
  measures: string;
  statistics: string;
  findings: string;
  claimed_contributions: string;
  keywords_for_search: string[];
  raw_response: string;
}

export interface JournalCandidate {
  journal_name: string;
  publisher: string;
  scope_fit: string;
  article_type_fit: string;
  similar_articles: string;
  impact_factor_or_metric: string;
  apc: string;
  word_limit: string;
  open_access_policy: string;
  pros: string;
  cons: string;
  recommendation_level: string;
  reason: string;
  source_evidence: string;
  match_score: number;
  publication_route: string;
  apc_required: string;
  apc_avoidance: string;
  recommended_submission_strategy: string;
  waiver_or_discount_info: string;
  cost_risk_level: string;
  raw_response: string;
}

export type PipelineStatus = "not_started" | "in_progress" | "done" | "failed";

export type LlmProvider = "openai" | "anthropic" | "deepseek" | "openrouter" | "ollama" | "gemini" | "kimi" | "mimo" | "minimax" | "custom";
export type ApiFormat = "openai_compatible" | "anthropic" | "gemini" | "ollama";
export type ReasoningMode = "off" | "standard" | "extended" | "max" | "custom";

export interface LlmSlotConfig {
  name: string;
  provider: LlmProvider;
  api_format: ApiFormat;
  base_url: string;
  model: string;
  api_key_env: string;
  reasoning_enabled: boolean;
  reasoning_mode: ReasoningMode;
  reasoning_budget: number | null;
  model_list_source: "static" | "api" | "local";
}

export interface ModelInfo {
  id: string;
  name: string;
  description: string;
}

export interface ModelListResult {
  ok: boolean;
  models: ModelInfo[];
  message: string;
}

export interface LlmTestResult {
  ok: boolean;
  message: string;
  latency_ms: number;
  url: string;
  http_status: number;
  model_used: string;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface ReportData {
  markdown: string;
  json: string;
}

export interface ProjectInfo {
  name: string;
  path: string;
  created_at: string;
  updated_at: string;
  docx_file: string;
  has_summary: boolean;
  has_journals: boolean;
}

function App() {
  const [activeView, setActiveView] = useState<ActiveView>("home");
  const [projectInfo, setProjectInfo] = useState<ProjectInfo | null>(null);
  const [docxPath, setDocxPath] = useState<string>("");
  const [manuscriptText, setManuscriptText] = useState<ManuscriptText | null>(null);
  const [summaryResult, setSummaryResult] = useState<SummaryResult | null>(null);
  const [journals, setJournals] = useState<JournalCandidate[]>([]);
  const [extractStatus, setExtractStatus] = useState<PipelineStatus>("not_started");
  const [summaryStatus, setSummaryStatus] = useState<PipelineStatus>("not_started");
  const [searchStatus, setSearchStatus] = useState<PipelineStatus>("not_started");
  const [searchPrompt, setSearchPrompt] = useState<string>("");
  const [positioningPrompt, setPositioningPrompt] = useState<string>("");
  const [positioningResult, setPositioningResult] = useState<string>("");
  const [journalSearchPrompt, setJournalSearchPrompt] = useState<string>("");
  const [statusMessage, setStatusMessage] = useState<{ type: "success" | "error" | "info"; text: string } | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [logExpanded, setLogExpanded] = useState(false);
  const [llmTestResults, setLlmTestResults] = useState<Record<string, LlmTestResult>>({});

  const addLog = useCallback((message: string) => {
    const now = new Date().toLocaleTimeString("ja-JP");
    setLogs((prev) => [...prev, `[${now}] ${message}`]);
  }, []);

  const showStatus = useCallback((type: "success" | "error" | "info", text: string) => {
    setStatusMessage({ type, text });
    setTimeout(() => setStatusMessage(null), 8000);
  }, []);

  const handleExtractDocx = useCallback(async () => {
    if (!docxPath) return;
    setExtractStatus("in_progress");
    addLog(`docx テキスト抽出開始: ${docxPath}`);
    try {
      const result = await invoke<ManuscriptText>("extract_docx", { path: docxPath });
      setManuscriptText(result);
      setExtractStatus("done");
      addLog(`テキスト抽出完了: ${result.paragraph_count} 段落, ${result.char_count} 文字, ${result.sections.length} セクション`);
      showStatus("success", `テキスト抽出完了: ${result.paragraph_count} 段落, ${result.char_count} 文字`);
      // Save to project folder
      if (projectInfo) {
        try {
          // Copy docx to source/
          await invoke("copy_to_project", { projectDir: projectInfo.path, sourcePath: docxPath, destSubdir: "source", destFilename: "manuscript.docx" });
          // Save extracted text to data/
          await invoke("save_project_file", { projectDir: projectInfo.path, filename: "data/manuscript_text.txt", content: result.raw_text });
          await invoke("save_project_file", { projectDir: projectInfo.path, filename: "data/manuscript_text.json", content: JSON.stringify(result, null, 2) });
          await invoke("save_project", { info: { ...projectInfo, docx_file: "source/manuscript.docx" } });
          addLog("プロジェクトフォルダに保存しました");
        } catch (_) {}
      }
    } catch (e) {
      setExtractStatus("failed");
      addLog(`テキスト抽出エラー: ${e}`);
      showStatus("error", `テキスト抽出に失敗しました: ${e}`);
    }
  }, [docxPath, projectInfo, addLog, showStatus]);

  const handleGenerateSummary = useCallback(async () => {
    if (!manuscriptText) return;
    setSummaryStatus("in_progress");
    addLog("論文要約生成開始（LLM 呼び出し中）");
    try {
      const result = await invoke<SummaryResult>("generate_summary", { manuscript: manuscriptText });
      setSummaryResult(result);
      setSummaryStatus("done");
      addLog(`要約完了: ${result.research_topic}`);
      showStatus("success", "論文要約が完了しました");
      // Save to project folder
      if (projectInfo) {
        try {
          await invoke("save_project_file", { projectDir: projectInfo.path, filename: "data/summary.json", content: JSON.stringify(result, null, 2) });
          await invoke("save_project", { info: { ...projectInfo, has_summary: true } });
          addLog("要約をプロジェクトフォルダに保存しました");
        } catch (_) {}
      }
    } catch (e) {
      setSummaryStatus("failed");
      addLog(`要約エラー: ${e}`);
      showStatus("error", `要約に失敗しました: ${e}`);
    }
  }, [manuscriptText, projectInfo, addLog, showStatus]);

  const handleGetPositioningPrompt = useCallback(async () => {
    if (!summaryResult) return;
    try {
      addLog("Step 1: 立ち位置調査プロンプト生成開始");
      const prompt = await invoke<string>("get_positioning_prompt", { summary: summaryResult });
      setPositioningPrompt(prompt);
      addLog("Step 1: プロンプト生成完了");
      showStatus("info", "プロンプトを生成しました。外部 AI に貼り付けてください。");
    } catch (e) {
      addLog(`プロンプト生成エラー: ${e}`);
      showStatus("error", `プロンプト生成に失敗しました: ${e}`);
    }
  }, [summaryResult, addLog, showStatus]);

  const handleGetJournalSearchPrompt = useCallback(async () => {
    if (!summaryResult) return;
    try {
      addLog("Step 2: ジャーナル調査プロンプト生成開始");
      const prompt = await invoke<string>("get_journal_search_prompt", { summary: summaryResult, positioning: positioningResult });
      setJournalSearchPrompt(prompt);
      addLog("Step 2: プロンプト生成完了");
      showStatus("info", "プロンプトを生成しました。外部 AI に貼り付けてください。");
    } catch (e) {
      addLog(`プロンプト生成エラー: ${e}`);
      showStatus("error", `プロンプト生成に失敗しました: ${e}`);
    }
  }, [summaryResult, positioningResult, addLog, showStatus]);

  const handleParseExternalResults = useCallback(async (externalA: string, externalB: string) => {
    if (!summaryResult || !externalA.trim()) return;
    setSearchStatus("in_progress");
    addLog("外部 Deep Research 結果の解析開始");
    try {
      const result = await invoke<JournalCandidate[]>("parse_external_results", {
        summary: summaryResult,
        externalA,
        externalB: externalB.trim() || null,
      });
      setJournals(result);
      setSearchStatus("done");
      addLog(`解析完了: ${result.length} 件のジャーナル候補`);
      showStatus("success", `${result.length} 件のジャーナル候補が見つかりました`);
      // Save to project folder
      if (projectInfo) {
        try {
          await invoke("save_project_file", { projectDir: projectInfo.path, filename: "data/journals.json", content: JSON.stringify(result, null, 2) });
          await invoke("save_project", { info: { ...projectInfo, has_journals: true } });
          addLog("ジャーナル候補をプロジェクトフォルダに保存しました");
        } catch (_) {}
      }
    } catch (e) {
      setSearchStatus("failed");
      addLog(`解析エラー: ${e}`);
      showStatus("error", `解析に失敗しました: ${e}`);
    }
  }, [summaryResult, projectInfo, addLog, showStatus]);

  const handleExportReport = useCallback(async (content: string, defaultFilename: string, format: string) => {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const path = await save({
        defaultPath: defaultFilename,
        filters: [{ name: format === "md" ? "Markdown" : "JSON", extensions: [format] }],
      });
      if (path) {
        await invoke("export_report", { path, content });
        addLog(`レポート保存完了: ${path}`);
        showStatus("success", `レポートを保存しました: ${path}`);
      }
    } catch (e) {
      addLog(`レポート保存エラー: ${e}`);
      showStatus("error", `レポートの保存に失敗しました: ${e}`);
    }
  }, [addLog, showStatus]);

  const handleCreateProject = useCallback(async (name: string, parentDir: string) => {
    try {
      const info = await invoke<ProjectInfo>("create_project", { name, parentDir });
      setProjectInfo(info);
      await invoke("add_recent_project", { info });
      addLog(`プロジェクト作成: ${info.path}`);
      showStatus("success", `プロジェクト「${name}」を作成しました`);
    } catch (e) {
      addLog(`プロジェクト作成エラー: ${e}`);
      showStatus("error", `${e}`);
    }
  }, [addLog, showStatus]);

  const handleOpenProject = useCallback(async (path: string) => {
    try {
      const info = await invoke<ProjectInfo>("open_project", { path });
      setProjectInfo(info);
      await invoke("add_recent_project", { info });
      addLog(`プロジェクト開く: ${info.path}`);
      showStatus("info", `プロジェクト「${info.name}」を開きました`);

      // Restore saved data
      try {
        const manuscriptJson = await invoke<string>("load_project_file", { projectDir: path, filename: "data/manuscript_text.json" });
        const ms = JSON.parse(manuscriptJson) as ManuscriptText;
        setManuscriptText(ms);
        setExtractStatus("done");
        addLog(`テキスト復元: ${ms.paragraph_count} 段落`);
      } catch (_) { /* no saved text */ }

      try {
        const summaryJson = await invoke<string>("load_project_file", { projectDir: path, filename: "data/summary.json" });
        const sr = JSON.parse(summaryJson) as SummaryResult;
        setSummaryResult(sr);
        setSummaryStatus("done");
        addLog(`要約復元: ${sr.research_topic}`);
      } catch (_) { /* no saved summary */ }

      try {
        const journalsJson = await invoke<string>("load_project_file", { projectDir: path, filename: "data/journals.json" });
        const js = JSON.parse(journalsJson) as JournalCandidate[];
        setJournals(js);
        setSearchStatus("done");
        addLog(`ジャーナル候補復元: ${js.length} 件`);
      } catch (_) { /* no saved journals */ }
    } catch (e) {
      addLog(`プロジェクト読み込みエラー: ${e}`);
      showStatus("error", `${e}`);
    }
  }, [addLog, showStatus]);

  const currentStep = extractStatus === "done"
    ? summaryStatus === "done"
      ? searchStatus === "done" ? 3 : 2
      : 1
    : 0;

  return (
    <div className="app-layout">
      <Sidebar activeView={activeView} onNavigate={setActiveView} />
      <div className="main-area">
        <ProgressBar currentStep={currentStep} onStepClick={(step) => {
          const views: ActiveView[] = ["input", "input", "journal", "results"];
          setActiveView(views[step]);
        }} />
        {statusMessage && (
          <div className={`status-banner ${statusMessage.type}`}>
            {statusMessage.text}
          </div>
        )}
        <div className="panel-area">
          {activeView === "home" && (
            <HomePanel
              onNavigate={setActiveView}
              projectInfo={projectInfo}
              onCreateProject={handleCreateProject}
              onOpenProject={handleOpenProject}
            />
          )}
          {activeView === "input" && (
            <InputPanel
              docxPath={docxPath}
              onDocxPathChange={setDocxPath}
              manuscriptText={manuscriptText}
              extractStatus={extractStatus}
              onExtract={handleExtractDocx}
            />
          )}
          {activeView === "journal" && (
            <JournalPanel
              summaryResult={summaryResult}
              journals={journals}
              summaryStatus={summaryStatus}
              searchStatus={searchStatus}
              positioningPrompt={positioningPrompt}
              positioningResult={positioningResult}
              journalSearchPrompt={journalSearchPrompt}
              onGenerateSummary={handleGenerateSummary}
              onGetPositioningPrompt={handleGetPositioningPrompt}
              onSetPositioningResult={setPositioningResult}
              onGetJournalSearchPrompt={handleGetJournalSearchPrompt}
              onParseExternalResults={handleParseExternalResults}
            />
          )}
          {activeView === "results" && (
            <ResultsPanel
              summaryResult={summaryResult}
              journals={journals}
              addLog={addLog}
              onExport={handleExportReport}
            />
          )}
          {activeView === "settings" && (
            <SettingsPanel addLog={addLog} showStatus={showStatus} testResults={llmTestResults} onTestResultsChange={setLlmTestResults} />
          )}
        </div>
        <LogPanel logs={logs} expanded={logExpanded} onToggle={() => setLogExpanded(!logExpanded)} onClear={() => setLogs([])} />
      </div>
    </div>
  );
}

export default App;
