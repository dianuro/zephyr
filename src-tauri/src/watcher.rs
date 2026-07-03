use notify::event::EventKind;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// 文件变更监听器（Tauri 托管状态，支持更新目标文件）
pub struct FileWatcher {
    stop_tx: Mutex<Option<Sender<()>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            stop_tx: Mutex::new(None),
        }
    }

    /// 停止旧的监听并开始监听新文件
    pub fn watch(&self, file_path: &str, app_handle: AppHandle) -> Result<(), String> {
        // 1. 停止旧的监听线程
        if let Ok(mut guard) = self.stop_tx.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(());
            }
        }

        let path = Path::new(file_path);
        let dir = path.parent().unwrap_or(Path::new("."));
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // 2. 创建新的停止通道
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        // 3. 创建 notify 事件通道
        let (event_tx, event_rx) = mpsc::channel::<Result<Event, notify::Error>>();

        let mut watcher: RecommendedWatcher = Watcher::new(event_tx, Config::default())
            .map_err(|e| format!("创建监听器失败: {}", e))?;

        watcher
            .watch(dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("监听目录失败: {}", e))?;

        // 4. 保存新停止信号
        if let Ok(mut guard) = self.stop_tx.lock() {
            *guard = Some(stop_tx);
        }

        let file_name_clone = file_name.clone();
        let file_path_owned = file_path.to_string();

        // 5. 启动监听线程（选择接收 stop 或 event）
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(300));

            loop {
                // 优先检查停止信号（非阻塞），再阻塞等待文件事件
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                match event_rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(Ok(event)) => {
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
                            let _ = app_handle.emit("file-changed", &file_path_owned);
                        }
                    }
                    Ok(Err(_)) => {}
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // 超时正常，继续循环以检查停止信号
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }

            drop(watcher);
        });

        Ok(())
    }
}
