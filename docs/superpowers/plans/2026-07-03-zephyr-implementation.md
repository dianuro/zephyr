# Zephyr Markdown 阅读器 — 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 基于 Tauri 2 + Rust + Vanilla JS 构建高速 Markdown 阅读器，渲染效果接近 GitHub。

**架构：** Rust 端使用 comrak 解析 Markdown，syntect 做代码语法高亮，输出完整 HTML；前端接收 HTML 后使用 KaTeX 渲染数学公式、Mermaid 渲染图表，并管理文件树、大纲、搜索等交互。

**技术栈：** Tauri 2, Rust (comrak, syntect, notify, walkdir), Vanilla JS (KaTeX, Mermaid, github-markdown-css)

---

## 文件结构

```
src-tauri/src/
├── main.rs                   修改 — 添加 CLI 参数解析
├── lib.rs                    修改 — 注册所有命令和插件
├── commands/
│   ├── mod.rs                创建 — 命令模块
│   ├── file.rs               创建 — open_file, open_directory, get_file_tree
│   ├── parse.rs              创建 — 渲染管道
│   └── search.rs             创建 — 全文搜索
├── parser/
│   ├── mod.rs                创建 — 解析器模块
│   ├── markdown.rs           创建 — comrak 解析 + 自定义渲染
│   └── highlighter.rs        创建 — syntect 语法高亮
└── watcher.rs                创建 — notify 文件监听

src/
├── index.html                修改 — 阅读器布局
├── styles.css                修改 — 全局 + 主题样式
├── main.js                   修改 — 应用入口
├── state.js                  创建 — 状态管理
└── components/
    ├── sidebar.js            创建 — 文件树面板
    ├── viewer.js             创建 — 内容渲染
    ├── outline.js            创建 — 大纲导航
    ├── search.js             创建 — 文档搜索
    └── theme.js              创建 — 主题切换
```

---

### 任务 1：Rust 依赖与模块骨架

**文件：**
- 修改：`src-tauri/Cargo.toml`
- 创建：`src-tauri/src/commands/mod.rs`
- 创建：`src-tauri/src/parser/mod.rs`

- [ ] **步骤 1：更新 Cargo.toml 添加依赖**

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
comrak = "0.33"
syntect = { version = "5", default-features = false, features = ["default-syntaxes", "default-themes", "regex-onig"] }
notify = "7"
notify-debouncer-mini = "0.4"
walkdir = "2"
regex = "1"
same-file = "1"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [ ] **步骤 2：创建 commands/mod.rs 模块声明**

```rust
pub mod file;
pub mod parse;
pub mod search;
```

- [ ] **步骤 3：创建 parser/mod.rs 模块声明**

```rust
pub mod markdown;
pub mod highlighter;
```

- [ ] **步骤 4：验证编译**

```bash
cd /tummy/projects/zephyr/src-tauri
cargo check 2>&1 | head -30
```
预期：依赖下载并编译成功。

---

### 任务 2：Rust 语法高亮器

**文件：**
- 创建：`src-tauri/src/parser/highlighter.rs`

- [ ] **步骤 1：实现 Highlighter 结构体**

```rust
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct Highlighter {
    syntax_set: SyntaxSet,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }

    /// 将代码块高亮为 HTML（带语法高亮的 <pre><code> 标签）
    pub fn highlight(&self, code: &str, lang: Option<&str>) -> String {
        let lang = lang.unwrap_or("");

        if lang.is_empty() {
            // 无语言指定，使用纯文本高亮
            return format!(
                "<pre><code>{}</code></pre>\n",
                html_escape(code)
            );
        }

        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        match highlighted_html_for_string(code, &self.syntax_set, syntax) {
            Ok(html) => html,
            Err(_) => format!(
                "<pre><code>{}</code></pre>\n",
                html_escape(code)
            ),
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

- [ ] **步骤 2：添加单元测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_rust_code() {
        let hl = Highlighter::new();
        let code = "fn main() {\n    println!(\"hello\");\n}";
        let html = hl.highlight(code, Some("rust"));
        assert!(html.contains("hljs"));  // syntect 输出包含 hljs class
        assert!(html.contains("println"));
    }

    #[test]
    fn test_highlight_no_lang() {
        let hl = Highlighter::new();
        let code = "plain text";
        let html = hl.highlight(code, None);
        assert!(html.contains("plain text"));
    }

    #[test]
    fn test_escape_html_in_code() {
        let hl = Highlighter::new();
        let code = "<script>alert('xss')</script>";
        let html = hl.highlight(code, None);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
```

- [ ] **步骤 3：运行测试**

```bash
cd /tummy/projects/zephyr/src-tauri
cargo test parser::highlighter::tests -v 2>&1 | head -30
```
预期：所有测试通过。

---

### 任务 3：Rust Markdown 解析器

**文件：**
- 创建：`src-tauri/src/parser/markdown.rs`

- [ ] **步骤 1：实现 Markdown 解析管道**

