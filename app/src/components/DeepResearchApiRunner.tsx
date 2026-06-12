import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SummaryResult } from "../App";

interface DeepResearchApiResult {
  ok: boolean;
  raw_report: string;
  model_used: string;
  output_tokens: number;
  message: string;
}

interface DeepResearchApiRunnerProps {
  summaryResult: SummaryResult;
  positioningResult?: string;
  taskType: "positioning" | "journal_search";
  onResult: (report: string) => void;
}

function DeepResearchApiRunner({ summaryResult, positioningResult, taskType, onResult }: DeepResearchApiRunnerProps) {
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<DeepResearchApiResult | null>(null);

  const handleRun = async () => {
    const label = taskType === "positioning" ? "立ち位置調査" : "ジャーナル調査";
    if (!confirm(`OpenAI Deep Research で${label}を実行しますか？\n通常の LLM 呼び出しより時間と費用がかかる可能性があります。数分〜数十分かかる場合があります。`)) return;

    setRunning(true);
    setResult(null);
    try {
      const res = await invoke<DeepResearchApiResult>("run_openai_deep_research", {
        summary: summaryResult,
        positioningReport: positioningResult || "",
        taskType,
      });
      setResult(res);
      if (res.ok) {
        onResult(res.raw_report);
      }
    } catch (e) {
      setResult({ ok: false, raw_report: "", model_used: "", output_tokens: 0, message: `${e}` });
    } finally {
      setRunning(false);
    }
  };

  return (
    <div style={{ padding: "12px 0" }}>
      <p className="hint-text">
        上級者向け: API 経由で Deep Research を実行します。検索回数、reasoning tokens、引用処理などにより高額になる可能性があります。
      </p>
      <p className="hint-text" style={{ color: "#c19c00", marginTop: 4 }}>
        ⚠ OpenAI Deep Research は Responses API を使用します（chat/completions では動作しません）。通常のチャット API より時間と費用がかかります。
      </p>
      <p className="hint-text" style={{ marginTop: 8 }}>
        対応モデル: o4-mini-deep-research, o3-deep-research（settings.json で deep_research_provider を OpenAI に設定してください）
      </p>
      <div className="action-row" style={{ marginTop: 12 }}>
        <button disabled={running} onClick={handleRun}>
          {running ? "Deep Research 実行中（数分かかる場合があります）..." : "OpenAI Deep Research を実行"}
        </button>
      </div>
      {result && (
        <div style={{ marginTop: 12 }}>
          <h4>Deep Research 結果</h4>
          {result.ok ? (
            <>
              <pre className="raw-response" style={{ maxHeight: 400 }}>{result.raw_report}</pre>
              <p className="hint-text" style={{ color: "#107c10", marginTop: 8 }}>
                ✓ 結果が貼り付け欄に自動反映されました。確認・編集後に「解析」ボタンで処理を続行できます。
              </p>
            </>
          ) : (
            <>
              <pre className="raw-response" style={{ maxHeight: 400, color: "#d13438" }}>{result.message}</pre>
              {result.raw_report && (
                <details className="raw-text-details" style={{ marginTop: 8 }}>
                  <summary>レスポンス詳細</summary>
                  <pre className="raw-response">{result.raw_report}</pre>
                </details>
              )}
            </>
          )}
        </div>
      )}
    </div>
  );
}

export default DeepResearchApiRunner;
