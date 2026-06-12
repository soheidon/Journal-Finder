import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SummaryResult, JournalCandidate, PipelineStatus, ProjectInfo } from "../App";
import DeepResearchApiRunner from "../components/DeepResearchApiRunner";
import { translateRecommendationLevel, translateCostRisk, translateApcRequired, translateCostRiskClass } from "../utils/translate";

type SortKey = "match_score" | "cost_risk" | "recommendation";
type CostFilter = "all" | "low" | "medium" | "no_apc";

interface JournalSearchPanelProps {
  summaryResult: SummaryResult | null;
  summaryStatus: PipelineStatus;
  positioningResult: string;
  journals: JournalCandidate[];
  searchStatus: PipelineStatus;
  journalSearchPrompt: string;
  projectInfo: ProjectInfo | null;
  onGetJournalSearchPrompt: () => void;
  onParseExternalResults: (externalA: string, externalB: string) => void;
  onNavigateToPositioning: () => void;
}

const statusLabels: Record<PipelineStatus, { label: string; cls: string }> = {
  not_started: { label: "未実行", cls: "unrun" },
  in_progress: { label: "実行中...", cls: "running" },
  done: { label: "完了", cls: "ok" },
  failed: { label: "エラー", cls: "err" },
};

const costRiskOrder: Record<string, number> = { low: 0, medium: 1, high: 2, unknown: 3 };
const recOrder: Record<string, number> = { strong: 0, moderate: 1, weak: 2 };