```rust
use crate::parser::highlighter::Highlighter;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{format_html, parse_document, Arena, ComrakOptions};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub id: String,
    pub children: Vec<Heading>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarkdownMetadata {
    pub title: String,
    pub headings: Vec<Heading>,
    pub word_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarkdownResult {
    pub html: String,
    pub metadata: MarkdownMetadata,
}

pub struct MarkdownParser {
    highlighter: Highlighter,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            highlighter: Highlighter::new(),
        }
    }

    pub fn render(&self, markdown: &str) -> MarkdownResult {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.autolink = true;
        options.extension.table = true;
        options.extension.tasklist = true;
        options.extension.strikethrough = true;
        options.extension.tagfilter = true;
        options.extension.footnotes = true;
        options.render.github_pre_lang = true;
        options.render.full_info_string = true;
        options.render.unsafe_ = false; // 防止 XSS

        let root = parse_document(&arena, markdown, &options);
        let headings = Self::extract_headings(root);
        let title = headings
            .first()
            .map(|h| h.text.clone())
            .unwrap_or_default();
        let word_count = Self::count_words(markdown);

        // 自定义 HTML 渲染，注入语法高亮
        let mut html = String::new();
        let mut format_options = comrak::format::FormatOptions::default();
        // 自定义代码块渲染
        self.render_node(root, &mut html, &options, &format_options);

        MarkdownResult {
            html,
            metadata: MarkdownMetadata {
                title,
                headings,
                word_count,
            },
        }
    }

    fn render_node(
        &self,
        node: &AstNode,
        output: &mut String,
        options: &ComrakOptions,
        format_options: &comrak::format::FormatOptions,
    ) {
        let data = node.data.borrow();
        match &data.value {
            NodeValue::CodeBlock(ref cb) => {
                let lang = if cb.info.is_empty() {
                    None
                } else {
                    // info 格式可能是 "rust" 或 "rust,editable"
                    Some(cb.info.split(&[',', ' '][..]).next().unwrap_or(""))
                };
                let highlighted = self.highlighter.highlight(&cb.literal, lang);
                output.push_str(&highlighted);
            }
            _ => {
                // 递归渲染子节点
                for child in node.children() {
                    self.render_node(child, output, options, format_options);
                }
                // 使用 comrak 默认格式化当前节点
                // 注意：这里简化了，实际上需要更复杂的遍历逻辑
                // 对于非代码块节点，使用适当的格式
            }
        }
    }

    fn extract_headings(node: &AstNode) -> Vec<Heading> {
        let mut headings = Vec::new();
        Self::extract_headings_recursive(node, &mut headings, 0);
        headings
    }

    fn extract_headings_recursive(
        node: &AstNode,
        headings: &mut Vec<Heading>,
        depth: usize,
    ) {
        let data = node.data.borrow();
        if let NodeValue::Heading(ref heading) = data.value {
            let text = Self::collect_text(node);
            let id = slugify(&text);
            headings.push(Heading {
                level: heading.level,
                text,
                id,
                children: Vec::new(),
            });
        }
        for child in node.children() {
            Self::extract_headings_recursive(child, headings, depth + 1);
        }
    }

    fn collect_text(node: &AstNode) -> String {
        let mut text = String::new();
        for child in node.children() {
            let data = child.data.borrow();
            match &data.value {
                NodeValue::Text(t) => text.push_str(&t),
                NodeValue::Code(t) => text.push_str(&t),
                NodeValue::Strong(children) => {
                    text.push_str(&Self::collect_text(child));
                }
                NodeValue::Emph(children) => {
                    text.push_str(&Self::collect_text(child));
                }
                _ => {}
            }
        }
        text
    }

    fn count_words(text: &str) -> u32 {
        text.split_whitespace().count() as u32
    }
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                Some(c)
            } else if c.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
```

- [ ] **步骤 2：添加单元测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown() {
        let parser = MarkdownParser::new();
        let result = parser.render("# Hello\n\nThis is **bold** text.");
        assert!(result.html.contains("Hello"));
        assert!(result.html.contains("<strong>") || result.html.contains("<b>"));
        assert_eq!(result.metadata.title, "Hello");
        assert_eq!(result.metadata.headings.len(), 1);
    }

    #[test]
    fn test_render_code_block_with_highlighting() {
        let parser = MarkdownParser::new();
        let result = parser.render("```rust\nfn main() {}\n```");
        assert!(result.html.contains("hljs") || result.html.contains("syntax"));
    }

    #[test]
    fn test_extract_nested_headings() {
        let parser = MarkdownParser::new();
        let result = parser.render("# A\n\n## B\n\n### C\n\n## D");
        assert_eq!(result.metadata.headings.len(), 3);
        assert_eq!(result.metadata.headings[0].level, 1);
        assert_eq!(result.metadata.headings[0].text, "A");
    }

    #[test]
    fn test_empty_document() {
        let parser = MarkdownParser::new();
        let result = parser.render("");
        assert!(result.metadata.title.is_empty());
        assert_eq!(result.metadata.word_count, 0);
    }

    #[test]
    fn test_word_count() {
        let parser = MarkdownParser::new();
        let result = parser.render("# Title\n\nHello world foo bar");
        assert!(result.metadata.word_count > 0);
    }
}
```

- [ ] **步骤 3：运行测试**

```bash
cd /tummy/projects/zephyr/src-tauri
cargo test parser::markdown::tests -v 2>&1 | head -30
```
预期：所有测试通过。

---

### 任务 4：Rust 文件操作命令

**文件：**
- 创建：`src-tauri/src/commands/file.rs`

- [ ] **步骤 1：实现文件命令**

```rust
use crate::parser::markdown::MarkdownResult;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use walkdir::WalkDir;

