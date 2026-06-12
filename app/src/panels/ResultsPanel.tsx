import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SummaryResult, JournalCandidate, ReportData } from "../App";
import { translateRecommendationLevel, translateCostRisk } from "../utils/translate";

interface ResultsPanelProps {
  summaryResult: SummaryResult | null;
  journals: JournalCandidate[];
  addLog: (message: string) => void;
  onExport: (content: string, defaultFilename: string, format: string) => void;
}

function ResultsPanel({ summaryResult, journals, addLog, onExport }: ResultsPanelProps) {
  const [report, setReport] = useState<ReportData | null>(null);
  const [generating, setGenerating] = useState(false);
  const [activeTab, setActiveTab] = useState<"preview" | "raw">("preview");
  const [copied, setCopied] = useState(false);

  const hasResults = summaryResult && journals.length > 0;

  const handleGenerateReport = useCallback(async () => {
    if (!summaryResult || journals.length === 0) return;
    setGenerating(true);
    addLog("レポート生成開始");
    try {
      const result = await invoke<ReportData>("generate_report", {
        summary: summaryResult,
        journals,
      });
      setReport(result);
      addLog("レポート生成完了");
    } catch (e) {
      addLog(`レポート生成エラー: ${e}`);
    } finally {
      setGenerating(false);
    }
  }, [summaryResult, journals, addLog]);

  const handleCopyMarkdown = useCallback(async () => {
    if (!report) return;
    await navigator.clipboard.writeText(report.markdown);
    setCopied(true);
    addLog("Markdown をクリップボードにコピーしました");
    setTimeout(() => setCopied(false), 2000);
  }, [report, addLog]);

  const handleSaveMarkdown = useCallback(() => {
    if (!report) return;
    onExport(report.markdown, "journal_finder_report.md", "md");
  }, [report, onExport]);

  const handleSaveJson = useCallback(() => {
    if (!report) return;
    onExport(report.json, "journal_finder_report.json", "json");
  }, [report, onExport]);

  const handleSaveDocx = useCallback(async () => {
    if (!report || !report.docx_base64) return;
    try {
      const binary = atob(report.docx_base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      const blob = new Blob([bytes], { type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "journal_finder_report.docx";
      a.click();
      URL.revokeObjectURL(url);
      addLog("Word ファイルをダウンロードしました");
    } catch (e) {
      addLog(`Word 保存エラー: ${e}`);
    }
  }, [report, addLog]);

  return (
    <div className="panel results-panel">
      <div className="input-section info-card">
        <h3>結果について</h3>
        <p>
          論文要約とジャーナル候補をまとめた投稿先選定レポートを生成・エクスポートできます。
          Markdown 形式でコピーまたはファイル保存が可能です。
        </p>
      </div>

      {!hasResults ? (
        <p className="hint-text">まだ結果がありません。「検索」画面でジャーナル検索を実行してください。</p>
      ) : (
        <>
          <div className="input-section">
            <h3>レポート生成</h3>
            <div className="action-row">
              <button onClick={handleGenerateReport} disabled={generating}>
                {generating ? "生成中..." : "レポートを生成"}
              </button>
            </div>
            <p className="hint-text">論文要約とジャーナル候補をまとめた投稿先選定レポートを生成します。</p>
          </div>

          {report && (
            <>
              <div className="input-section">
                <h3>エクスポート</h3>
              <div className="action-row">
                <button onClick={handleCopyMarkdown}>
                  {copied ? "コピー済み ✓" : "Markdown をコピー"}
                </button>
                <button onClick={handleSaveMarkdown}>Markdown として保存</button>
                <button onClick={handleSaveDocx}>Word として保存</button>
                <button onClick={handleSaveJson}>JSON として保存</button>
              </div>
              </div>

              <div className="input-section">
                <h3>レポートプレビュー</h3>
                <div className="report-tabs">
                  <button
                    className={`report-tab ${activeTab === "preview" ? "active" : ""}`}
                    onClick={() => setActiveTab("preview")}
                  >
                    プレビュー
                  </button>
                  <button
                    className={`report-tab ${activeTab === "raw" ? "active" : ""}`}
                    onClick={() => setActiveTab("raw")}
                  >
                    Markdown ソース
                  </button>
                </div>

                {activeTab === "preview" ? (
                  <div className="report-preview">
                    <h4>論文要約</h4>
                    <dl>
                      <dt>研究テーマ</dt>
                      <dd>{summaryResult!.research_topic}</dd>
                      <dt>目的</dt>
                      <dd>{summaryResult!.objective}</dd>
                      <dt>対象</dt>
                      <dd>{summaryResult!.sample_summary}</dd>
                      <dt>研究デザイン</dt>
                      <dd>{summaryResult!.design}</dd>
                      <dt>主要結果</dt>
                      <dd>{summaryResult!.findings}</dd>
                      <dt>キーワード</dt>
                      <dd>{summaryResult!.keywords_for_search.join(", ")}</dd>
                    </dl>

                    <h4>推薦ジャーナル候補 ({journals.length} 件)</h4>
                    <table className="journal-table">
                      <thead>
                        <tr>
                            <th>#</th>
                            <th>ジャーナル名</th>
                            <th>マッチ度</th>
                            <th>指標</th>
                            <th>APC</th>
                            <th>推薦</th>
                            <th>理由</th>
                        </tr>
                      </thead>
                      <tbody>
                        {journals.map((j, i) => (
                          <tr key={i}>
                            <td>{i + 1}</td>
                            <td><strong>{j.journal_name}</strong></td>
                            <td>{j.publisher}</td>
                            <td>{j.impact_factor_or_metric !== "未取得" ? j.impact_factor_or_metric : (j.quartile_or_rank || "—")}</td>
                            <td>{j.apc}</td>
                            <td><span className={`rec-${j.recommendation_level.toLowerCase()}`}>{translateRecommendationLevel(j.recommendation_level)}</span></td>
                            <td>{j.reason.length > 60 ? j.reason.substring(0, 60) + "..." : j.reason}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>

                    <h4 style={{ marginTop: 16 }}>最終推奨</h4>
                    {(() => {
                      const strong = journals.filter(j => j.recommendation_level.toLowerCase() === "strong");
                      const moderate = journals.filter(j => j.recommendation_level.toLowerCase() === "moderate");
                      const weak = journals.filter(j => j.recommendation_level.toLowerCase() === "weak");
                      return (
                        <div className="recommendation-summary">
                          {strong.length > 0 && (
                            <div className="rec-group">
                              <strong>第一候補:</strong> {strong.map(j => j.journal_name).join(", ")}
                            </div>
                          )}
                          {moderate.length > 0 && (
                            <div className="rec-group">
                              <strong>第二候補:</strong> {moderate.map(j => j.journal_name).join(", ")}
                            </div>
                          )}
                          {weak.length > 0 && (
                            <div className="rec-group">
                              <strong>チャレンジ:</strong> {weak.map(j => j.journal_name).join(", ")}
                            </div>
                          )}
                        </div>
                      );
                    })()}
                  </div>
                ) : (
                  <pre className="report-raw">{report.markdown}</pre>
                )}
              </div>
            </>
          )}
        </>
      )}
    </div>
  );
}

export default ResultsPanel;
