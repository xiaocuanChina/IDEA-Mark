use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Runtime, Manager};
use chrono::Local;

/// 用于直接展示的书签结构，包含用户友好的字段
#[derive(serde::Serialize, Clone, Debug)]
pub struct ParsedBookmark {
    /// 项目名称（从 XML 文件名或路径解析）
    pub project_name: String,
    /// 文件名（不含路径）
    pub file_name: String,
    /// 完整文件路径
    pub file_path: String,
    /// 行号（已转换为 1-indexed）
    pub line_number: i32,
    /// 书签描述
    pub description: String,
    /// 助记符（如 1, 2, A, B 等）
    pub mnemonic: Option<String>,
    /// 书签类型（匿名/助记符）
    pub bookmark_type: String,
}

#[derive(serde::Serialize, Clone)]
pub struct IdeaVersion {
    pub name: String,
    pub path: String,
    pub workspace_path: String,
}

#[derive(serde::Serialize, Clone)]
pub struct WorkspaceFile {
    pub name: String,
    pub path: String,
    pub modified_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct BackupFile {
    pub id: String,
    pub original_file_name: String,
    pub timestamp: String,
    pub path: String,
    pub projects: Vec<String>,  // 备份包含的项目名列表
}

// Helper to get the base JetBrains roaming directory
fn get_jetbrains_roaming_dir() -> Option<PathBuf> {
    // Windows: C:\Users\{User}\AppData\Roaming\JetBrains
    dirs::config_dir().map(|p: PathBuf| p.join("JetBrains"))
}

#[tauri::command]
pub fn find_idea_dirs() -> Result<Vec<IdeaVersion>, String> {
    let jb_dir = get_jetbrains_roaming_dir().ok_or("Could not find AppData directory")?;
    
    if !jb_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&jb_dir).map_err(|e| e.to_string())?;
    let mut versions = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("IntelliJIdea") {
                    let workspace_path = path.join("workspace");
                     // Even if workspace subdir doesn't exist yet, we list it so user knows it was detected
                    versions.push(IdeaVersion {
                        name: name.to_string(),
                        path: path.to_string_lossy().to_string(),
                        workspace_path: workspace_path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    
    // Sort reverse to get latest versions first roughly
    versions.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(versions)
}

#[tauri::command]
pub fn list_workspace_files(workspace_path: String) -> Result<Vec<WorkspaceFile>, String> {
    let path = Path::new(&workspace_path);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut files = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if p.is_file() && p.extension().map_or(false, |ext| ext == "xml") {
             let metadata = fs::metadata(&p).map_err(|e| e.to_string())?;
             let modified: chrono::DateTime<Local> = metadata.modified().unwrap_or(std::time::SystemTime::now()).into();
             
             files.push(WorkspaceFile {
                 name: p.file_name().unwrap_or_default().to_string_lossy().to_string(),
                 path: p.to_string_lossy().to_string(),
                 modified_at: modified.format("%Y-%m-%d %H:%M:%S").to_string(),
             });
        }
    }
    Ok(files)
}

fn get_backup_dir<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
    app_dir.join("backups")
}

fn get_backup_meta_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
    app_dir.join("backup_meta.json")
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct BackupMeta {
    // 备份ID -> 项目名列表
    entries: std::collections::HashMap<String, Vec<String>>,
}

fn load_backup_meta<R: Runtime>(app: &AppHandle<R>) -> BackupMeta {
    let meta_path = get_backup_meta_path(app);
    if meta_path.exists() {
        if let Ok(content) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str(&content) {
                return meta;
            }
        }
    }
    BackupMeta::default()
}

