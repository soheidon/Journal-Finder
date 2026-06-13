import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ActiveView, ProjectInfo } from "../App";

interface HomePanelProps {
  onNavigate: (view: ActiveView) => void;
  projectInfo: ProjectInfo | null;
  onCreateProject: (name: string, parentDir: string) => void;
  onOpenProject: (path: string) => void;
}

function HomePanel({ onNavigate, projectInfo, onCreateProject, onOpenProject }: HomePanelProps) {
  const [recentProjects, setRecentProjects] = useState<ProjectInfo[]>([]);
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [parentDir, setParentDir] = useState("");

  useEffect(() => {
    invoke<ProjectInfo[]>("list_recent_projects").then(setRecentProjects).catch(() => {});
  }, []);

  const handleSelectFolder = useCallback(async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, multiple: false });
      if (selected) setParentDir(selected as string);
    } catch (_) {}
  }, []);

  const handleCreate = useCallback(() => {
    if (!newName.trim() || !parentDir) return;
    onCreateProject(newName.trim(), parentDir);
    setShowCreate(false);
    setNewName("");
  }, [newName, parentDir, onCreateProject]);

  const handleOpenFolder = useCallback(async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, multiple: false });
      if (selected) onOpenProject(selected as string);
    } catch (_) {}
  }, [onOpenProject]);

  return (
    <div className="panel home-panel">
      <h2>Journal Finder 論文投稿先アドバイザ</h2>
      <p className="home-description">
        論文原稿を入力し、投稿先ジャーナル候補を推薦するツールです。
      </p>

      {/* Project section */}
      <div className="input-section">
        <h3>プロジェクト</h3>
        {projectInfo ? (
          <div className="project-current">
            <div className="project-info">
              <strong>{projectInfo.name}</strong>
              <span className="project-path">{projectInfo.path}</span>
            </div>
          </div>
        ) : (
          <p className="hint-text">論文を分析するには、まずプロジェクトを作成してください。プロジェクトフォルダに docx、要約、検索結果、レポートがまとめて保存されます。</p>
        )}

        <div className="action-row" style={{ marginTop: 8 }}>
          <button onClick={() => setShowCreate(!showCreate)}>新規プロジェクト</button>
          <button onClick={handleOpenFolder}>既存プロジェクトを開く</button>
        </div>

        {showCreate && (
          <div className="project-create-form">
            <div className="form-grid">
              <label>プロジェクト名</label>
              <input type="text" value={newName} onChange={e => setNewName(e.target.value)} placeholder="例: sleep-study-2026" />
              <label>保存先</label>
              <div className="model-row">
                <input type="text" value={parentDir} readOnly placeholder="フォルダを選択..." />
                <button onClick={handleSelectFolder}>選択</button>
              </div>
            </div>
            <div className="action-row" style={{ marginTop: 8 }}>
              <button onClick={handleCreate} disabled={!newName.trim() || !parentDir}>作成</button>
              <button onClick={() => setShowCreate(false)}>キャンセル</button>
            </div>
          </div>
        )}

        {recentProjects.length > 0 && !projectInfo && (
          <div className="recent-projects">
            <h4>最近のプロジェクト</h4>
            {recentProjects.map((p, i) => (
              <button key={i} className="recent-project-item" onClick={() => onOpenProject(p.path)}>
                <strong>{p.name}</strong>
                <span className="recent-project-path">{p.path}</span>
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Steps */}
      <div className="home-steps">
        <h3>使い方</h3>
        <ol>
          <li>
            <button className="link-button" onClick={() => onNavigate("settings")}>
              API設定
            </button>
            で LLM の接続設定を行う
          </li>
          <li>プロジェクトを作成または開く</li>
          <li>
            <button className="link-button" onClick={() => onNavigate("input")}>
              入力画面
            </button>
            で docx ファイルを選択し、テキスト抽出を実行
          </li>
          <li>
            <button className="link-button" onClick={() => onNavigate("summary")}>
              論文要約
            </button>
            で要約を生成し、
            <button className="link-button" onClick={() => onNavigate("positioning")}>
              立ち位置調査
            </button>
            →
            <button className="link-button" onClick={() => onNavigate("journal_search")}>
              ジャーナル調査
            </button>
            で投稿先を探す
          </li>
          <li>
            <button className="link-button" onClick={() => onNavigate("results")}>
              結果画面
            </button>
            で推薦結果を確認・エクスポート
          </li>
        </ol>
      </div>
    </div>
  );
}

export default HomePanel;
