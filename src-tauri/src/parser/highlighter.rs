use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use std::sync::OnceLock;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(|| SyntaxSet::load_defaults_newlines())
}

fn get_theme() -> &'static Theme {
    let ts = THEME_SET.get_or_init(ThemeSet::load_defaults);
    &ts.themes["InspiredGitHub"]
}

/// 将代码块高亮为 HTML
///
/// 返回完整的 `<pre><code>` HTML 片段，其中包含 syntect 生成的语法高亮 span。
pub fn highlight(code: &str, lang: Option<&str>) -> String {
    let lang = lang.unwrap_or("");

    if lang.is_empty() {
        // 无语言指定，使用纯文本
        return format!(
            "<pre style=\"background-color:#f6f8fa;\" data-lang=\"\"><code>{}</code></pre>\n",
            html_escape(code)
        );
    }

    let ss = get_syntax_set();
    let syntax = ss
        .find_syntax_by_token(lang)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme = get_theme();

    match highlighted_html_for_string(code, ss, syntax, theme) {
        Ok(html) => html,
        Err(_) => format!(
            "<pre style=\"background-color:#f6f8fa;\" data-lang=\"{}\"><code>{}</code></pre>\n",
            html_escape(lang),
            html_escape(code)
        ),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_rust() {
        let code = "fn main() {\n    println!(\"hello\");\n}";
        let html = highlight(code, Some("rust"));
        assert!(html.contains("<pre") && html.contains("</pre>"));
        assert!(html.contains("fn"));
        assert!(html.contains("println"));
    }

    #[test]
    fn test_highlight_no_lang() {
        let html = highlight("plain text", None);
        assert!(html.contains("plain text"));
        assert!(html.contains("<pre"));
    }

    #[test]
    fn test_highlight_unknown_lang() {
        let html = highlight("some code", Some("nonexistent_lang_xyz"));
        assert!(html.contains("some code"));
    }

    #[test]
    fn test_xss_prevention() {
        let code = "<script>alert('xss')</script>";
        let html = highlight(code, None);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_highlight_python() {
        let code = "def hello():\n    print('world')";
        let html = highlight(code, Some("python"));
        assert!(html.contains("def"));
        assert!(html.contains("hello"));
    }

    #[test]
    fn test_empty_code() {
        let html = highlight("", Some("rust"));
        assert!(html.contains("<pre") && html.contains("</pre>"));
    }
}