fn save_backup_meta<R: Runtime>(app: &AppHandle<R>, meta: &BackupMeta) -> Result<(), String> {
    let meta_path = get_backup_meta_path(app);
    let content = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(meta_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn backup_bookmark_file<R: Runtime>(
    app: AppHandle<R>, 
    file_path: String,
    projects: Vec<String>
) -> Result<String, String> {
    let source_path = Path::new(&file_path);
    if !source_path.exists() {
        return Err("Source file does not exist".to_string());
    }
    
    let file_name = source_path.file_name().ok_or("Invalid file name")?.to_string_lossy();
    let backup_dir = get_backup_dir(&app);
    
    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}_{}", timestamp, "backup", file_name);
    let backup_path = backup_dir.join(&backup_name);

    fs::copy(source_path, &backup_path).map_err(|e| e.to_string())?;

    // 保存项目名元数据
    let mut meta = load_backup_meta(&app);
    meta.entries.insert(backup_name.clone(), projects);
    save_backup_meta(&app, &meta)?;

    Ok(backup_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_backup_list<R: Runtime>(app: AppHandle<R>) -> Result<Vec<BackupFile>, String> {
    let backup_dir = get_backup_dir(&app);
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let meta = load_backup_meta(&app);
    let entries = fs::read_dir(backup_dir).map_err(|e| e.to_string())?;
    let mut backups = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if p.is_file() {
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            // Expected format: YYYYMMDD_HHMMSS_backup_filename.xml
            let parts: Vec<&str> = name.splitn(4, '_').collect();
            if parts.len() >= 4 && parts[2] == "backup" {
                let original_name = parts[3];
                
                // Format timestamp for display
                let display_time = if parts[0].len() >= 8 && parts[1].len() >= 6 {
                    format!("{}-{}-{} {}:{}:{}", 
                        &parts[0][0..4], &parts[0][4..6], &parts[0][6..8],
                        &parts[1][0..2], &parts[1][2..4], &parts[1][4..6]
                    )
                } else {
                    format!("{}_{}", parts[0], parts[1])
                };

                // 获取项目名列表
                let projects = meta.entries.get(&name).cloned().unwrap_or_default();

                backups.push(BackupFile {
                    id: name.clone(),
                    original_file_name: original_name.to_string(),
                    timestamp: display_time,
                    path: p.to_string_lossy().to_string(),
                    projects,
                });
            }
        }
    }
    
    // Sort newest first
    backups.sort_by(|a, b| b.id.cmp(&a.id));
    
    Ok(backups)
}

/// 检查 IntelliJ IDEA 是否正在运行
#[tauri::command]
pub fn check_idea_running() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq idea64.exe", "/NH"])
            .output()
            .map_err(|e| e.to_string())?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        // 如果找到进程，输出会包含 "idea64.exe"
        if stdout.contains("idea64.exe") {
            return Ok(true);
        }
        
        // 也检查 32 位版本
        let output32 = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq idea.exe", "/NH"])
            .output()
            .map_err(|e| e.to_string())?;
        
        let stdout32 = String::from_utf8_lossy(&output32.stdout);
        Ok(stdout32.contains("idea.exe"))
    }
    
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pgrep")
            .args(["-f", "IntelliJ IDEA"])
            .output()
            .map_err(|e| e.to_string())?;
        
        Ok(!output.stdout.is_empty())
    }
    
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("pgrep")
            .args(["-f", "idea"])
            .output()
            .map_err(|e| e.to_string())?;
        
        Ok(!output.stdout.is_empty())
    }
}