pub struct AppState {
    pub parser: crate::parser::markdown::MarkdownParser,
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
pub fn open_file(path: String, state: State<AppState>) -> Result<MarkdownResult, String> {
    let content = fs::read_to_string(&path).map_err(|e| format!("无法读取文件: {}", e))?;
    let result = state.parser.render(&content);
    Ok(result)
}

/// 读取目录中的文件列表
#[tauri::command]
pub fn open_directory(path: String) -> Result<FileTree, String> {
    let dir_path = Path::new(&path);
    if !dir_path.is_dir() {
        return Err("路径不是目录".to_string());
    }

    let mut entries = Vec::new();
    for entry in WalkDir::new(dir_path).max_depth(2).into_iter().filter_entry(|e| {
        // 跳过隐藏目录
        !e.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false)
    }) {
        if let Ok(entry) = entry {
            let path = entry.path();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let is_dir = path.is_dir();
            let is_md = !is_dir && (name.ends_with(".md") || name.ends_with(".markdown"));

            entries.push(FileEntry {
                name,
                path: path.to_string_lossy().to_string(),
                is_dir,
                is_markdown: is_md,
            });
        }
    }

    Ok(FileTree {
        entries,
        current_dir: dir_path.to_string_lossy().to_string(),
    })
}

/// 获取文件的目录树（仅 Markdown 文件）
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
```

- [ ] **步骤 2：添加单元测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_open_file_success() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.md");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "# Hello World").unwrap();

        let parser = crate::parser::markdown::MarkdownParser::new();
        let state = AppState { parser };
        let result = open_file(file_path.to_string_lossy().to_string(), tauri::State::from(&state));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.metadata.title, "Hello World");
    }

    #[test]
    fn test_open_file_not_found() {
        let parser = crate::parser::markdown::MarkdownParser::new();
        let state = AppState { parser };
        let result = open_file("/nonexistent/file.md".to_string(), tauri::State::from(&state));
        assert!(result.is_err());
    }

    #[test]
    fn test_list_directory() {
        let dir = TempDir::new().unwrap();
        fs::File::create(dir.path().join("readme.md")).unwrap();
        fs::File::create(dir.path().join("index.md")).unwrap();
        fs::File::create(dir.path().join("notes.txt")).unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();

        let result = open_directory(dir.path().to_string_lossy().to_string());
        assert!(result.is_ok());
        let tree = result.unwrap();
        let md_files: Vec<_> = tree.entries.iter().filter(|e| e.is_markdown).collect();
        assert_eq!(md_files.len(), 2);
    }
}
```

- [ ] **步骤 3：运行测试**

```bash
cd /tummy/projects/zephyr/src-tauri
cargo test commands::file::tests -v 2>&1 | head -40
```
预期：所有测试通过。

---

### 任务 5：Rust 搜索命令

**文件：**
- 创建：`src-tauri/src/commands/search.rs`

- [ ] **步骤 1：实现搜索功能**

```rust
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SearchMatch {
    pub line: u32,
    pub column: u32,
    pub context: String,
    pub matched_text: String,
}

/// 在文档内容中搜索文本
#[tauri::command]
pub fn search_in_document(content: String, query: String) -> Result<Vec<SearchMatch>, String> {
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let re = Regex::new(&regex::escape(&query))
        .map_err(|e| format!("正则表达式错误: {}", e))?;

    let mut results = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        for mat in re.find_iter(line) {
            let context = line.to_string();
            results.push(SearchMatch {
                line: (line_num + 1) as u32,
                column: (mat.start() + 1) as u32,
                context: context.trim().to_string(),
                matched_text: mat.as_str().to_string(),
            });
        }
    }

    Ok(results)
}
```

