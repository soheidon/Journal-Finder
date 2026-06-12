use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
    pub docx_file: String,
    pub has_summary: bool,
    pub has_journals: bool,
}

fn project_json_path(project_dir: &str) -> PathBuf {
    Path::new(project_dir).join("project.json")
}

fn now_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    format!("{}", secs)
}

#[tauri::command]
pub fn create_project(name: String, parent_dir: String) -> Result<ProjectInfo, String> {
    let project_dir = Path::new(&parent_dir).join(&name);
    if project_dir.exists() {
        return Err(format!("フォルダ '{}' は既に存在します", project_dir.display()));
    }
    fs::create_dir_all(&project_dir).map_err(|e| format!("フォルダ作成失敗: {}", e))?;
    fs::create_dir_all(project_dir.join("source")).ok();

    let info = ProjectInfo {
        name,
        path: project_dir.to_string_lossy().to_string(),
        created_at: now_str(),
        updated_at: now_str(),
        docx_file: String::new(),
        has_summary: false,
        has_journals: false,
    };
    save_project_json(&project_dir, &info)?;
    Ok(info)
}

#[tauri::command]
pub fn open_project(path: String) -> Result<ProjectInfo, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("プロジェクトフォルダが見つかりません".to_string());
    }
    let json_path = project_json_path(&path);
    if !json_path.exists() {
        return Err("project.json が見つかりません".to_string());
    }
    let data = fs::read_to_string(&json_path).map_err(|e| format!("読み込み失敗: {}", e))?;
    let info: ProjectInfo = serde_json::from_str(&data).map_err(|e| format!("JSON エラー: {}", e))?;
    Ok(info)
}

#[tauri::command]
pub fn save_project(info: ProjectInfo) -> Result<(), String> {
    let p = Path::new(&info.path);
    save_project_json(p, &info)
}

#[tauri::command]
pub fn save_project_file(project_dir: String, filename: String, content: String) -> Result<(), String> {
    let path = Path::new(&project_dir).join(&filename);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&path, &content).map_err(|e| format!("ファイル保存失敗: {}", e))
}

#[tauri::command]
pub fn load_project_file(project_dir: String, filename: String) -> Result<String, String> {
    let path = Path::new(&project_dir).join(&filename);
    if !path.exists() {
        return Err(format!("ファイル '{}' が見つかりません", filename));
    }
    fs::read_to_string(&path).map_err(|e| format!("読み込み失敗: {}", e))
}

#[tauri::command]
pub fn list_recent_projects() -> Vec<ProjectInfo> {
    let recent_path = recent_projects_path();
    if !recent_path.exists() {
        return vec![];
    }
    let data = fs::read_to_string(&recent_path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

#[tauri::command]
pub fn add_recent_project(info: ProjectInfo) {
    let mut recent = list_recent_projects();
    recent.retain(|p| p.path != info.path);
    recent.insert(0, info);
    if recent.len() > 10 {
        recent.truncate(10);
    }
    let recent_path = recent_projects_path();
    if let Some(parent) = recent_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Ok(json) = serde_json::to_string_pretty(&recent) {
        fs::write(&recent_path, json).ok();
    }
}

fn recent_projects_path() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("journal-finder");
    dir.push("recent_projects.json");
    dir
}

fn save_project_json(project_dir: &Path, info: &ProjectInfo) -> Result<(), String> {
    let path = project_dir.join("project.json");
    let json = serde_json::to_string_pretty(info).map_err(|e| format!("JSON エラー: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("保存失敗: {}", e))
}