function JournalSearchPanel({
  summaryResult,
  summaryStatus,
  positioningResult,
  journals,
  searchStatus,
  journalSearchPrompt,
  projectInfo,
  onGetJournalSearchPrompt,
  onParseExternalResults,
  onNavigateToPositioning,
}: JournalSearchPanelProps) {
  const [selectedJournal, setSelectedJournal] = useState<JournalCandidate | null>(null);
  const [sortBy, setSortBy] = useState<SortKey>("match_score");
  const [filterCost, setFilterCost] = useState<CostFilter>("all");
  const [searchTab, setSearchTab] = useState<"api" | "paste">("paste");
  const [copied, setCopied] = useState(false);
  const [externalA, setExternalA] = useState("");
  const [savedContent, setSavedContent] = useState("");
  const [saving, setSaving] = useState(false);

  const stSearch = statusLabels[searchStatus];
  const ready = summaryResult && summaryStatus === "done";
  const isSaved = savedContent === externalA && externalA.length > 0;
  const hasUnsavedChanges = externalA.length > 0 && !isSaved;

  const sortedJournals = useMemo(() => {
    let filtered = [...journals];
    if (filterCost === "low") filtered = filtered.filter(j => j.cost_risk_level === "low");
    else if (filterCost === "medium") filtered = filtered.filter(j => j.cost_risk_level === "low" || j.cost_risk_level === "medium");
    else if (filterCost === "no_apc") filtered = filtered.filter(j => j.apc_required === "no_apc");

    if (sortBy === "match_score") filtered.sort((a, b) => b.match_score - a.match_score);
    else if (sortBy === "cost_risk") filtered.sort((a, b) => (costRiskOrder[a.cost_risk_level] ?? 3) - (costRiskOrder[b.cost_risk_level] ?? 3));
    else if (sortBy === "recommendation") filtered.sort((a, b) => (recOrder[a.recommendation_level.toLowerCase()] ?? 3) - (recOrder[b.recommendation_level.toLowerCase()] ?? 3));
    return filtered;
  }, [journals, sortBy, filterCost]);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(journalSearchPrompt);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleSave = async () => {
    if (!projectInfo || !externalA.trim()) return;
    setSaving(true);
    try {
      await invoke("save_project_file", {
        projectDir: projectInfo.path,
        filename: "data/journal_research.md",
        content: externalA,
      });
      setSavedContent(externalA);
    } catch (_) {}
    setSaving(false);
  };

  const handleParse = async () => {
    if (!externalA.trim()) return;
    if (hasUnsavedChanges) {
      if (!confirm("未保存の変更があります。保存してから解析しますか？")) return;
      await handleSave();
    }
    onParseExternalResults(externalA, "");
  };

  // Load saved content on mount
  useEffect(() => {
    if (!projectInfo) return;
    invoke<string>("load_project_file", { projectDir: projectInfo.path, filename: "data/journal_research.md" })
      .then(content => {
        if (content) {
          setSavedContent(content);
          if (!externalA) setExternalA(content);
        }
      })
      .catch(() => {});
  }, [projectInfo]);

  return (
    <div className="panel">
      <div className="input-section info-card">
        <h3>ジャーナル調査について</h3>
        <p>
          論文要約と立ち位置調査の結果をもとに、投稿先ジャーナル候補を探します。
          特に APC を避けられる候補を重視します。
        </p>
      </div>

      {!ready && (
        <div className="input-section">
          <p className="hint-text">先に「論文要約」画面で要約を生成してください。</p>
        </div>
      )}

      {ready && (
        <>
          <div className="input-section">
            <h3>立ち位置調査の状態</h3>
            {positioningResult ? (
              <p className="hint-text" style={{ color: "#107c10" }}>
                ✓ 立ち位置調査結果を使用してジャーナル候補を探します。先行研究・新規性・研究領域を踏まえた候補が得られます。
              </p>
            ) : (
              <p className="hint-text" style={{ color: "#c19c00" }}>
                ⚠ 立ち位置調査結果がありません。論文要約のみでジャーナル調査を行います。候補の精度が下がる可能性があります。{" "}
                <button className="link-button" onClick={onNavigateToPositioning}>
                  立ち位置調査へ戻る
                </button>
              </p>
            )}
          </div>

          <div className="input-section">
            <h3>調査方式</h3>
            <div className="report-tabs">
              <button className={`report-tab ${searchTab === "paste" ? "active" : ""}`} onClick={() => setSearchTab("paste")}>外部 Deep Research を利用（推奨）</button>
              <button className={`report-tab ${searchTab === "api" ? "active" : ""}`} onClick={() => setSearchTab("api")}>API で実行（上級者向け）</button>
            </div>

            {searchTab === "api" && (
              <DeepResearchApiRunner
                summaryResult={summaryResult!}
                positioningResult={positioningResult}
                taskType="journal_search"
                onResult={(report) => setExternalA(report)}
              />
            )}

            {searchTab === "paste" && (
              <div style={{ paddingTop: 12 }}>
                <h4>プロンプト生成</h4>
                <p className="hint-text">
                  推奨: 外部 Deep Research を利用します。API 課金を抑えながら、ChatGPT / Perplexity / Gemini / Claude 等の Deep Research 結果を貼り付けて解析します。
                </p>
                <div className="action-row">
                  <button onClick={() => {
                    if (!positioningResult) {
                      if (!confirm("立ち位置調査結果がありません。論文要約のみでジャーナル調査を実行しますか？\nより精度を上げるには、先に立ち位置調査を行うことを推奨します。")) return;
                    }
                    onGetJournalSearchPrompt();
                  }}>
                    {journalSearchPrompt ? "プロンプトを再生成" : "プロンプトを生成"}
                  </button>
                  {journalSearchPrompt && (
                    <button onClick={handleCopy}>
                      {copied ? "コピー済み ✓" : "プロンプトをコピー"}
                    </button>
                  )}
                </div>
                {journalSearchPrompt && (
                  <details className="raw-text-details" style={{ marginTop: 8 }}>
                    <summary>プロンプト内容を表示</summary>
                    <pre className="prompt-preview">{journalSearchPrompt}</pre>
                  </details>
                )}
              </div>
            )}
          </div>

          {/* Result textarea */}
          <div className="input-section">
            <h3>ジャーナル調査結果</h3>
            <p className="hint-text">
              {searchTab === "paste"
                ? "外部 AI の Deep Research 結果を以下のテキストエリアに貼り付けてください。"
                : "API 実行結果が自動反映されます。確認・編集してください。"}
            </p>
            <textarea
              className="result-textarea"
              value={externalA}
              onChange={e => setExternalA(e.target.value)}
              placeholder="ジャーナル候補の検索結果をここに貼り付けてください..."
              rows={12}
            />
            <div className="action-row" style={{ marginTop: 8 }}>
              <button onClick={handleSave} disabled={!externalA.trim() || saving || isSaved}>
                {saving ? "保存中..." : isSaved ? "保存済み ✓" : "ジャーナル調査結果を保存"}
              </button>
              {hasUnsavedChanges && (
                <span className="status-chip warn">未保存の変更あり</span>
              )}
              {isSaved && (
                <span className="status-chip ok">保存済み</span>
              )}
            </div>
          </div>

          {/* Parse section */}
          {isSaved && (
            <div className="input-section">
              <h3>解析</h3>
              <p className="hint-text">
                保存済みのジャーナル調査結果を解析し、候補ジャーナルを生成します。
              </p>
              <div className="action-row">
                <button
                  onClick={handleParse}
                  disabled={searchStatus === "in_progress"}
                >
                  {searchStatus === "in_progress" ? "解析中..." : "保存済み結果を解析して候補を生成"}
                </button>
                <span className={`status-chip ${stSearch.cls}`}>{stSearch.label}</span>
              </div>
            </div>
          )}
        </>
      )}

      {/* Journal results table */}
      {journals.length > 0 && (
        <div className="input-section">
          <h3>推薦ジャーナル候補 ({journals.length} 件)</h3>
          {journals[0]?.journal_name.startsWith("(JSON parse failed") ? (
            <div className="raw-fallback">
              <p className="hint-text" style={{ color: "#d13438" }}>JSON パース失敗。生レスポンスを表示します。</p>
              <pre className="raw-response">{journals[0].raw_response}</pre>
            </div>
          ) : (
            <>
              <div className="journal-filters">
                <label>ソート: </label>
                <select value={sortBy} onChange={e => setSortBy(e.target.value as SortKey)}>
                  <option value="match_score">マッチ度</option>
                  <option value="cost_risk">費用リスク（低→高）</option>
                  <option value="recommendation">推薦レベル</option>
                </select>
                <label style={{ marginLeft: 12 }}>フィルタ: </label>
                <select value={filterCost} onChange={e => setFilterCost(e.target.value as CostFilter)}>
                  <option value="all">すべて</option>
                  <option value="low">低コストのみ</option>
                  <option value="medium">中コストまで</option>
                  <option value="no_apc">APC なしのみ</option>
                </select>
              </div>
              <div className="journal-table-wrapper">
                <table className="journal-table">
                  <thead>
                    <tr>
                      <th>#</th><th>ジャーナル名</th><th>マッチ度</th><th>IF/指標</th><th>APC</th><th>掲載方法</th><th>費用リスク</th><th>推薦</th>
                    </tr>
                  </thead>
                  <tbody>
                    {sortedJournals.map((j, i) => (
                      <tr key={j.journal_name} className={selectedJournal === j ? "selected" : ""} onClick={() => setSelectedJournal(selectedJournal === j ? null : j)}>
                        <td>{i + 1}</td>
                        <td><strong>{j.journal_name}</strong></td>
                        <td>{j.match_score}%</td>
                        <td>{j.impact_factor_or_metric}</td>
                        <td>{j.apc}</td>
                        <td>{j.publication_route}</td>
                        <td><span className={`cost-${translateCostRiskClass(j.cost_risk_level)}`}>{translateCostRisk(j.cost_risk_level)}</span></td>
                        <td><span className={`rec-${j.recommendation_level.toLowerCase()}`}>{translateRecommendationLevel(j.recommendation_level)}</span></td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              {selectedJournal && (
                <div className="journal-detail">
                  <h4>{selectedJournal.journal_name} (マッチ度: {selectedJournal.match_score}%)</h4>
                  <div className="detail-grid">
                    <div className="detail-item"><label>出版社</label><p>{selectedJournal.publisher}</p></div>
                    <div className="detail-item"><label>スコープ適合</label><p>{selectedJournal.scope_fit}</p></div>
                    <div className="detail-item"><label>記事タイプ適合</label><p>{selectedJournal.article_type_fit}</p></div>
                    <div className="detail-item"><label>類似論文</label><p>{selectedJournal.similar_articles}</p></div>
                    <div className="detail-item"><label>IF/指標</label><p>{selectedJournal.impact_factor_or_metric}</p></div>
                    <div className="detail-item"><label>掲載方法</label><p>{selectedJournal.publication_route}</p></div>
                    <div className="detail-item"><label>APC</label><p>{selectedJournal.apc}</p></div>
                    <div className="detail-item"><label>APC 必須/任意</label><p>{translateApcRequired(selectedJournal.apc_required)}</p></div>
                    <div className="detail-item"><label>APC 回避の可能性</label><p>{selectedJournal.apc_avoidance}</p></div>
                    <div className="detail-item"><label>waiver / 割引情報</label><p>{selectedJournal.waiver_or_discount_info}</p></div>
                    <div className="detail-item"><label>費用リスク</label><p><span className={`cost-${translateCostRiskClass(selectedJournal.cost_risk_level)}`}>{translateCostRisk(selectedJournal.cost_risk_level)}</span></p></div>
                    <div className="detail-item"><label>推奨投稿方法</label><p>{selectedJournal.recommended_submission_strategy}</p></div>
                    <div className="detail-item"><label>文字数制限</label><p>{selectedJournal.word_limit}</p></div>
                    <div className="detail-item"><label>OA ポリシー</label><p>{selectedJournal.open_access_policy}</p></div>
                    <div className="detail-item"><label>メリット</label><p>{selectedJournal.pros}</p></div>
                    <div className="detail-item"><label>デメリット</label><p>{selectedJournal.cons}</p></div>
                    <div className="detail-item"><label>推薦レベル</label><p><span className={`rec-${selectedJournal.recommendation_level.toLowerCase()}`}>{translateRecommendationLevel(selectedJournal.recommendation_level)}</span></p></div>
                    <div className="detail-item"><label>推薦理由</label><p>{selectedJournal.reason}</p></div>
                    <div className="detail-item"><label>根拠</label><p>{selectedJournal.source_evidence}</p></div>
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      )}
    </div>
  );
}

export default JournalSearchPanel;
