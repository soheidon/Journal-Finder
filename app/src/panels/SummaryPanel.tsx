import type { SummaryResult, PipelineStatus } from "../App";

interface SummaryPanelProps {
  summaryResult: SummaryResult | null;
  summaryStatus: PipelineStatus;
  onGenerateSummary: () => void;
}

const statusLabels: Record<PipelineStatus, { label: string; cls: string }> = {
  not_started: { label: "未実行", cls: "unrun" },
  in_progress: { label: "実行中...", cls: "running" },
  done: { label: "完了", cls: "ok" },
  failed: { label: "エラー", cls: "err" },
};

function SummaryPanel({ summaryResult, summaryStatus, onGenerateSummary }: SummaryPanelProps) {
  const st = statusLabels[summaryStatus];

  return (
    <div className="panel">
      <div className="input-section info-card">
        <h3>論文要約について</h3>
        <p>
          抽出済みの論文テキストを LLM に読み込み、研究テーマ・目的・方法・結果などを構造化要約として生成します。
          この要約は後の立ち位置調査・ジャーナル調査で使用されます。
        </p>
      </div>

      <div className="input-section">
        <h3>論文要約を生成</h3>
        <div className="action-row">
          <button onClick={onGenerateSummary} disabled={summaryStatus === "in_progress"}>
            {summaryStatus === "in_progress" ? "要約生成中..." : "論文要約を生成"}
          </button>
          <span className={`status-chip ${st.cls}`}>{st.label}</span>
        </div>
        {summaryStatus === "done" && summaryResult && (
          <p className="hint-text" style={{ color: "#107c10", marginTop: 8 }}>
            ✓ 要約完了: {summaryResult.research_topic}
          </p>
        )}
        {summaryStatus === "not_started" && (
          <p className="hint-text">先に「入力」画面でテキスト抽出を実行してください。</p>
        )}
      </div>

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
              <div className="summary-item"><label>測定指標</label><p>{summaryResult.measures}</p></div>
              <div className="summary-item"><label>統計手法</label><p>{summaryResult.statistics}</p></div>
              <div className="summary-item"><label>主要結果</label><p>{summaryResult.findings}</p></div>
              <div className="summary-item"><label>著者の主張する貢献</label><p>{summaryResult.claimed_contributions}</p></div>
              <div className="summary-item"><label>キーワード</label><p>{summaryResult.keywords_for_search.join(", ")}</p></div>
            </div>
          )}
          <details className="raw-text-details">
            <summary>LLM 生レスポンス</summary>
            <pre>{summaryResult.raw_response}</pre>
          </details>
        </div>
      )}
    </div>
  );
}

export default SummaryPanel;
