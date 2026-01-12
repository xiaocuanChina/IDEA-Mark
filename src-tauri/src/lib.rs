mod db;
mod bookmark_manager;
mod preferences;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            db::init_db(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            db::get_bookmarks,
            db::add_bookmark,
            db::delete_bookmark,
            db::import_bookmarks,
            bookmark_manager::find_idea_dirs,
            bookmark_manager::list_workspace_files,
            bookmark_manager::backup_bookmark_file,
            bookmark_manager::get_backup_list,
            bookmark_manager::restore_bookmark_file,
            bookmark_manager::delete_backup_file,
            bookmark_manager::read_bookmarks_from_workspace,
            bookmark_manager::read_backup_bookmarks,
            bookmark_manager::check_idea_running,
            preferences::get_saved_idea_version,
            preferences::save_idea_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
