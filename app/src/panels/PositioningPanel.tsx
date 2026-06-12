import { useState } from "react";
import type { SummaryResult, PipelineStatus } from "../App";
import DeepResearchApiRunner from "../components/DeepResearchApiRunner";

interface PositioningPanelProps {
  summaryResult: SummaryResult | null;
  summaryStatus: PipelineStatus;
  positioningPrompt: string;
  positioningResult: string;
  onGetPositioningPrompt: () => void;
  onSetPositioningResult: (result: string) => void;
  onNavigateToJournalSearch: () => void;
}

function PositioningPanel({
  summaryResult,
  summaryStatus,
  positioningPrompt,
  positioningResult,
  onGetPositioningPrompt,
  onSetPositioningResult,
  onNavigateToJournalSearch,
}: PositioningPanelProps) {
  const [copied, setCopied] = useState(false);
  const [searchTab, setSearchTab] = useState<"paste" | "api">("paste");

  const handleCopy = async () => {
    await navigator.clipboard.writeText(positioningPrompt);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const ready = summaryResult && summaryStatus === "done";

  return (
    <div className="panel">
      <div className="input-section info-card">
        <h3>立ち位置調査について</h3>
        <p>
          論文の要約をもとに、先行研究の中でこの論文がどういう位置にあるかを調べます。
          立ち位置調査を先に行うと、類似研究・研究領域・新規性・方法論上の特徴が整理されます。
          この結果を使ってジャーナル調査を行うことで、単なるキーワード検索ではなく、どの領域のどの読者に向けた論文かを踏まえた投稿先候補を探せます。
        </p>
        <ul>
          <li>近い先行研究を調べる</li>
          <li>新規性・方法論の特徴を整理する</li>
          <li>どの研究領域に属するかを整理する</li>
          <li>投稿先ジャーナル選定に使う検索キーワードを提案する</li>
        </ul>
      </div>

      {!ready && (
        <div className="input-section">
          <p className="hint-text">先に「論文要約」画面で要約を生成してください。</p>
        </div>
      )}

      {ready && (
        <>
          <div className="input-section">
            <h3>調査方式</h3>
            <div className="report-tabs">
              <button className={`report-tab ${searchTab === "paste" ? "active" : ""}`} onClick={() => setSearchTab("paste")}>外部 Deep Research を利用（推奨）</button>
              <button className={`report-tab ${searchTab === "api" ? "active" : ""}`} onClick={() => setSearchTab("api")}>API で実行（上級者向け）</button>
            </div>

            {searchTab === "paste" && (
              <div style={{ paddingTop: 12 }}>
                <h4>プロンプト生成</h4>
                <p className="hint-text">
                  推奨: 外部 Deep Research を利用します。API 課金を抑えながら、ChatGPT / Perplexity / Gemini / Claude 等の Deep Research 結果を貼り付けて解析します。
                </p>
                <div className="action-row">
                  <button onClick={onGetPositioningPrompt}>
                    {positioningPrompt ? "プロンプトを再生成" : "プロンプトを生成"}
                  </button>
                  {positioningPrompt && (
                    <button onClick={handleCopy}>
                      {copied ? "コピー済み ✓" : "プロンプトをコピー"}
                    </button>
                  )}
                </div>
                {positioningPrompt && (
                  <details className="raw-text-details" style={{ marginTop: 8 }}>
                    <summary>プロンプト内容を表示</summary>
                    <pre className="prompt-preview">{positioningPrompt}</pre>
                  </details>
                )}
              </div>
            )}

            {searchTab === "api" && (
              <DeepResearchApiRunner
                summaryResult={summaryResult}
                taskType="positioning"
                onResult={(report) => onSetPositioningResult(report)}
              />
            )}
          </div>

          {/* Result textarea — shared between both modes */}
          <div className="input-section">
            <h3>立ち位置調査結果</h3>
            <p className="hint-text">
              {searchTab === "paste"
                ? "外部 AI の Deep Research 結果を以下のテキストエリアに貼り付けてください。"
                : "API 実行結果が自動反映されます。確認・編集してください。"}
            </p>
            <textarea
              className="result-textarea"
              value={positioningResult}
              onChange={e => onSetPositioningResult(e.target.value)}
              placeholder="立ち位置調査結果がここに表示されます..."
              rows={12}
            />
            {positioningResult && (
              <>
                <p className="hint-text" style={{ color: "#107c10", marginTop: 8 }}>
                  ✓ 立ち位置調査結果が保存されました。次の「ジャーナル調査」画面に進んでください。
                </p>
                <div className="action-row" style={{ marginTop: 8 }}>
                  <button onClick={onNavigateToJournalSearch}>
                    次へ：ジャーナル調査へ進む →
                  </button>
                </div>
              </>
            )}
          </div>
        </>
      )}
    </div>
  );
}

export default PositioningPanel;
