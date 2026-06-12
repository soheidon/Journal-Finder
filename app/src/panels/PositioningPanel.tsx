import { useState } from "react";
import type { SummaryResult, PipelineStatus } from "../App";

interface PositioningPanelProps {
  summaryResult: SummaryResult | null;
  summaryStatus: PipelineStatus;
  positioningPrompt: string;
  positioningResult: string;
  onGetPositioningPrompt: () => void;
  onSetPositioningResult: (result: string) => void;
}

function PositioningPanel({
  summaryResult,
  summaryStatus,
  positioningPrompt,
  positioningResult,
  onGetPositioningPrompt,
  onSetPositioningResult,
}: PositioningPanelProps) {
  const [copied, setCopied] = useState(false);

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
          外部 AI（ChatGPT / Perplexity / Gemini / Claude 等）の Deep Research を使って調査し、結果を貼り戻してください。
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
            <h3>Step 1: プロンプト生成</h3>
            <p className="hint-text">
              以下のプロンプトをコピーして、外部 AI の Deep Research に貼り付けてください。
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

          {positioningPrompt && (
            <div className="input-section">
              <h3>Step 2: 結果を貼り戻し</h3>
              <p className="hint-text">
                外部 AI の Deep Research 結果を以下のテキストエリアに貼り付けてください。
              </p>
              <textarea
                className="result-textarea"
                value={positioningResult}
                onChange={e => onSetPositioningResult(e.target.value)}
                placeholder="ChatGPT / Perplexity / Gemini / Claude 等の Deep Research 結果をここに貼り付けてください..."
                rows={12}
              />
              {positioningResult && (
                <p className="hint-text" style={{ color: "#107c10", marginTop: 8 }}>
                  ✓ 立ち位置調査結果が貼り付けられました。次の「ジャーナル調査」画面に進んでください。
                </p>
              )}
            </div>
          )}
        </>
      )}
    </div>
  );
}

export default PositioningPanel;
