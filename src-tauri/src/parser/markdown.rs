use comrak::nodes::{AstNode, NodeValue};
use comrak::plugins::syntect::SyntectAdapterBuilder;
use comrak::{format_html_with_plugins, parse_document, Arena, Options, Plugins};
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::OnceLock;

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

/// 渲染 Markdown 为 HTML
/// `is_dark` 控制代码语法高亮的主题（亮色/暗色）
pub fn render(markdown: &str, is_dark: bool) -> MarkdownResult {
    let arena = Arena::new();

    let mut options = Options::default();
    options.extension.autolink = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.extension.tagfilter = true;
    options.extension.footnotes = true;
    options.render.github_pre_lang = true;
    options.render.full_info_string = true;
    options.render.unsafe_ = false;
    options.extension.header_ids = Some(String::new());

    // 从主题配置读取语法高亮主题名称
    // 注意：这里直接使用默认值，完整方案需要将 config 传入
    // 对于 CLI 启动的场景，从配置目录读取
    let theme_name = get_syntax_theme(is_dark);
    let syntax_highlighter = SyntectAdapterBuilder::new()
        .theme(&theme_name)
        .build();

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&syntax_highlighter);

    let root = parse_document(&arena, markdown, &options);

    // 提取标题结构（与 comrak 共享 ID 跟踪，确保 DOM ID 一致）
    let mut used_ids = HashSet::new();
    let headings = extract_headings(root, &mut used_ids);
    let title = headings
        .first()
        .map(|h| h.text.clone())
        .unwrap_or_default();
    let word_count = count_words(markdown);

    // 格式化 HTML
    let mut html_buf = Vec::new();
    format_html_with_plugins(root, &options, &mut html_buf, &plugins)
        .expect("HTML 格式化失败");
    let html = String::from_utf8(html_buf).expect("HTML 不是合法 UTF-8");

    MarkdownResult {
        html,
        metadata: MarkdownMetadata {
            title,
            headings,
            word_count,
        },
    }
}

/// 从 AST 中提取标题结构，构建嵌套树
fn extract_headings<'a>(node: &'a AstNode<'a>, used: &mut HashSet<String>) -> Vec<Heading> {
    let flat = collect_all_headings(node);
    build_nested_tree(&flat, used)
}

fn collect_all_headings<'a>(node: &'a AstNode<'a>) -> Vec<(u8, String)> {
    let mut result = Vec::new();
    for child in node.children() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Heading(ref heading) => {
                let text = extract_heading_text(child);
                result.push((heading.level, text));
            }
            _ => {
                // 有些平台或格式可能在其他结构中有 heading
                // 但标准 Markdown 中 heading 是 block-level，直接是 document 的子节点
            }
        }
        drop(data);
    }
    result
}

fn extract_heading_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    for child in node.children() {
        let data = child.data.borrow();
        match &data.value {
            NodeValue::Text(t) => text.push_str(t),
            NodeValue::Code(c) => text.push_str(&c.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => text.push(' '),
            _ => {}
        }
        drop(data);
    }
    text.trim().to_string()
}

/// 将扁平的标题列表按层级构建为嵌套树
fn build_nested_tree(flat: &[(u8, String)], used: &mut HashSet<String>) -> Vec<Heading> {
    let items: Vec<(u8, String)> = flat.to_vec();
    let (headings, _) = parse_headings(&items, 0, 0, used);
    headings
}

/// 递归解析 headings：从 start 开始，读取所有 level > parent_level 的项
/// 返回 (解析出的 headings, 消耗的项数)
fn parse_headings(
    items: &[(u8, String)],
    start: usize,
    parent_level: u8,
    used: &mut HashSet<String>,
) -> (Vec<Heading>, usize) {
    let mut headings = Vec::new();
    let mut pos = start;

    while pos < items.len() {
        let (level, text) = &items[pos];
        if *level <= parent_level {
            break;
        }

        let id = anchorize_id(text, used);

        // 检查下一个项是否是子标题
        let (children, consumed) = if pos + 1 < items.len() && items[pos + 1].0 > *level {
            parse_headings(items, pos + 1, *level, used)
        } else {
            (Vec::new(), 0)
        };

        headings.push(Heading {
            level: *level,
            text: text.clone(),
            id,
            children,
        });

        if consumed > 0 {
            pos += 1 + consumed;
        } else {
            pos += 1;
        }
    }

    (headings, pos - start)
}

/// 从配置文件中读取语法高亮主题名称
pub fn get_syntax_theme(is_dark: bool) -> String {
    let config = crate::theme_config::get_config(is_dark);
    config.syntax.theme
}