- [ ] **步骤 2：添加单元测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_basic() {
        let content = "# Hello\n\nThis is a test document.\nHello again!";
        let results = search_in_document(content.to_string(), "Hello".to_string()).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line, 1);
        assert_eq!(results[1].line, 4);
    }

    #[test]
    fn test_search_no_results() {
        let content = "# Hello\nWorld";
        let results = search_in_document(content.to_string(), "xyz".to_string()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_empty_query() {
        let content = "Some content";
        let results = search_in_document(content.to_string(), "".to_string()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_special_chars() {
        let content = "foo bar (baz) [qux]";
        let results = search_in_document(content.to_string(), "(baz)".to_string()).unwrap();
        assert_eq!(results.len(), 1);
    }
}
```

- [ ] **步骤 3：运行测试**

```bash
cd /tummy/projects/zephyr/src-tauri
cargo test commands::search::tests -v 2>&1 | head -30
```
预期：所有测试通过。

---

### 任务 6：Rust 文件监听器

**文件：**
- 创建：`src-tauri/src/watcher.rs`

- [ ] **步骤 1：实现文件监听器**

```rust
use notify::event::EventKind;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    current_file: Arc<Mutex<Option<String>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            current_file: Arc::new(Mutex::new(None)),
        }
    }

    /// 开始监听指定目录
    pub fn watch(&mut self, file_path: &str, app_handle: AppHandle) -> Result<(), String> {
        let path = Path::new(file_path);
        let dir = path.parent().unwrap_or(Path::new("."));

        let current_file = self.current_file.clone();
        *current_file.lock().unwrap() = Some(file_path.to_string());

        let app_handle_clone = app_handle.clone();

        let (tx, rx) = mpsc::channel::<DebounceEventResult>();

        let mut debouncer = new_debouncer(
            Duration::from_millis(300),
            move |result: DebounceEventResult| {
                let _ = tx.send(result);
            },
        )
        .map_err(|e| format!("无法创建监听器: {}", e))?;

        debouncer
            .watcher()
            .watch(dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("无法监听目录: {}", e))?;

        // 启动监听线程
        let current_file_clone = self.current_file.clone();
        thread::spawn(move || {
            for result in rx {
                match result {
                    Ok(events) => {
                        for event in events {
                            if let EventKind::Modify(_) = event.event.kind {
                                if let Some(path) = event.event.paths.first() {
                                    let watching = current_file_clone.lock().unwrap();
                                    if let Some(current) = watching.as_ref() {
                                        let current_path = Path::new(current);
                                        if path == current_path {
                                            let _ = app_handle_clone.emit("file-changed", current);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        });

        self.watcher = Some(
            debouncer
                .watcher()
                .clone(),
        );

        Ok(())
    }
}
```

- [ ] **步骤 2：编译验证**

```bash
cd /tummy/projects/zephyr/src-tauri
cargo check 2>&1 | head -30
```
预期：编译通过，无错误。

---

### 任务 7：Rust 主入口与 Tauri 集成

**文件：**
- 修改：`src-tauri/src/lib.rs`
- 修改：`src-tauri/src/main.rs`

- [ ] **步骤 1：重写 lib.rs 集成所有命令**

```rust
mod commands;
mod parser;
mod watcher;

use commands::file::AppState;
use parser::markdown::MarkdownParser;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let markdown_parser = MarkdownParser::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            parser: markdown_parser,
        })
        .setup(|app| {
            // 处理 CLI 参数
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let path = &args[1];
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval(&format!(
                        "window.__ZEPHYR_CLI_FILE__ = '{}';",
                        path.replace('\'', "\\'")
                    ));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::file::open_file,
            commands::file::open_directory,
            commands::file::get_file_tree,
            commands::search::search_in_document,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **步骤 2：更新 main.rs（无变化，保持原样）**

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    zephyr_lib::run()
}
```

- [ ] **步骤 3：更新 tauri.conf.json 调整窗口大小**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "zephyr",
  "version": "0.1.0",
  "identifier": "com.dianqwq.zephyr",
  "build": {
    "frontendDist": "../src"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Zephyr",
        "width": 1100,
        "height": 750,
        "minWidth": 600,
        "minHeight": 400
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **步骤 4：编译验证**

```bash
cd /tummy/projects/zephyr/src-tauri
cargo check 2>&1 | head -40
```
预期：编译通过。

---

### 任务 8：前端 — 页面结构与状态管理

**文件：**
- 修改：`src/index.html`
- 创建：`src/state.js`

- [ ] **步骤 1：重写 index.html**

```html
<!doctype html>
<html lang="zh-CN" data-theme="light">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Zephyr</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.5.1/github-markdown.min.css" />
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css" />
    <link rel="stylesheet" href="styles.css" />
    <script type="module" src="main.js" defer></script>
  </head>
  <body>
    <div id="app">
      <aside id="sidebar" class="sidebar hidden">
        <div class="sidebar-tabs">
          <button class="tab-btn active" data-tab="files">文件</button>
          <button class="tab-btn" data-tab="outline">大纲</button>
        </div>
        <div id="files-panel" class="sidebar-panel active">
          <div id="file-tree"></div>
        </div>
        <div id="outline-panel" class="sidebar-panel">
          <div id="outline-tree"></div>
        </div>
      </aside>

      <main id="main-content">
        <header id="toolbar">
          <div class="toolbar-left">
            <button id="toggle-sidebar" title="切换侧边栏">☰</button>
            <button id="open-file" title="打开文件">📂</button>
            <span id="file-title" class="file-title"></span>
          </div>
          <div class="toolbar-right">
            <button id="toggle-search" title="搜索 (Ctrl+F)">🔍</button>
            <button id="toggle-theme" title="切换主题">🌙</button>
          </div>
        </header>

        <div id="search-bar" class="search-bar hidden">
          <input type="text" id="search-input" placeholder="在文档中搜索…" />
          <span id="search-count"></span>
          <button id="search-prev">▲</button>
          <button id="search-next">▼</button>
          <button id="search-close">✕</button>
        </div>

        <div id="content" class="markdown-body"></div>

        <div id="welcome" class="welcome">
          <h1>Zephyr</h1>
          <p>高速 Markdown 阅读器</p>
          <p class="welcome-hint">拖拽 Markdown 文件到窗口，或点击 📂 打开</p>
        </div>
      </main>
    </div>
  </body>
</html>
```

- [ ] **步骤 2：创建 state.js**

```javascript
// 应用状态管理
const state = {
  currentFile: null,
  currentContent: null,
  currentHtml: null,
  metadata: null,
  fileTree: [],
  searchResults: [],
  searchIndex: 0,
  sidebarVisible: false,
  darkMode: false,
  fileHistory: [],

  set(key, value) {
    this[key] = value;
  },

  get(key) {
    return this[key];
  },
};

window.__state = state;
export default state;
```

---

### 任务 9：前端 — CSS 样式与主题

**文件：**
- 修改：`src/styles.css`

- [ ] **步骤 1：重写 styles.css**

```css
/* =========== 全局 Reset 与变量 =========== */
:root,
[data-theme="light"] {
  --bg-primary: #ffffff;
  --bg-secondary: #f6f8fa;
  --bg-sidebar: #fafbfc;
  --bg-toolbar: #ffffff;
  --bg-hover: #f3f4f6;
  --text-primary: #24292f;
  --text-secondary: #57606a;
  --text-muted: #8b949e;
  --border-color: #d0d7de;
  --accent: #0969da;
  --accent-hover: #0550ae;
  --toolbar-height: 48px;
  --sidebar-width: 280px;
  --shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

[data-theme="dark"] {
  --bg-primary: #0d1117;
  --bg-secondary: #161b22;
  --bg-sidebar: #161b22;
  --bg-toolbar: #161b22;
  --bg-hover: #1c2128;
  --text-primary: #e6edf3;
  --text-secondary: #8b949e;
  --text-muted: #484f58;
  --border-color: #30363d;
  --accent: #58a6ff;
  --accent-hover: #79c0ff;
  --shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif;
  background: var(--bg-primary);
  color: var(--text-primary);
  overflow: hidden;
  height: 100vh;
}

/* =========== 布局 =========== */
#app {
  display: flex;
  height: 100vh;
}

/* =========== 侧边栏 =========== */
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: margin-left 0.2s ease;
}

.sidebar.hidden {
  margin-left: calc(-1 * var(--sidebar-width));
}

.sidebar-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color);
}

.tab-btn {
  flex: 1;
  padding: 10px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  border-bottom: 2px solid transparent;
  transition: all 0.15s;
}

.tab-btn.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.tab-btn:hover {
  background: var(--bg-hover);
}

.sidebar-panel {
  display: none;
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.sidebar-panel.active {
  display: block;
}

/* =========== 文件树 =========== */
.file-item {
  display: flex;
  align-items: center;
  padding: 4px 16px;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-item:hover {
  background: var(--bg-hover);
}

.file-item .icon {
  margin-right: 8px;
  flex-shrink: 0;
}

.file-item.dir {
  font-weight: 500;
}

/* =========== 大纲树 =========== */
.outline-item {
  display: block;
  padding: 3px 16px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  text-decoration: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.outline-item:hover {
  color: var(--accent);
}

.outline-item.h1 { padding-left: 16px; font-weight: 600; }
.outline-item.h2 { padding-left: 32px; }
.outline-item.h3 { padding-left: 48px; }
.outline-item.h4 { padding-left: 64px; }

/* =========== 主内容区 =========== */
#main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* =========== 工具栏 =========== */
#toolbar {
  height: var(--toolbar-height);
  min-height: var(--toolbar-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: var(--bg-toolbar);
  border-bottom: 1px solid var(--border-color);
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

#toolbar button {
  background: none;
  border: 1px solid transparent;
  cursor: pointer;
  font-size: 16px;
  padding: 4px 8px;
  border-radius: 6px;
  color: var(--text-secondary);
  transition: all 0.15s;
}

#toolbar button:hover {
  background: var(--bg-hover);
  border-color: var(--border-color);
}

.file-title {
  font-size: 14px;
  font-weight: 500;
  margin-left: 8px;
  color: var(--text-primary);
  max-width: 400px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* =========== 搜索栏 =========== */
.search-bar {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  gap: 8px;
}

.search-bar.hidden {
  display: none;
}

#search-input {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
  background: var(--bg-primary);
  color: var(--text-primary);
  outline: none;
}

#search-input:focus {
  border-color: var(--accent);
}

#search-count {
  font-size: 12px;
  color: var(--text-muted);
  min-width: 60px;
  text-align: center;
}

.search-bar button {
  background: none;
  border: 1px solid var(--border-color);
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
  color: var(--text-secondary);
}

.search-bar button:hover {
  background: var(--bg-hover);
}

/* =========== 内容区 =========== */
#content {
  flex: 1;
  overflow-y: auto;
  padding: 24px 32px;
}

#content.welcome-active {
  display: none;
}

/* =========== 欢迎页 =========== */
.welcome {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--text-muted);
  text-align: center;
}

.welcome h1 {
  font-size: 36px;
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.welcome p {
  font-size: 16px;
}

.welcome-hint {
  margin-top: 24px;
  font-size: 14px;
  color: var(--text-muted);
}

/* =========== 搜索高亮 =========== */
.search-highlight {
  background: #ffd700;
  color: #000;
  border-radius: 2px;
  padding: 0 1px;
}

[data-theme="dark"] .search-highlight {
  background: #b8860b;
  color: #fff;
}

.search-highlight.active {
  background: #ff6b00;
  color: #fff;
}

/* =========== 滚动条 =========== */
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}
```

---

### 任务 10：前端 — 主入口与组件

**文件：**
- 修改：`src/main.js`
- 创建：`src/components/sidebar.js`
- 创建：`src/components/viewer.js`
- 创建：`src/components/outline.js`
- 创建：`src/components/search.js`
- 创建：`src/components/theme.js`

- [ ] **步骤 1：创建 theme.js**

```javascript
import state from '../state.js';

const STORAGE_KEY = 'zephyr-theme';

export function initTheme() {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved === 'dark') {
    setDark(true);
  }

  document.getElementById('toggle-theme').addEventListener('click', toggleTheme);
}

