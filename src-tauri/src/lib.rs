mod commands;
mod parser;
mod theme_config;
pub(crate) mod watcher;

use crate::watcher::FileWatcher;
use serde_json::json;
use std::sync::Mutex;
use tauri::Manager;

/// 缓存主题配置（启动时加载，运行时只读）
struct ConfigCache {
    light: Mutex<theme_config::ThemeConfig>,
    dark: Mutex<theme_config::ThemeConfig>,
}

/// 获取主题配置（前端调用）
#[tauri::command]
fn get_theme_config(state: tauri::State<ConfigCache>, is_dark: bool) -> serde_json::Value {
    let config = if is_dark {
        state.dark.lock().unwrap().clone()
    } else {
        state.light.lock().unwrap().clone()
    };
    json!(config)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 启动时加载配置文件（若不存在则生成默认配置）
    let (light, dark) = theme_config::load_configs();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ConfigCache {
            light: Mutex::new(light),
            dark: Mutex::new(dark),
        })
        .manage(FileWatcher::new())
        .setup(|app| {
            // 处理 CLI 参数：传递原始路径给前端（open_file 内部会做智能解析）
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let raw_path = &args[1];
                if let Some(window) = app.get_webview_window("main") {
                    let escaped_path = raw_path.replace('\\', "\\\\").replace('\'', "\\'");
                    let _ = window.eval(format!(
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
            commands::file::select_markdown_file,
            get_theme_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
