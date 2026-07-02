use comrak::nodes::{AstNode, NodeValue};
use comrak::plugins::syntect::SyntectAdapterBuilder;
use comrak::{format_html_with_plugins, parse_document, Arena, Options, Plugins};
use serde::Serialize;

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
pub fn render(markdown: &str) -> MarkdownResult {
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

    // 使用 comrak 内置的 SyntectAdapter（InspiredGitHub 主题）
    let syntax_highlighter = SyntectAdapterBuilder::new()
        .theme("InspiredGitHub")
        .build();

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&syntax_highlighter);

    let root = parse_document(&arena, markdown, &options);

    // 提取标题结构
    let headings = extract_headings(root);
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
fn extract_headings<'a>(node: &'a AstNode<'a>) -> Vec<Heading> {
    let flat = collect_all_headings(node);
    build_nested_tree(&flat)
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
fn build_nested_tree(flat: &[(u8, String)]) -> Vec<Heading> {
    let items: Vec<(u8, String)> = flat.to_vec();
    let (headings, _) = parse_headings(&items, 0, 0);
    headings
}

/// 递归解析 headings：从 start 开始，读取所有 level > parent_level 的项
/// 返回 (解析出的 headings, 消耗的项数)
fn parse_headings(items: &[(u8, String)], start: usize, parent_level: u8) -> (Vec<Heading>, usize) {
    let mut headings = Vec::new();
    let mut pos = start;

    while pos < items.len() {
        let (level, text) = &items[pos];
        if *level <= parent_level {
            break;
        }

        let id = slugify(text);

        // 检查下一个项是否是子标题
        let (children, consumed) = if pos + 1 < items.len() && items[pos + 1].0 > *level {
            parse_headings(items, pos + 1, *level)
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

/// 统计单词数
fn count_words(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

/// 生成 GitHub 风格的 anchor id
fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut prev_was_hyphen = false;

    for c in text.to_lowercase().chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            slug.push(c);
            prev_was_hyphen = c == '-';
        } else if c.is_whitespace() || c == '.' || c == ',' || c == '!' || c == '?' {
            if !prev_was_hyphen && !slug.is_empty() {
                slug.push('-');
                prev_was_hyphen = true;
            }
        } else if c == '&' {
            if !prev_was_hyphen && !slug.is_empty() {
                slug.push('-');
                prev_was_hyphen = true;
            }
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown() {
        let result = render("# Hello\n\nThis is **bold** text.");
        assert!(result.html.contains("Hello"));
        assert!(result.html.contains("<strong") || result.html.contains("<b"));
        assert_eq!(result.metadata.title, "Hello");
        assert_eq!(result.metadata.headings.len(), 1);
    }

    #[test]
    fn test_render_code_block() {
        let result = render("```rust\nfn main() {}\n```");
        assert!(result.html.contains("<pre") && result.html.contains("</pre>"));
        // InspiredGitHub theme uses inline styles
        assert!(result.html.contains("style=\""));
    }

    #[test]
    fn test_extract_headings() {
        let result = render("# A\n\n## B\n\n# C");
        assert_eq!(result.metadata.headings.len(), 2);
        assert_eq!(result.metadata.headings[0].level, 1);
        assert_eq!(result.metadata.headings[0].text, "A");
        assert_eq!(result.metadata.headings[0].children.len(), 1);
        assert_eq!(result.metadata.headings[0].children[0].text, "B");
        assert_eq!(result.metadata.headings[1].text, "C");
    }

    #[test]
    fn test_empty_document() {
        let result = render("");
        assert!(result.metadata.title.is_empty());
        assert_eq!(result.metadata.word_count, 0);
    }

    #[test]
    fn test_word_count() {
        let result = render("# Title\n\nHello world foo bar");
        assert!(result.metadata.word_count > 0);
    }

    #[test]
    fn test_table_rendering() {
        let result = render("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(result.html.contains("<table"));
        assert!(result.html.contains("<th"));
        assert!(result.html.contains("<td"));
    }

    #[test]
    fn test_task_list() {
        let result = render("- [x] done\n- [ ] todo");
        assert!(result.html.contains("checked") || result.html.contains("checkbox"));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Foo & Bar"), "foo-bar");
        assert_eq!(slugify("  spaces  "), "spaces");
    }

    #[test]
    fn test_nested_headings() {
        let md = "# L1\n\n## L2a\n\n### L3\n\n## L2b";
        let result = render(md);
        // Only L1 at top level; L2a, L3, L2b are all nested under L1
        assert_eq!(result.metadata.headings.len(), 1);
        let h1 = &result.metadata.headings[0];
        assert_eq!(h1.text, "L1");
        assert_eq!(h1.level, 1);
        // L2a and L2b are children of L1
        assert_eq!(h1.children.len(), 2);
        assert_eq!(h1.children[0].text, "L2a");
        // L3 is child of L2a
        assert_eq!(h1.children[0].children.len(), 1);
        assert_eq!(h1.children[0].children[0].text, "L3");
        assert_eq!(h1.children[1].text, "L2b");
    }

    #[test]
    fn test_footnotes() {
        let md = "Hello[^1]\n\n[^1]: World";
        let result = render(md);
        assert!(result.html.contains("footnote") || result.html.contains("sup"));
    }
}