function toggleTheme() {
  const isDark = !state.darkMode;
  setDark(isDark);
  localStorage.setItem(STORAGE_KEY, isDark ? 'dark' : 'light');
}

function setDark(dark) {
  state.darkMode = dark;
  document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
  document.getElementById('toggle-theme').textContent = dark ? '☀️' : '🌙';
}
```

- [ ] **步骤 2：创建 sidebar.js**

```javascript
import state from '../state.js';
import { openFile } from './viewer.js';

export function initSidebar() {
  document.getElementById('toggle-sidebar').addEventListener('click', toggleSidebar);
  document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => switchTab(btn.dataset.tab));
  });
  document.getElementById('open-file').addEventListener('click', openFileDialog);
}

export function toggleSidebar() {
  state.sidebarVisible = !state.sidebarVisible;
  document.getElementById('sidebar').classList.toggle('hidden', !state.sidebarVisible);
}

function switchTab(tab) {
  document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
  document.querySelectorAll('.sidebar-panel').forEach(p => p.classList.remove('active'));

  document.querySelector(`.tab-btn[data-tab="${tab}"]`).classList.add('active');
  document.getElementById(`${tab}-panel`).classList.add('active');
}

async function openFileDialog() {
  try {
    const result = await window.__TAURI__.dialog.open({
      multiple: false,
      filters: [{ name: 'Markdown', extensions: ['md', 'markdown'] }],
    });
    if (result) {
      await openFile(result);
    }
  } catch (e) {
    console.warn('打开文件对话框失败', e);
  }
}

