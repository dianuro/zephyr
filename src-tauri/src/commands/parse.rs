use crate::parser::markdown::{self, MarkdownResult};

/// 直接渲染 Markdown 文本为 HTML（用于编辑器内预览等场景）
#[tauri::command]
pub fn render_markdown(content: String) -> MarkdownResult {
    markdown::render(&content)
}
