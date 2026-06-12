import { useState, useMemo } from "react";
import type { SummaryResult, JournalCandidate, PipelineStatus } from "../App";

type SortKey = "match_score" | "cost_risk" | "recommendation";
type CostFilter = "all" | "low" | "medium" | "no_apc";

interface JournalPanelProps {
  summaryResult: SummaryResult | null;
  journals: JournalCandidate[];
  summaryStatus: PipelineStatus;
  searchStatus: PipelineStatus;
  positioningPrompt: string;
  positioningResult: string;
  journalSearchPrompt: string;
  onGenerateSummary: () => void;
  onGetPositioningPrompt: () => void;
  onSetPositioningResult: (result: string) => void;
  onGetJournalSearchPrompt: () => void;
  onParseExternalResults: (externalA: string, externalB: string) => void;
}

const statusLabels: Record<PipelineStatus, { label: string; cls: string }> = {
  not_started: { label: "未実行", cls: "unrun" },
  in_progress: { label: "実行中...", cls: "running" },
  done: { label: "完了", cls: "ok" },
  failed: { label: "エラー", cls: "err" },
};

const costRiskOrder: Record<string, number> = { low: 0, medium: 1, high: 2, unknown: 3 };
const recOrder: Record<string, number> = { strong: 0, moderate: 1, weak: 2 };

