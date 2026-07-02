use notify::event::EventKind;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct FileWatcher {
    current_file: Arc<Mutex<Option<String>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            current_file: Arc::new(Mutex::new(None)),
        }
    }

    /// 开始监听当前文件所在目录的修改
    pub fn start_watching(&self, file_path: &str, app_handle: AppHandle) -> Result<(), String> {
        let path = Path::new(file_path);
        let dir = path.parent().unwrap_or(Path::new("."));
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // 更新当前文件
        {
            let mut current = self.current_file.lock().map_err(|e| e.to_string())?;
            *current = Some(file_path.to_string());
        }

        let current_file = self.current_file.clone();

        // 使用 notify 的事件通道
        let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, Config::default()).map_err(|e| format!("创建监听器失败: {}", e))?;

        watcher
            .watch(dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("监听目录失败: {}", e))?;

        let file_name_clone = file_name.clone();

        // 将 watcher 移入线程，保持其存活
        thread::spawn(move || {
            // 给文件系统一些时间来稳定
            thread::sleep(Duration::from_millis(300));

            for event in rx {
                match event {
                    Ok(event) => {
                        let is_modify = matches!(
                            event.kind,
                            EventKind::Modify(_) | EventKind::Create(_)
                        );

                        if !is_modify {
                            continue;
                        }

                        let is_target = event.paths.iter().any(|p| {
                            p.file_name()
                                .and_then(|n| n.to_str())
                                .map(|n| n == file_name_clone)
                                .unwrap_or(false)
                        });

                        if is_target {
                            let path = current_file.lock().ok().and_then(|f| f.clone());
                            if let Some(p) = path {
                                let _ = app_handle.emit("file-changed", p);
                            }
                        }
                    }
                    Err(_) => {}
                }
            }

            // watcher 在此被 drop，但线程会一直运行到 rx 关闭
            drop(watcher);
        });

        Ok(())
    }
}
