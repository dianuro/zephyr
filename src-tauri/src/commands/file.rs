use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::parser::markdown::{self, MarkdownResult};

/// 应用状态，包含 Markdown 解析器
pub struct AppState {
    // 未来可以在这里缓存解析器或其他资源
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_markdown: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileTree {
    pub entries: Vec<FileEntry>,
    pub current_dir: String,
}

/// 打开并渲染 Markdown 文件
#[tauri::command]
pub fn open_file(path: String, is_dark: bool) -> Result<MarkdownResult, String> {
    let resolved = resolve_path(&path);
    let content = fs::read_to_string(&resolved).map_err(|e| format!("无法读取文件: {}", e))?;
    let result = markdown::render(&content, is_dark);
    Ok(result)
}

/// 智能路径解析：处理 dev 模式下 cwd=src-tauri/ 的问题
fn resolve_path(path: &str) -> std::path::PathBuf {
    let p = std::path::Path::new(path);

    // 1. 直接 canonicalize（绝对路径 / 已存在的相对路径）
    if let Ok(abs) = p.canonicalize() {
        return abs;
    }

    // 2. 如果是相对路径，拼接 current_dir 再试
    if p.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            // 2a. cwd + path
            let candidate = cwd.join(p);
            if candidate.exists() {
                if let Ok(abs) = candidate.canonicalize() {
                    return abs;
                }
                return candidate;
            }

            // 2b. dev 模式回退：cwd=src-tauri/ 时，父目录 = 项目根目录
            if cwd.file_name().map(|n| n == "src-tauri").unwrap_or(false) {
                if let Some(parent) = cwd.parent() {
                    let candidate = parent.join(p);
                    if candidate.exists() {
                        if let Ok(abs) = candidate.canonicalize() {
                            return abs;
                        }
                        return candidate;
                    }
                }
            }
        }
    }

    // 3. 最后的回退：直接用原路径
    p.to_path_buf()
}

/// 读取目录中的文件和子目录（深度 2 层）
#[tauri::command]
pub fn open_directory(path: String) -> Result<FileTree, String> {
    let dir_path = Path::new(&path);
    if !dir_path.is_dir() {
        return Err("路径不是目录".to_string());
    }

    let mut entries = Vec::new();

    // 先添加子目录
    let mut dirs = Vec::new();
    // 再添加 markdown 文件
    let mut md_files = Vec::new();
    // 再添加其他文件
    let mut other_files = Vec::new();

    if let Ok(read_dir) = fs::read_dir(dir_path) {
        for entry in read_dir.flatten() {
            let entry_path = entry.path();
            let name = entry_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // 跳过隐藏文件和目录
            if name.starts_with('.') {
                continue;
            }

            let is_dir = entry_path.is_dir();
            let is_md = !is_dir
                && (name.ends_with(".md") || name.ends_with(".markdown"));

            let fe = FileEntry {
                name,
                path: entry_path.to_string_lossy().to_string(),
                is_dir,
                is_markdown: is_md,
            };

            if is_dir {
                dirs.push(fe);
            } else if is_md {
                md_files.push(fe);
            } else {
                other_files.push(fe);
            }
        }
    }

    // 排序：目录 > Markdown 文件 > 其他文件，各组内按名称排序
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    md_files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    other_files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    entries.extend(dirs);
    entries.extend(md_files);
    entries.extend(other_files);

    Ok(FileTree {
        entries,
        current_dir: dir_path.to_string_lossy().to_string(),
    })
}

/// 获取文件的目录树
#[tauri::command]
pub fn get_file_tree(path: String) -> Result<FileTree, String> {
    let p = Path::new(&path);
    let dir = if p.is_dir() {
        p.to_path_buf()
    } else {
        p.parent().unwrap_or(Path::new(".")).to_path_buf()
    };
    open_directory(dir.to_string_lossy().to_string())
}

/// 读取文件的原始文本内容（用于搜索等）
#[tauri::command]
pub fn read_file_content(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("无法读取文件: {}", e))
}

/// 打开原生文件选择对话框，返回所选文件路径
#[tauri::command]
pub async fn select_markdown_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app
        .dialog()
        .file()
        .add_filter("Markdown", &["md", "markdown"])
        .blocking_pick_file();
    // FilePath 是枚举，Path 变体包含 PathBuf
    Ok(file.and_then(|f| f.as_path().map(|p| p.to_string_lossy().to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_open_file_success() {
        let dir = std::env::temp_dir().join("zephyr-test-file");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("test.md");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "# Hello World").unwrap();

        let result = open_file(file_path.to_string_lossy().to_string(), false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.metadata.title, "Hello World");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_open_file_not_found() {
        let result = open_file("/nonexistent/file.md".to_string(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_open_directory() {
        let dir = std::env::temp_dir().join("zephyr-test-dir");
        let _ = fs::create_dir_all(&dir);
        fs::File::create(dir.join("readme.md")).unwrap();
        fs::File::create(dir.join("index.md")).unwrap();
        fs::File::create(dir.join("notes.txt")).unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();

        let result = open_directory(dir.to_string_lossy().to_string());
        assert!(result.is_ok());
        let tree = result.unwrap();
        let md_files: Vec<_> = tree.entries.iter().filter(|e| e.is_markdown).collect();
        assert_eq!(md_files.len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_open_directory_not_found() {
        let result = open_directory("/nonexistent/dir".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_file_tree_with_file() {
        let dir = std::env::temp_dir().join("zephyr-test-tree");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("test.md");
        fs::File::create(&file_path).unwrap();

        let result = get_file_tree(file_path.to_string_lossy().to_string());
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&dir);
    }
}
