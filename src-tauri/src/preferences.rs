use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Serialize, Deserialize, Default)]
struct UserPreferences {
    last_idea_version: Option<String>,
}

fn get_prefs_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("failed to create app data dir");
    }
    app_dir.join("preferences.json")
}

fn load_prefs<R: Runtime>(app: &AppHandle<R>) -> UserPreferences {
    let path = get_prefs_path(app);
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        UserPreferences::default()
    }
}

fn save_prefs<R: Runtime>(app: &AppHandle<R>, prefs: &UserPreferences) -> Result<(), String> {
    let path = get_prefs_path(app);
    let json = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

/// 获取上次保存的 IDEA 版本路径
#[tauri::command]
pub fn get_saved_idea_version<R: Runtime>(app: AppHandle<R>) -> Option<String> {
    load_prefs(&app).last_idea_version
}

/// 保存用户选择的 IDEA 版本路径
#[tauri::command]
pub fn save_idea_version<R: Runtime>(app: AppHandle<R>, workspace_path: String) -> Result<(), String> {
    let mut prefs = load_prefs(&app);
    prefs.last_idea_version = Some(workspace_path);
    save_prefs(&app, &prefs)
}