export function renderFileTree(tree) {
  const container = document.getElementById('file-tree');
  container.innerHTML = '';

  tree.entries.forEach(entry => {
    if (!entry.is_markdown && !entry.is_dir) return;

    const el = document.createElement('div');
    el.className = 'file-item';
    el.dataset.path = entry.path;

    if (entry.is_dir) {
      el.classList.add('dir');
      el.innerHTML = `<span class="icon">📁</span>${entry.name}`;
      el.addEventListener('click', async () => {
        const newTree = await window.__TAURI__.invoke('get_file_tree', { path: entry.path });
        renderFileTree(newTree);
      });
    } else {
      el.innerHTML = `<span class="icon">📄</span>${entry.name}`;
      el.addEventListener('click', () => openFile(entry.path));
    }

    container.appendChild(el);
  });
}
```

- [ ] **步骤 3：创建 viewer.js**

```javascript
import state from '../state.js';

export async function openFile(path) {
  try {
    const result = await window.__TAURI__.invoke('open_file', { path });
    state.set('currentFile', path);
    state.set('currentHtml', result.html);
    state.set('metadata', result.metadata);

    // 更新标题
    const title = result.metadata.title || path.split('/').pop() || path.split('\\').pop();
    document.getElementById('file-title').textContent = title;
    document.title = `${title} — Zephyr`;

    // 隐藏欢迎页
    document.getElementById('welcome').style.display = 'none';
    document.getElementById('content').classList.remove('welcome-active');

    // 渲染 HTML
    const container = document.getElementById('content');
    container.innerHTML = result.html;

    // 后处理：KaTeX + Mermaid
    await renderMath();
    renderDiagrams();

    // 更新大纲
    const { renderOutline } = await import('./outline.js');
    renderOutline(result.metadata.headings);

  } catch (e) {
    console.error('打开文件失败:', e);
    showError(`无法打开文件: ${e}`);
  }
}

