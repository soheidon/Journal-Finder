import { useState } from "react";

interface PasteModalProps {
  isOpen: boolean;
  searchPrompt: string;
  onClose: () => void;
  onParse: (externalA: string, externalB: string) => void;
  isParsing: boolean;
}

function PasteModal({ isOpen, searchPrompt, onClose, onParse, isParsing }: PasteModalProps) {
  const [externalA, setExternalA] = useState("");
  const [externalB, setExternalB] = useState("");
  const [promptCopied, setPromptCopied] = useState(false);

  if (!isOpen) return null;

  const handleCopyPrompt = async () => {
    await navigator.clipboard.writeText(searchPrompt);
    setPromptCopied(true);
    setTimeout(() => setPromptCopied(false), 2000);
  };

  const handleParse = () => {
    if (!externalA.trim()) return;
    onParse(externalA, externalB);
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>Deep Research 結果を貼り付け</h3>
          <button className="modal-close" onClick={onClose}>✕</button>
        </div>

        <div className="modal-body">
          <div className="modal-section">
            <div className="modal-section-header">
              <h4>検索プロンプト</h4>
              <button onClick={handleCopyPrompt} className="copy-btn">
                {promptCopied ? "コピー済み ✓" : "コピー"}
              </button>
            </div>
            <p className="hint-text">
              以下のプロンプトを ChatGPT / Perplexity / Gemini / Claude 等の Deep Research に貼り付けてください。
            </p>
            <textarea
              className="prompt-textarea"
              value={searchPrompt}
              readOnly
              rows={6}
            />
          </div>

          <div className="modal-section">
            <h4>外部 AI A の検索結果（必須）</h4>
            <p className="hint-text">外部 AI の Deep Research 結果を貼り付けてください。</p>
            <textarea
              className="result-textarea"
              value={externalA}
              onChange={(e) => setExternalA(e.target.value)}
              placeholder="ここに外部 AI の検索結果を貼り付けてください..."
              rows={8}
            />
          </div>

          <div className="modal-section">
            <h4>外部 AI B の検索結果（任意）</h4>
            <p className="hint-text">別の AI で同じプロンプトを実行した場合、結果を貼り付けてください。2 つの結果を統合します。</p>
            <textarea
              className="result-textarea"
              value={externalB}
              onChange={(e) => setExternalB(e.target.value)}
              placeholder="2 つ目の AI の検索結果を貼り付け（任意）..."
              rows={6}
            />
          </div>
        </div>

        <div className="modal-footer">
          <button className="modal-cancel" onClick={onClose}>キャンセル</button>
          <button
            className="modal-parse"
            onClick={handleParse}
            disabled={!externalA.trim() || isParsing}
          >
            {isParsing ? "解析中..." : "貼り付け結果を解析"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default PasteModal;
