/// Result of a single fetch operation.
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// Response body markdown.
    pub body: String,
    /// Extracted citations.
    pub cells: Vec<super::super::citations::Citation>,
    /// Reference links.
    pub links: Vec<String>,
}

/// Return log record for XAI request.
pub fn note(
    model: &str,
    turns: u32,
    tokens: u32,
    include: &[String],
    tools: &[String],
    text: &str,
) -> serde_json::Value {
    let lines = text.lines().count();
    let size = text.len();
    serde_json::json!({
        "kind": "xai_request",
        "model": model,
        "max_turns": turns,
        "max_tokens": tokens,
        "tools": tools,
        "include": include,
        "size": size,
        "lines": lines
    })
}

/// Return log line for prompt.
pub fn line(text: &str) -> String {
    format!("xai prompt {}", text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_py_client_note_stringifies_tools() {
        let mut rng = ids::ids(18301);
        let model = ids::latin(&mut rng, 6);
        let tool = ids::arabic(&mut rng, 5);
        let text = ids::greek(&mut rng, 7);
        let turn = 1 + ids::digit(&mut rng, 4) as u32;
        let token = 1 + ids::digit(&mut rng, 1000) as u32;
        let items: Vec<String> = vec![ids::armenian(&mut rng, 4), ids::hebrew(&mut rng, 4)];
        let result = note(&model, turn, token, &items, &[tool.clone()], &text);
        let tools = result.get("tools").unwrap().as_array().unwrap();
        let first = tools[0].as_str().unwrap();
        assert_eq!(tool, first, "tools were not stringified");
    }

    #[test]
    fn the_line_removes_periods_from_prompts() {
        let mut rng = ids::ids(18313);
        let alpha = ids::greek(&mut rng, 5);
        let beta = ids::greek(&mut rng, 4);
        let dot = '.';
        let gap = "\n\n";
        let text = format!("{}{}{}{}{}", alpha, dot, gap, beta, dot);
        let result = line(&text);
        let expect = format!("xai prompt {}", text);
        assert_eq!(expect, result, "prompt line was not sanitized");
    }
}