#[tauri::command]
pub fn restore_bookmark_file(backup_path: String, target_path: String) -> Result<(), String> {
    let backup = Path::new(&backup_path);
    let target = Path::new(&target_path);

    if !backup.exists() {
        return Err("Backup file not found".to_string());
    }

    // Ensure target directory exists (it should, but just in case)
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::copy(backup, target).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_backup_file<R: Runtime>(app: AppHandle<R>, backup_path: String) -> Result<(), String> {
    let backup = Path::new(&backup_path);
    
    if !backup.exists() {
        return Err("Backup file not found".to_string());
    }
    
    // 获取文件名用于清理元数据
    let file_name = backup.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    fs::remove_file(backup).map_err(|e| e.to_string())?;
    
    // 清理元数据
    let mut meta = load_backup_meta(&app);
    meta.entries.remove(&file_name);
    let _ = save_backup_meta(&app, &meta);
    
    Ok(())
}

/// 读取备份文件中的书签内容
#[tauri::command]
pub fn read_backup_bookmarks(backup_path: String) -> Result<Vec<ParsedBookmark>, String> {
    let path = Path::new(&backup_path);
    if !path.exists() {
        return Err("Backup file not found".to_string());
    }
    
    // 备份文件就是原始 XML 文件的副本，直接使用现有解析函数
    let bookmarks = parse_bookmarks_from_global_workspace(path);
    Ok(bookmarks)
}

/// 辅助函数：提取 XML 属性值
fn extract_attr_value(chunk: &str, attr: &str) -> Option<String> {
    let patterns = [format!("{}=\"", attr), format!("{}='", attr)];
    for p in patterns {
        if let Some(idx) = chunk.find(&p) {
            let start = idx + p.len();
            let quote = p.chars().last().unwrap();
            if let Some(end) = chunk[start..].find(quote) {
                return Some(chunk[start..start+end].to_string());
            }
        }
    }
    None
}

/// 辅助函数：处理路径，将 $PROJECT_DIR$ 替换为实际路径
fn clean_bookmark_path(raw_url: &str, project_root: Option<&str>) -> String {
    let mut p = raw_url.to_string();
    if p.contains("$PROJECT_DIR$") {
        if let Some(root) = project_root {
            p = p.replace("file://$PROJECT_DIR$", root);
        } else {
            p = p.replace("file://$PROJECT_DIR$", "[项目根目录]");
        }
    } else {
        p = p.replace("file://", "");
    }
    // Windows 路径修正: /C:/... -> C:/...
    if p.starts_with('/') && p.chars().nth(2) == Some(':') {
        p = p[1..].to_string();
    }
    p
}

/// 从全局 workspace XML 文件解析书签 (IDEA 2025+ 新格式)
/// 这些文件位于 AppData/Roaming/JetBrains/IntelliJIdea20XX.X/workspace/*.xml
fn parse_bookmarks_from_global_workspace(file_path: &Path) -> Vec<ParsedBookmark> {
    let mut bookmarks = Vec::new();
    
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            println!("  Failed to read file {:?}: {}", file_path, e);
            return bookmarks;
        }
    };

    // 检查是否包含 BookmarksManager 组件
    if !content.contains("BookmarksManager") {
        return bookmarks;
    }

    println!("  Found BookmarksManager in {:?}", file_path);

    // 按 GroupState 分割，每个 GroupState 代表一个项目的书签组
    let group_chunks: Vec<&str> = content.split("<GroupState>").collect();
    
    for group_chunk in group_chunks.iter().skip(1) {
        // 提取项目名称 (在 GroupState 末尾的 <option name="name" value="xxx" />)
        let project_name = if let Some(name_idx) = group_chunk.find("<option name=\"name\"") {
            extract_attr_value(&group_chunk[name_idx..], "value")
                .unwrap_or_else(|| "未知项目".to_string())
        } else {
            "未知项目".to_string()
        };

        println!("    Processing group: {}", project_name);

        // 按 BookmarkState 分割
        let bookmark_chunks: Vec<&str> = group_chunk.split("<BookmarkState>").collect();
        
        for bm_chunk in bookmark_chunks.iter().skip(1) {
            // 只处理到 </BookmarkState> 之前的内容
            let bm_content = bm_chunk.split("</BookmarkState>").next().unwrap_or(bm_chunk);
            
            // 提取 URL (在 <entry key="url" value="..." />)
            let url_opt = if let Some(url_idx) = bm_content.find("key=\"url\"") {
                extract_attr_value(&bm_content[url_idx..], "value")
            } else {
                None
            };

            if let Some(raw_url) = url_opt {
                // 暂时不替换 $PROJECT_DIR$，保留原始路径显示
                let file_path_str = clean_bookmark_path(&raw_url, None);
                
                let file_name = std::path::Path::new(&file_path_str)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "未知文件".to_string());

                // 提取行号
                let line_number = if let Some(line_idx) = bm_content.find("key=\"line\"") {
                    extract_attr_value(&bm_content[line_idx..], "value")
                        .and_then(|s| s.parse::<i32>().ok())
                        .map(|l| l + 1)
                        .unwrap_or(1)
                } else {
                    1
                };

                // 提取描述 (在 <option name="description" value="..." />)
                let mut description = String::new();
                if let Some(desc_idx) = bm_content.find("name=\"description\"") {
                    if let Some(d) = extract_attr_value(&bm_content[desc_idx..], "value") {
                        description = d.replace("&quot;", "\"")
                            .replace("&lt;", "<")
                            .replace("&gt;", ">")
                            .replace("&amp;", "&");
                    }
                }

                // 提取助记符
                let mnemonic = if let Some(mn_idx) = bm_content.find("key=\"mnemonic\"") {
                    extract_attr_value(&bm_content[mn_idx..], "value").filter(|s| !s.is_empty())
                } else {
                    None
                };

                let bookmark_type = if mnemonic.is_some() { "助记符书签" } else { "匿名书签" }.to_string();

                bookmarks.push(ParsedBookmark {
                    project_name: project_name.clone(),
                    file_name,
                    file_path: file_path_str,
                    line_number,
                    description,
                    mnemonic,
                    bookmark_type,
                });
            }
        }
    }

    println!("    Parsed {} bookmarks from this file", bookmarks.len());
    bookmarks
}