/// 统计单词数
fn count_words(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

/// 生成与 comrak Anchorizer::anchorize 完全一致的 anchor ID
fn anchorize_id(text: &str, used: &mut HashSet<String>) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let rejected_chars =
        RE.get_or_init(|| Regex::new(r"[^\p{L}\p{M}\p{N}\p{Pc} -]").unwrap());

    // 必须与 comrak Anchorizer::anchorize 算法完全一致：
    // 1. 转小写
    // 2. 移除所有非 [\p{L}\p{M}\p{N}\p{Pc} -] 字符
    // 3. 空格替换为连字符
    // 4. 处理重复（追加 -1, -2, ...）
    let mut id = text.to_lowercase();
    id = rejected_chars.replace_all(&id, "").to_string();
    id = id.replace(' ', "-");

    let mut uniq: u32 = 0;
    let result = loop {
        let anchor = if uniq == 0 {
            id.clone()
        } else {
            format!("{}-{}", id, uniq)
        };
        if !used.contains(&anchor) {
            break anchor;
        }
        uniq += 1;
    };

    used.insert(result.clone());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown() {
        let result = render("# Hello\n\nThis is **bold** text.", false);
        assert!(result.html.contains("Hello"));
        assert!(result.html.contains("<strong") || result.html.contains("<b"));
        assert_eq!(result.metadata.title, "Hello");
        assert_eq!(result.metadata.headings.len(), 1);
    }

    #[test]
    fn test_render_code_block() {
        let result = render("```rust\nfn main() {}\n```", false);
        assert!(result.html.contains("<pre") && result.html.contains("</pre>"));
        assert!(result.html.contains("style=\""));
    }

    #[test]
    fn test_extract_headings() {
        let result = render("# A\n\n## B\n\n# C", false);
        assert_eq!(result.metadata.headings.len(), 2);
        assert_eq!(result.metadata.headings[0].level, 1);
        assert_eq!(result.metadata.headings[0].text, "A");
        assert_eq!(result.metadata.headings[0].children.len(), 1);
        assert_eq!(result.metadata.headings[0].children[0].text, "B");
        assert_eq!(result.metadata.headings[1].text, "C");
    }

    #[test]
    fn test_empty_document() {
        let result = render("", false);
        assert!(result.metadata.title.is_empty());
        assert_eq!(result.metadata.word_count, 0);
    }

    #[test]
    fn test_word_count() {
        let result = render("# Title\n\nHello world foo bar", false);
        assert!(result.metadata.word_count > 0);
    }

    #[test]
    fn test_table_rendering() {
        let result = render("| A | B |\n|---|---|\n| 1 | 2 |", false);
        assert!(result.html.contains("<table"));
        assert!(result.html.contains("<th"));
        assert!(result.html.contains("<td"));
    }

    #[test]
    fn test_task_list() {
        let result = render("- [x] done\n- [ ] todo", false);
        assert!(result.html.contains("checked") || result.html.contains("checkbox"));
    }

    #[test]
    fn test_anchorize_id() {
        let mut used = HashSet::new();
        // comrak 算法：& 被移除后留下两个空格，变成两个连字符
        assert_eq!(anchorize_id("Foo & Bar", &mut used), "foo--bar");
        assert_eq!(anchorize_id("Hello World", &mut used), "hello-world");
        // 重复标题触发生成 -1
        assert_eq!(anchorize_id("Hello World", &mut used), "hello-world-1");
        // 新的 HashSet 从头开始计数
        let mut used2 = HashSet::new();
        assert_eq!(anchorize_id("Hello World", &mut used2), "hello-world");
    }

    #[test]
    fn test_nested_headings() {
        let md = "# L1\n\n## L2a\n\n### L3\n\n## L2b";
        let result = render(md, false);
        assert_eq!(result.metadata.headings.len(), 1);
        let h1 = &result.metadata.headings[0];
        assert_eq!(h1.text, "L1");
        assert_eq!(h1.level, 1);
        assert_eq!(h1.children.len(), 2);
        assert_eq!(h1.children[0].text, "L2a");
        assert_eq!(h1.children[0].children.len(), 1);
        assert_eq!(h1.children[0].children[0].text, "L3");
        assert_eq!(h1.children[1].text, "L2b");
    }

    #[test]
    fn test_footnotes() {
        let md = "Hello[^1]\n\n[^1]: World";
        let result = render(md, false);
        assert!(result.html.contains("footnote") || result.html.contains("sup"));
    }
}


