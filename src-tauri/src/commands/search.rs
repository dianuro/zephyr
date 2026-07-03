use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SearchMatch {
    pub line: u32,
    pub column: u32,
    pub context: String,
    pub matched_text: String,
}

/// 在文档内容中搜索文本（大小写不敏感）
#[tauri::command]
pub fn search_in_document(content: String, query: String) -> Result<Vec<SearchMatch>, String> {
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let re = Regex::new(&regex::escape(&query))
        .map_err(|e| format!("搜索表达式错误: {}", e))?;

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

    #[test]
    fn test_search_multiline() {
        let content = "line1\nline2\nline1 again";
        let results = search_in_document(content.to_string(), "line1".to_string()).unwrap();
        assert_eq!(results.len(), 2);
    }
}