/// 从项目 .idea 目录的 XML 文件解析书签 (旧格式兼容)
fn parse_bookmarks_from_project_idea(file_path: &Path, project_name: &str) -> Vec<ParsedBookmark> {
    let mut bookmarks = Vec::new();
    
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return bookmarks,
    };

    // 传统格式 (<bookmark ...>)
    if content.contains("<bookmark") {
        let chunks: Vec<&str> = content.split("<bookmark").collect();
        for chunk in chunks.iter().skip(1) {
            let url_opt = extract_attr_value(chunk, "url");
            if let Some(raw_url) = url_opt {
                let file_path_str = clean_bookmark_path(&raw_url, None);
                
                let file_name = std::path::Path::new(&file_path_str)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "未知文件".to_string());

                let line_number = extract_attr_value(chunk, "line")
                    .and_then(|s| s.parse::<i32>().ok())
                    .map(|l| l + 1)
                    .unwrap_or(1);
                
                let mut description = extract_attr_value(chunk, "description").unwrap_or_default();
                description = description.replace("&quot;", "\"")
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&amp;", "&");

                let mnemonic = extract_attr_value(chunk, "mnemonic");
                let bookmark_type = if mnemonic.is_some() { "助记符书签" } else { "匿名书签" }.to_string();

                bookmarks.push(ParsedBookmark {
                    project_name: project_name.to_string(),
                    file_name,
                    file_path: file_path_str,
                    line_number,
                    description,
                    mnemonic,
                    bookmark_type,
                });
            }
        }
    }

    bookmarks
}