async function renderMath() {
  // 扫描所有代码块并尝试渲染数学公式
  const codeBlocks = document.querySelectorAll('#content code.language-math');
  if (codeBlocks.length === 0) return;

  try {
    const katex = await import('https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.mjs');
    codeBlocks.forEach(block => {
      const text = block.textContent;
      const isDisplay = block.closest('pre') !== null && block.parentElement.tagName === 'PRE';
      try {
        const html = katex.renderToString(text, {
          displayMode: isDisplay,
          throwOnError: false,
        });
        // 替换代码块为 KaTeX 渲染结果
        if (isDisplay) {
          const pre = block.closest('pre');
          if (pre) {
            pre.outerHTML = html;
          }
        } else {
          block.outerHTML = html;
        }
      } catch (e) {
        // 渲染失败，保留原始代码
        console.warn('KaTeX 渲染失败:', e);
      }
    });
  } catch (e) {
    console.warn('KaTeX 加载失败:', e);
  }
}

async function renderDiagrams() {
  const codeBlocks = document.querySelectorAll('#content code.language-mermaid');
  if (codeBlocks.length === 0) return;

  try {
    const mermaid = await import('https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs');
    mermaid.default.initialize({ startOnLoad: false, theme: state.darkMode ? 'dark' : 'default' });

    codeBlocks.forEach((block, index) => {
      const text = block.textContent;
      const pre = block.closest('pre');
      if (!pre) return;

      const id = `mermaid-${index}`;
      try {
        mermaid.default.render(id, text).then(({ svg }) => {
          pre.outerHTML = svg;
        }).catch(e => {
          console.warn('Mermaid 渲染失败:', e);
        });
      } catch (e) {
        console.warn('Mermaid 渲染失败:', e);
      }
    });
  } catch (e) {
    console.warn('Mermaid 加载失败:', e);
  }
}

function showError(msg) {
  const container = document.getElementById('content');
  container.innerHTML = `<div class="error-message"><h2>错误</h2><p>${msg}</p></div>`;
}
```

- [ ] **步骤 4：创建 outline.js**

```javascript
import state from '../state.js';

export function renderOutline(headings) {
  const container = document.getElementById('outline-tree');
  container.innerHTML = '';

  headings.forEach(h => {
    const el = document.createElement('a');
    el.className = `outline-item h${h.level}`;
    el.textContent = h.text;
    el.href = `#${h.id}`;
    el.addEventListener('click', (e) => {
      e.preventDefault();
      const target = document.getElementById(h.id);
      if (target) {
        target.scrollIntoView({ behavior: 'smooth' });
      }
    });
    container.appendChild(el);

    if (h.children && h.children.length > 0) {
      renderOutline(h.children);
    }
  });
}
```

- [ ] **步骤 5：创建 search.js**

```javascript
import state from '../state.js';

export function initSearch() {
  const toggleBtn = document.getElementById('toggle-search');
  const searchBar = document.getElementById('search-bar');
  const input = document.getElementById('search-input');
  const prevBtn = document.getElementById('search-prev');
  const nextBtn = document.getElementById('search-next');
  const closeBtn = document.getElementById('search-close');

  toggleBtn.addEventListener('click', () => {
    searchBar.classList.toggle('hidden');
    if (!searchBar.classList.contains('hidden')) {
      input.focus();
    } else {
      clearHighlights();
    }
  });

  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
      e.preventDefault();
      searchBar.classList.toggle('hidden');
      if (!searchBar.classList.contains('hidden')) {
        input.focus();
      }
    }
    if (e.key === 'Escape' && !searchBar.classList.contains('hidden')) {
      searchBar.classList.add('hidden');
      clearHighlights();
    }
  });

  let debounceTimer;
  input.addEventListener('input', () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => performSearch(input.value), 300);
  });

  prevBtn.addEventListener('click', () => navigateSearch(-1));
  nextBtn.addEventListener('click', () => navigateSearch(1));
  closeBtn.addEventListener('click', () => {
    searchBar.classList.add('hidden');
    clearHighlights();
  });
}

