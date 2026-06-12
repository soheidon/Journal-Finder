import type { ActiveView } from "./App";

interface SidebarProps {
  activeView: ActiveView;
  onNavigate: (view: ActiveView) => void;
}

const menuItems: { key: ActiveView; icon: string; label: string }[] = [
  { key: "home", icon: "🏠", label: "ホーム" },
  { key: "settings", icon: "⚙️", label: "API設定" },
  { key: "input", icon: "📄", label: "入力" },
  { key: "summary", icon: "📝", label: "論文要約" },
  { key: "positioning", icon: "🔬", label: "立ち位置調査" },
  { key: "journal_search", icon: "🔍", label: "ジャーナル調査" },
  { key: "results", icon: "📊", label: "結果" },
];

function Sidebar({ activeView, onNavigate }: SidebarProps) {
  return (
    <nav className="sidebar">
      {menuItems.map((item) => (
        <button
          key={item.key}
          className={`sidebar-item ${activeView === item.key ? "active" : ""}`}
          onClick={() => onNavigate(item.key)}
        >
          <span className="sidebar-icon">{item.icon}</span>
          <span className="sidebar-label">{item.label}</span>
        </button>
      ))}
    </nav>
  );
}

export default Sidebar;
