import type { ManuscriptText, PipelineStatus } from "../App";

interface InputPanelProps {
  docxPath: string;
  onDocxPathChange: (path: string) => void;
  manuscriptText: ManuscriptText | null;
  extractStatus: PipelineStatus;
  onExtract: () => void;
}

const statusLabels: Record<PipelineStatus, { label: string; cls: string }> = {
  not_started: { label: "未実行", cls: "unrun" },
  in_progress: { label: "実行中...", cls: "running" },
  done: { label: "完了", cls: "ok" },
  failed: { label: "エラー", cls: "err" },
};

function InputPanel({ docxPath, onDocxPathChange, manuscriptText, extractStatus, onExtract }: InputPanelProps) {
  const handleFileSelect = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [{ name: "Word Document", extensions: ["docx"] }],
      });
      if (selected) {
        onDocxPathChange(selected as string);
      }
    } catch (e) {
      console.error("File dialog error:", e);
    }
  };

  const st = statusLabels[extractStatus];

  return (
    <div className="panel input-panel">
      <div className="input-section info-card">
        <h3>入力について</h3>
        <p>
          分析する論文の docx ファイルを選択し、テキスト抽出を行います。
          抽出されたテキストは次の「検索」タブで要約生成に使われます。
        </p>
      </div>

      <div className="input-section">
        <h3>docx ファイル選択</h3>
        <div className="file-row">
          <button onClick={handleFileSelect}>ファイルを選択</button>
          <span className="file-path">{docxPath || "ファイルが選択されていません"}</span>
        </div>
      </div>

      <div className="input-section">
        <h3>テキスト抽出</h3>
        <div className="action-row">
          <button onClick={onExtract} disabled={!docxPath || extractStatus === "in_progress"}>
            {extractStatus === "in_progress" ? "抽出中..." : "テキスト抽出を実行"}
          </button>
          <span className={`status-chip ${st.cls}`}>{st.label}</span>
        </div>
        {!docxPath && <p className="hint-text">先に docx ファイルを選択してください</p>}
      </div>

      {manuscriptText && (
        <div className="input-section">
          <h3>抽出結果プレビュー</h3>
          <div className="stats-row">
            <span>段落数: {manuscriptText.paragraph_count}</span>
            <span>文字数: {manuscriptText.char_count}</span>
            <span>セクション数: {manuscriptText.sections.length}</span>
          </div>
          <div className="section-list">
            <h4>セクション一覧</h4>
            {manuscriptText.sections.map((sec, i) => (
              <div key={i} className="section-item">
                <strong>{sec.heading}</strong>
                <p>{sec.body.length > 150 ? sec.body.substring(0, 150) + "..." : sec.body}</p>
              </div>
            ))}
          </div>
          <details className="raw-text-details">
            <summary>全文テキスト</summary>
            <pre>{manuscriptText.raw_text}</pre>
          </details>
        </div>
      )}
    </div>
  );
}

export default InputPanel;
