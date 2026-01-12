use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug)]
pub struct Bookmark {
    pub id: Option<i64>,
    pub title: String,
    pub file_path: String,
    pub line_number: i32,
    pub content: String,
    pub created_at: Option<String>,
    pub project: String,
}

fn get_db_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
    }
    app_dir.join("bookmarks.db")
}

pub fn init_db<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path(app);
    let conn = Connection::open(db_path)?;

    // Drop table to ensure schema update (User requested fresh start/fix)
    conn.execute("DROP TABLE IF EXISTS bookmarks", [])?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS bookmarks (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            file_path TEXT NOT NULL,
            line_number INTEGER,
            content TEXT,
            created_at TEXT NOT NULL,
            project TEXT DEFAULT 'Unknown'
        )",
        [],
    )?;
    Ok(())
}

#[tauri::command]
pub fn get_bookmarks<R: Runtime>(app: AppHandle<R>) -> Result<Vec<Bookmark>, String> {
    let db_path = get_db_path(&app);
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, title, file_path, line_number, content, created_at, project FROM bookmarks ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let bookmark_iter = stmt
        .query_map([], |row| {
            Ok(Bookmark {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                file_path: row.get(2)?,
                line_number: row.get(3)?,
                content: row.get(4)?,
                created_at: Some(row.get(5)?),
                project: row.get(6).unwrap_or("Unknown".to_string()),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut bookmarks = Vec::new();
    for bookmark in bookmark_iter {
        bookmarks.push(bookmark.map_err(|e| e.to_string())?);
    }
    Ok(bookmarks)
}

#[tauri::command]
pub fn add_bookmark<R: Runtime>(
    app: AppHandle<R>,
    title: String,
    file_path: String,
    line_number: i32,
    content: String,
) -> Result<(), String> {
    let db_path = get_db_path(&app);
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let now = Utc::now().to_rfc3339();
    let project = extract_project_name(&file_path);

    conn.execute(
        "INSERT INTO bookmarks (title, file_path, line_number, content, created_at, project) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![title, file_path, line_number, content, now, project],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_bookmark<R: Runtime>(app: AppHandle<R>, id: i64) -> Result<(), String> {
    let db_path = get_db_path(&app);
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn extract_project_name(path: &str) -> String {
    let p = std::path::Path::new(path);
    // Simple heuristic: Try to find "src" and take parent, or just take the parent of the file
    // Enhanced: iterate components, look for standard structure
    for component in p.components() {
         if let Some(s) = component.as_os_str().to_str() {
             if s.eq_ignore_ascii_case("src") {
                 // Return the component before src?
                 let _parent = p.parent().unwrap_or(p); // This is just the file's parent if passed file
                 // Correct logic: we need to find where 'src' is in the path string
                 // Easier: split by 'src'
                 let parts: Vec<&str> = path.split("src").collect();
                 if !parts.is_empty() {
                     let pre_src = std::path::Path::new(parts[0]);
                     return pre_src.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or("Unknown".to_string());
                 }
             }
         }
    }
    // Fallback: take parent directory name
    p.parent()
        .and_then(|parent| parent.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or("Unknown".to_string())
}

#[tauri::command]
pub fn import_bookmarks<R: Runtime>(app: AppHandle<R>, file_path: String) -> Result<String, String> {
    println!("Starting import from: {}", file_path);
    let content = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    
    // Determine project root only for relative paths
    let path_obj = std::path::Path::new(&file_path);
    let project_root = path_obj.parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    println!("Project root context (if needed): {}", project_root);

    let db_path = get_db_path(&app);
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    // Helper closure for attribute extraction
    let extract_attr = |chunk: &str, attr: &str| -> Option<String> {
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
    };

    let chunks: Vec<&str> = content.split("<bookmark").collect();
    let mut count = 0;
    
    // Log the first chunk to see what we are dealing with (it might be the header)
    if !chunks.is_empty() {
        println!("Header chunk info: {}...", &chunks[0].chars().take(50).collect::<String>());
    }

    for (i, chunk) in chunks.iter().skip(1).enumerate() {
        let url_opt = extract_attr(chunk, "url");
        if let Some(raw_url) = url_opt {
            let mut file_path = raw_url.clone();
            
            // Handle $PROJECT_DIR$
            if file_path.contains("$PROJECT_DIR$") {
                file_path = file_path.replace("file://$PROJECT_DIR$", &project_root);
            } else {
                // Handle absolute paths like file://C:/...
                file_path = file_path.replace("file://", "");
            }
            // Removing extra file:/ if present (sometimes file:/C:/...)
            if file_path.starts_with('/') && file_path.chars().nth(2) == Some(':') {
                 // e.g. /C:/Users... -> C:/Users... on Windows
                 file_path = file_path[1..].to_string();
            }

            let line_number = extract_attr(chunk, "line")
                .and_then(|s| s.parse::<i32>().ok())
                .map(|l| l + 1)
                .unwrap_or(1);
            
            let description = extract_attr(chunk, "description").unwrap_or_default();
            
            let title = std::path::Path::new(&file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown File".to_string());
            
            let project = extract_project_name(&file_path);

            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM bookmarks WHERE file_path = ?1 AND line_number = ?2)",
                params![file_path, line_number],
                |row| row.get(0),
            ).unwrap_or(false);

            if !exists {
                 match conn.execute(
                    "INSERT INTO bookmarks (title, file_path, line_number, content, created_at, project) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![title, file_path, line_number, description, now, project],
                ) {
                    Ok(_) => count += 1,
                    Err(e) => println!("Failed to insert bookmark {}: {}", i, e),
                }
            } else {
                println!("Bookmark {} ({}) already exists.", i, title);
            }
        } else {
            println!("Chunk {} skipped: no url found. Content start: {}", i, &chunk.chars().take(20).collect::<String>());
        }
    }

    println!("Import finished. Imported {} bookmarks.", count);
    Ok(format!("Successfully imported {} bookmarks", count))
}