/// 解析 recentProjects.xml 获取项目路径列表
fn get_recent_projects(options_dir: &Path) -> Vec<PathBuf> {
    let mut projects = Vec::new();
    let recent_xml = options_dir.join("recentProjects.xml");
    
    if !recent_xml.exists() {
        println!("recentProjects.xml not found at {:?}", recent_xml);
        return projects;
    }

    let content = match fs::read_to_string(&recent_xml) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to read recentProjects.xml: {}", e);
            return projects;
        }
    };

    // 简单解析：寻找包含路径的 value="..." 或 key="..."
    // 路径特征：包含 "/" 或 "\" 或者是 $USER_HOME$
    // 更准确：查找 <entry key="..."> 或 <option value="...">
    
    let home_dir = dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    
    // 粗略策略：按引号分割，找出看起来像路径的字符串
    // 这种方法虽然粗糙，但能涵盖多种 XML 变体
    let parts: Vec<&str> = content.split('"').collect();
    for part in parts {
        if part.contains('/') || part.contains('\\') || part.contains("$USER_HOME$") {
            // 过滤掉明显的非项目路径（如 jar 包、插件路径等）
            if part.ends_with(".jar") || part.contains(".svg") || part.contains(".xml") {
                continue;
            }
            
            let resolved = part.replace("$USER_HOME$", &home_dir);
            let path = PathBuf::from(resolved);
            
            // 验证是否为有效目录
            if path.exists() && path.is_dir() {
                // 必须包含 .idea 目录才算 IDEA 项目
                if path.join(".idea").exists() {
                    if !projects.contains(&path) {
                        projects.push(path);
                    }
                }
            }
        }
    }
    
    projects
}

/// 从 workspace 目录读取所有书签（不保存到数据库，直接返回给前端展示）
#[tauri::command]
pub fn read_bookmarks_from_workspace(workspace_path: String) -> Result<Vec<ParsedBookmark>, String> {
    let ws_path = Path::new(&workspace_path);
    let config_dir = ws_path.parent().ok_or("Cannot find config parent dir")?;
    
    println!("Scanning bookmarks from config dir: {:?}", config_dir);
    println!("Workspace path: {:?}", ws_path);

    let mut all_bookmarks = Vec::new();

    // 方法1: 从全局 workspace 目录读取 (IDEA 2025+ 新格式)
    // 书签存储在 workspace/*.xml 文件中
    if ws_path.exists() && ws_path.is_dir() {
        println!("Scanning global workspace directory: {:?}", ws_path);
        
        if let Ok(entries) = fs::read_dir(ws_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "xml") {
                    println!("  Checking file: {:?}", path);
                    let bookmarks = parse_bookmarks_from_global_workspace(&path);
                    all_bookmarks.extend(bookmarks);
                }
            }
        }
    }

    // 方法2: 如果全局 workspace 没找到书签，尝试从项目 .idea 目录读取 (旧版兼容)
    if all_bookmarks.is_empty() {
        println!("No bookmarks found in global workspace, trying project .idea directories...");
        
        let options_dir = config_dir.join("options");
        let project_paths = get_recent_projects(&options_dir);
        println!("Found {} recent projects", project_paths.len());

        for proj_path in &project_paths {
            let idea_dir = proj_path.join(".idea");
            let project_name = proj_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or("Unknown Project".to_string());

            // 检查 bookmarks.xml
            let bm_xml = idea_dir.join("bookmarks.xml");
            if bm_xml.exists() {
                let b = parse_bookmarks_from_project_idea(&bm_xml, &project_name);
                println!("  Found {} bookmarks in {:?}", b.len(), bm_xml);
                all_bookmarks.extend(b);
            }

            // 检查 workspace.xml
            let ws_xml = idea_dir.join("workspace.xml");
            if ws_xml.exists() {
                let b = parse_bookmarks_from_project_idea(&ws_xml, &project_name);
                println!("  Found {} bookmarks in {:?}", b.len(), ws_xml);
                // 去重
                for copy in b {
                    let exists = all_bookmarks.iter().any(|existing| 
                        existing.file_path == copy.file_path && existing.line_number == copy.line_number
                    );
                    if !exists {
                        all_bookmarks.push(copy);
                    }
                }
            }
        }
    }

    println!("Total bookmarks found: {}", all_bookmarks.len());

    // 按项目名称排序
    all_bookmarks.sort_by(|a, b| a.project_name.cmp(&b.project_name));
    
    Ok(all_bookmarks)
}