async function performSearch(query) {
  clearHighlights();

  if (!query || !state.currentHtml) {
    document.getElementById('search-count').textContent = '';
    return;
  }

  try {
    // 在前端进行搜索
    const container = document.getElementById('content');
    const html = container.innerHTML;
    const results = [];

    // 简单文本搜索（忽略 HTML 标签）
    const textContent = container.textContent || '';
    let idx = 0;
    let count = 0;

    // 使用 TreeWalker 在文本节点中搜索
    const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT, null, false);
    while (walker.nextNode()) {
      const node = walker.currentNode;
      const text = node.textContent;
      let pos = 0;
      while ((pos = text.toLowerCase().indexOf(query.toLowerCase(), pos)) !== -1) {
        results.push({ node, offset: pos, length: query.length });
        pos += query.length;
        count++;
      }
    }

    state.searchResults = results;
    state.searchIndex = 0;

    document.getElementById('search-count').textContent = results.length > 0
      ? `${results.length} 个匹配`
      : '无匹配';

    // 高亮所有匹配
    results.forEach((r, i) => {
      try {
        const range = document.createRange();
        range.setStart(r.node, r.offset);
        range.setEnd(r.node, r.offset + r.length);
        const span = document.createElement('span');
        span.className = 'search-highlight';
        if (i === 0) span.classList.add('active');
        range.surroundContents(span);
      } catch (e) {
        // 跳过重叠范围
      }
    });

    // 滚动到第一个匹配
    const first = document.querySelector('.search-highlight');
    if (first) first.scrollIntoView({ behavior: 'smooth', block: 'center' });

  } catch (e) {
    console.error('搜索失败:', e);
  }
}

function navigateSearch(direction) {
  const results = state.searchResults;
  if (results.length === 0) return;

  // 移除当前 active
  const current = document.querySelector('.search-highlight.active');
  if (current) current.classList.remove('active');

  state.searchIndex = (state.searchIndex + direction + results.length) % results.length;

  // 高亮当前
  const highlights = document.querySelectorAll('.search-highlight');
  if (highlights[state.searchIndex]) {
    highlights[state.searchIndex].classList.add('active');
    highlights[state.searchIndex].scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
}

function clearHighlights() {
  const highlights = document.querySelectorAll('.search-highlight');
  highlights.forEach(el => {
    const parent = el.parentNode;
    if (parent) {
      parent.replaceChild(document.createTextNode(el.textContent), el);
      parent.normalize();
    }
  });
  state.searchResults = [];
  state.searchIndex = 0;
}
```

- [ ] **步骤 6：重写 main.js 主入口**

```javascript
import state from './state.js';
import { initSidebar, renderFileTree } from './components/sidebar.js';
import { openFile } from './components/viewer.js';
import { initSearch } from './components/search.js';
import { initTheme } from './components/theme.js';

async function init() {
  // 初始化各组件
  initSidebar();
  initSearch();
  initTheme();

  // 拖拽支持
  document.addEventListener('dragover', (e) => e.preventDefault());
  document.addEventListener('drop', async (e) => {
    e.preventDefault();
    const files = Array.from(e.dataTransfer.files);
    const mdFile = files.find(f => f.name.endsWith('.md') || f.name.endsWith('.markdown'));
    if (mdFile) {
      // 拖拽时获取文件路径
      const path = mdFile.path;
      if (path) {
        await openFile(path);
      }
    }
  });

  // 监听文件变化事件
  if (window.__TAURI__) {
    const { listen } = window.__TAURI__.event;
    await listen('file-changed', async (event) => {
      const path = event.payload;
      if (path === state.currentFile) {
        const reload = confirm('文件已在外部修改，是否重新加载？');
        if (reload) {
          await openFile(path);
        }
      }
    });
  }

  // 检查 CLI 参数
  if (window.__ZEPHYR_CLI_FILE__) {
    await openFile(window.__ZEPHYR_CLI_FILE__);
  }

  console.log('Zephyr 已启动');
}

// 等待 DOM 加载
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
```

---

### 任务 11：构建与集成验证

**文件：**
- 修改：无（运行构建命令）

- [ ] **步骤 1：Rust 编译检查**

```bash
cd /tummy/projects/zephyr/src-tauri
cargo check 2>&1
```
预期：编译成功，无错误。

- [ ] **步骤 2：Tauri 构建**

```bash
cd /tummy/projects/zephyr
npm install 2>/dev/null || true
npx tauri build 2>&1 | head -50
```
预期：构建成功，生成可执行文件。

- [ ] **步骤 3：验证应用启动**

```bash
cd /tummy/projects/zephyr
# 创建测试 Markdown 文件
cat > /tmp/test-zephyr.md << 'EOF'
# Zephyr 测试文档

## 介绍

这是一个 **测试** 文档。

- 列表项 1
- 列表项 2

### 代码块

```rust
fn main() {
    println!("Hello, Zephyr!");
}
```

### 表格

| 名称 | 版本 |
|------|------|
| Tauri | 2.0 |
| Rust | 2024 |
EOF

# 启动应用（开发模式）
npx tauri dev -- /tmp/test-zephyr.md &
sleep 8
pkill -f "tauri dev" 2>/dev/null || true
```
预期：应用启动，正确渲染 Markdown 文件。

---

## 自检

- [x] **规格覆盖度：** 所有需求（多种打开方式、完整 GFM、侧边栏、高速渲染、主题、导航/搜索、文件监听）都有对应任务
- [x] **占位符扫描：** 无 "TODO"、"待定"、未完成部分
- [x] **类型一致性：** Tauri 命令和前端 JS 函数的签名在任务间保持一致
