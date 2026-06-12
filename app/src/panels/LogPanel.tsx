interface LogPanelProps {
  logs: string[];
  expanded: boolean;
  onToggle: () => void;
  onClear: () => void;
}

function LogPanel({ logs, expanded, onToggle, onClear }: LogPanelProps) {
  return (
    <div className={`log-panel ${expanded ? "expanded" : "collapsed"}`}>
      <div className="log-header" onClick={onToggle}>
        <span>▼ ログ ({logs.length} 件)</span>
        {expanded && (
          <button
            className="log-clear-btn"
            onClick={(e) => {
              e.stopPropagation();
              onClear();
            }}
          >
            ログを消去
          </button>
        )}
      </div>
      {expanded && (
        <div className="log-body">
          {logs.length === 0 ? (
            <p className="log-empty">ログはまだありません</p>
          ) : (
            logs.map((line, i) => (
              <div key={i} className="log-line">{line}</div>
            ))
          )}
        </div>
      )}
    </div>
  );
}

export default LogPanel;
