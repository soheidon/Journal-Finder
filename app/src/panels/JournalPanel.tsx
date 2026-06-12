import { useState, useMemo } from "react";
import type { SummaryResult, JournalCandidate, PipelineStatus } from "../App";
import PasteModal from "../components/PasteModal";

type SortKey = "match_score" | "cost_risk" | "recommendation";
type CostFilter = "all" | "low" | "medium" | "no_apc";

interface JournalPanelProps {
  summaryResult: SummaryResult | null;
  journals: JournalCandidate[];
  summaryStatus: PipelineStatus;
  searchStatus: PipelineStatus;
  searchPrompt: string;
  onGenerateSummary: () => void;
  onGetSearchPrompt: () => void;
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
  searchPrompt,
  onGenerateSummary,
  onGetSearchPrompt,
  onParseExternalResults,
}: JournalPanelProps) {
  const [modalOpen, setModalOpen] = useState(false);
  const [selectedJournal, setSelectedJournal] = useState<JournalCandidate | null>(null);
  const [sortBy, setSortBy] = useState<SortKey>("match_score");
  const [filterCost, setFilterCost] = useState<CostFilter>("all");

  const stSummary = statusLabels[summaryStatus];
  const stSearch = statusLabels[searchStatus];

  const sortedJournals = useMemo(() => {
    let filtered = [...journals];
    if (filterCost === "low") {
      filtered = filtered.filter(j => j.cost_risk_level === "low");
    } else if (filterCost === "medium") {
      filtered = filtered.filter(j => j.cost_risk_level === "low" || j.cost_risk_level === "medium");
    } else if (filterCost === "no_apc") {
      filtered = filtered.filter(j => j.apc_required === "no_apc");
    }

    if (sortBy === "match_score") {
      filtered.sort((a, b) => b.match_score - a.match_score);
    } else if (sortBy === "cost_risk") {
      filtered.sort((a, b) => (costRiskOrder[a.cost_risk_level] ?? 3) - (costRiskOrder[b.cost_risk_level] ?? 3));
    } else if (sortBy === "recommendation") {
      filtered.sort((a, b) => (recOrder[a.recommendation_level.toLowerCase()] ?? 3) - (recOrder[b.recommendation_level.toLowerCase()] ?? 3));
    }
    return filtered;
  }, [journals, sortBy, filterCost]);

  const handleOpenModal = () => {
    if (!searchPrompt) {
      onGetSearchPrompt();
    }
    setModalOpen(true);
  };

  const handleParse = (externalA: string, externalB: string) => {
    setModalOpen(false);
    onParseExternalResults(externalA, externalB);
  };

  return (
    <div className="panel journal-panel">
      <h2>ジャーナル検索</h2>

      <div className="input-section">
        <h3>Step 1: 論文要約</h3>
        <div className="action-row">
          <button
            onClick={onGenerateSummary}
            disabled={!summaryResult === false && summaryStatus === "in_progress"}
          >
            {summaryStatus === "in_progress" ? "要約生成中..." : "論文要約を生成"}
          </button>
          <span className={`status-chip ${stSummary.cls}`}>{stSummary.label}</span>
        </div>
        {summaryStatus === "done" && summaryResult && (
          <p className="hint-text" style={{ color: "#81c784", marginTop: 8 }}>
            ✓ 要約完了: {summaryResult.research_topic}
          </p>
        )}
      </div>

      {summaryResult && summaryStatus === "done" && (
        <div className="input-section">
          <h3>要約結果</h3>
          {summaryResult.research_topic.startsWith("(JSON parse failed") ? (
            <div className="raw-fallback">
              <p className="hint-text" style={{ color: "#e57373" }}>
                JSON のパースに失敗しました。LLM の生レスポンスを表示します。
              </p>
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
          <details className="raw-text-details">
            <summary>LLM 生レスポンス</summary>
            <pre>{summaryResult.raw_response}</pre>
          </details>
        </div>
      )}

      {summaryResult && summaryStatus === "done" && (
        <div className="input-section">
          <h3>Step 2: Deep Research（貼り付け方式）</h3>
          <div className="search-mode-note">
            <p>外部 AI（ChatGPT / Perplexity / Gemini / Claude 等）の Deep Research 結果を貼り付けて、ジャーナル候補を解析します。</p>
          </div>
          <div className="action-row">
            <button onClick={handleOpenModal}>
              Deep Research 結果を貼り付け
            </button>
            <span className={`status-chip ${stSearch.cls}`}>{stSearch.label}</span>
          </div>
        </div>
      )}

      {journals.length > 0 && (
        <div className="input-section">
          <h3>推薦ジャーナル候補 ({journals.length} 件)</h3>
          {journals[0]?.journal_name.startsWith("(JSON parse failed") ? (
            <div className="raw-fallback">
              <p className="hint-text" style={{ color: "#e57373" }}>
                JSON のパースに失敗しました。LLM の生レスポンスを表示します。
              </p>
              <pre className="raw-response">{journals[0].raw_response}</pre>
            </div>
          ) : (
            <>
              <div className="journal-filters">
                <label>ソート: </label>
                <select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortKey)}>
                  <option value="match_score">マッチ度</option>
                  <option value="cost_risk">費用リスク（低→高）</option>
                  <option value="recommendation">推薦レベル</option>
                </select>
                <label style={{ marginLeft: 12 }}>フィルタ: </label>
                <select value={filterCost} onChange={(e) => setFilterCost(e.target.value as CostFilter)}>
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
                      <th>#</th>
                      <th>ジャーナル名</th>
                      <th>マッチ度</th>
                      <th>IF/指標</th>
                      <th>APC</th>
                      <th>掲載方法</th>
                      <th>費用リスク</th>
                      <th>推薦</th>
                    </tr>
                  </thead>
                  <tbody>
                    {sortedJournals.map((j, i) => (
                      <tr
                        key={j.journal_name}
                        className={selectedJournal === j ? "selected" : ""}
                        onClick={() => setSelectedJournal(selectedJournal === j ? null : j)}
                      >
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

      <PasteModal
        isOpen={modalOpen}
        searchPrompt={searchPrompt}
        onClose={() => setModalOpen(false)}
        onParse={handleParse}
        isParsing={searchStatus === "in_progress"}
      />
    </div>
  );
}

export default JournalPanel;
