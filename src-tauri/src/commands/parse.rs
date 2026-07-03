use crate::parser::markdown::{self, MarkdownResult};

/// 直接渲染 Markdown 文本为 HTML
#[tauri::command]
pub fn render_markdown(content: String, is_dark: bool) -> MarkdownResult {
    markdown::render(&content, is_dark)
}
