mod commands;
mod parser;
mod watcher;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 处理 CLI 参数：如果提供了文件路径，在页面加载后打开
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let path = &args[1];
                if let Some(window) = app.get_webview_window("main") {
                    // 通过 eval 将路径传递给前端
                    let escaped_path = path.replace('\\', "\\\\").replace('\'', "\\'");
                    let _ = window.eval(&format!(
                        "setTimeout(() => window.__ZEPHYR_CLI_FILE__ = '{}', 100);",
                        escaped_path
                    ));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::file::open_file,
            commands::file::open_directory,
            commands::file::get_file_tree,
            commands::file::read_file_content,
            commands::parse::render_markdown,
            commands::search::search_in_document,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