function JournalPanel({
  summaryResult,
  journals,
  summaryStatus,
  searchStatus,
  positioningPrompt,
  positioningResult,
  journalSearchPrompt,
  onGenerateSummary,
  onGetPositioningPrompt,
  onSetPositioningResult,
  onGetJournalSearchPrompt,
  onParseExternalResults,
}: JournalPanelProps) {
  const [selectedJournal, setSelectedJournal] = useState<JournalCandidate | null>(null);
  const [sortBy, setSortBy] = useState<SortKey>("match_score");
  const [filterCost, setFilterCost] = useState<CostFilter>("all");
  const [searchTab, setSearchTab] = useState<"api" | "paste">("paste");
  const [positioningCopied, setPositioningCopied] = useState(false);
  const [journalCopied, setJournalCopied] = useState(false);
  const [externalJournalA, setExternalJournalA] = useState("");
  const [externalJournalB, setExternalJournalB] = useState("");

  const stSummary = statusLabels[summaryStatus];
  const stSearch = statusLabels[searchStatus];

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

  const copyToClipboard = async (text: string, setCopied: (v: boolean) => void) => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="panel journal-panel">
      <div className="input-section info-card">
        <h3>検索について</h3>
        <p>論文の構造化要約をもとに、投稿先ジャーナル候補を検索します。</p>
        <ul>
          <li><strong>API で Deep Research</strong>: API 経由で自動調査（準備中）</li>
          <li><strong>外部 Deep Research を利用</strong>: 2段階で ChatGPT / Perplexity 等の結果を貼り付け</li>
        </ul>
      </div>

      {/* Summary */}
      <div className="input-section">
        <h3>Step 1: 論文要約</h3>
        <div className="action-row">
          <button onClick={onGenerateSummary} disabled={summaryStatus === "in_progress"}>
            {summaryStatus === "in_progress" ? "要約生成中..." : "論文要約を生成"}
          </button>
          <span className={`status-chip ${stSummary.cls}`}>{stSummary.label}</span>
        </div>
        {summaryStatus === "done" && summaryResult && (
          <p className="hint-text" style={{ color: "#107c10", marginTop: 8 }}>
            ✓ 要約完了: {summaryResult.research_topic}
          </p>
        )}
      </div>

      {/* Summary details */}
      {summaryResult && summaryStatus === "done" && (
        <div className="input-section">
          <h3>要約結果</h3>
          {summaryResult.research_topic.startsWith("(JSON parse failed") ? (
            <div className="raw-fallback">
              <p className="hint-text" style={{ color: "#d13438" }}>JSON パース失敗。生レスポンスを表示します。</p>
              <pre className="raw-response">{summaryResult.raw_response}</pre>
            </div>
          ) : (
            <div className="summary-grid">
              <div className="summary-item"><label>研究テーマ</label><p>{summaryResult.research_topic}</p></div>
              <div className="summary-item"><label>目的</label><p>{summaryResult.objective}</p></div>
              <div className="summary-item"><label>対象</label><p>{summaryResult.sample_summary}</p></div>
              <div className="summary-item"><label>研究デザイン</label><p>{summaryResult.design}</p></div>
              <div className="summary-item"><label>方法</label><p>{summaryResult.methods_summary}</p></div>
              <div className="summary-item"><label>主要結果</label><p>{summaryResult.findings}</p></div>
              <div className="summary-item"><label>キーワード</label><p>{summaryResult.keywords_for_search.join(", ")}</p></div>
            </div>
          )}
        </div>
      )}

      {/* Deep Research tabs */}
      {summaryResult && summaryStatus === "done" && (
        <div className="input-section">
          <h3>Step 2: Deep Research</h3>
          <div className="report-tabs">
            <button className={`report-tab ${searchTab === "api" ? "active" : ""}`} onClick={() => setSearchTab("api")}>API で実行</button>
            <button className={`report-tab ${searchTab === "paste" ? "active" : ""}`} onClick={() => setSearchTab("paste")}>外部 Deep Research を利用</button>
          </div>

          {searchTab === "api" && (
            <div style={{ padding: "12px 0" }}>
              <p className="hint-text">API Deep Research は今後実装予定です。現在は「外部 Deep Research を利用」タブをご使用ください。</p>
              <div className="form-grid" style={{ maxWidth: 500, marginTop: 12, opacity: 0.5 }}>
                <label>プロバイダー</label>
                <select disabled><option>Perplexity Sonar Deep Research</option></select>
              </div>
              <button disabled style={{ marginTop: 12 }}>Deep Research を実行（準備中）</button>
            </div>
          )}

          {searchTab === "paste" && (
            <div>
              {/* Step 2A: Positioning */}
              <div className="search-step">
                <h4>Step 2A: 先行研究上の立ち位置を調査</h4>
                <p className="hint-text">この論文が先行研究の中でどういう位置にあるかを調べます。外部 AI の Deep Research に貼り付けてください。</p>

                <div className="action-row" style={{ marginTop: 8 }}>
                  <button onClick={onGetPositioningPrompt}>
                    {positioningPrompt ? "プロンプトを再生成" : "プロンプトを生成"}
                  </button>
                  {positioningPrompt && (
                    <button onClick={() => copyToClipboard(positioningPrompt, setPositioningCopied)}>
                      {positioningCopied ? "コピー済み ✓" : "プロンプトをコピー"}
                    </button>
                  )}
                </div>

                {positioningPrompt && (
                  <details className="raw-text-details" style={{ marginTop: 8 }}>
                    <summary>プロンプト内容を表示</summary>
                    <pre className="prompt-preview">{positioningPrompt}</pre>
                  </details>
                )}

                {positioningPrompt && (
                  <>
                    <label style={{ marginTop: 12, display: "block", fontSize: 13, color: "#555" }}>
                      外部 AI の Deep Research 結果を貼り付けてください:
                    </label>
                    <textarea
                      className="result-textarea"
                      value={positioningResult}
                      onChange={e => onSetPositioningResult(e.target.value)}
                      placeholder="ChatGPT / Perplexity / Gemini / Claude 等の Deep Research 結果をここに貼り付けてください..."
                      rows={8}
                    />
                    {positioningResult && (
                      <p className="hint-text" style={{ color: "#107c10" }}>✓ 立ち位置調査結果が貼り付けられました</p>
                    )}
                  </>
                )}
              </div>

              {/* Step 2B: Journal search */}
              {positioningResult && (
                <div className="search-step" style={{ marginTop: 20 }}>
                  <h4>Step 2B: 投稿先ジャーナルを調査</h4>
                  <p className="hint-text">Step 2A の結果を踏まえて、投稿先候補を探します。特に APC を避けられる候補を重視します。</p>

                  <div className="action-row" style={{ marginTop: 8 }}>
                    <button onClick={onGetJournalSearchPrompt}>
                      {journalSearchPrompt ? "プロンプトを再生成" : "プロンプトを生成"}
                    </button>
                    {journalSearchPrompt && (
                      <button onClick={() => copyToClipboard(journalSearchPrompt, setJournalCopied)}>
                        {journalCopied ? "コピー済み ✓" : "プロンプトをコピー"}
                      </button>
                    )}
                  </div>

                  {journalSearchPrompt && (
                    <details className="raw-text-details" style={{ marginTop: 8 }}>
                      <summary>プロンプト内容を表示</summary>
                      <pre className="prompt-preview">{journalSearchPrompt}</pre>
                    </details>
                  )}

                  {journalSearchPrompt && (
                    <>
                      <label style={{ marginTop: 12, display: "block", fontSize: 13, color: "#555" }}>
                        ジャーナル調査の Deep Research 結果を貼り付けてください:
                      </label>
                      <textarea
                        className="result-textarea"
                        value={externalJournalA}
                        onChange={e => setExternalJournalA(e.target.value)}
                        placeholder="ジャーナル候補の検索結果をここに貼り付けてください..."
                        rows={8}
                      />
                      <label style={{ marginTop: 8, display: "block", fontSize: 12, color: "#888" }}>
                        2つ目の AI 結果（任意）:
                      </label>
                      <textarea
                        className="result-textarea"
                        value={externalJournalB}
                        onChange={e => setExternalJournalB(e.target.value)}
                        placeholder="別の AI で同じプロンプトを実行した場合の結果（任意）..."
                        rows={5}
                      />
                      <div className="action-row" style={{ marginTop: 12 }}>
                        <button
                          onClick={() => onParseExternalResults(externalJournalA, externalJournalB)}
                          disabled={!externalJournalA.trim() || searchStatus === "in_progress"}
                        >
                          {searchStatus === "in_progress" ? "解析中..." : "貼り付け結果を解析"}
                        </button>
                        <span className={`status-chip ${stSearch.cls}`}>{stSearch.label}</span>
                      </div>
                    </>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
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
                        <td><span className={`cost-${j.cost_risk_level}`}>{j.cost_risk_level}</span></td>
                        <td><span className={`rec-${j.recommendation_level.toLowerCase()}`}>{j.recommendation_level}</span></td>
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
                    <div className="detail-item"><label>APC 必須/任意</label><p>{selectedJournal.apc_required}</p></div>
                    <div className="detail-item"><label>APC 回避の可能性</label><p>{selectedJournal.apc_avoidance}</p></div>
                    <div className="detail-item"><label>waiver / 割引情報</label><p>{selectedJournal.waiver_or_discount_info}</p></div>
                    <div className="detail-item"><label>費用リスク</label><p><span className={`cost-${selectedJournal.cost_risk_level}`}>{selectedJournal.cost_risk_level}</span></p></div>
                    <div className="detail-item"><label>推奨投稿方法</label><p>{selectedJournal.recommended_submission_strategy}</p></div>
                    <div className="detail-item"><label>文字数制限</label><p>{selectedJournal.word_limit}</p></div>
                    <div className="detail-item"><label>OA ポリシー</label><p>{selectedJournal.open_access_policy}</p></div>
                    <div className="detail-item"><label>メリット</label><p>{selectedJournal.pros}</p></div>
                    <div className="detail-item"><label>デメリット</label><p>{selectedJournal.cons}</p></div>
                    <div className="detail-item"><label>推薦レベル</label><p><span className={`rec-${selectedJournal.recommendation_level.toLowerCase()}`}>{selectedJournal.recommendation_level}</span></p></div>
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

export default JournalPanel;
