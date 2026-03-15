use regex::Regex;
use serde::{Deserialize, Serialize};

/// Research question with scope and sub-questions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Question {
    /// Research scope.
    pub scope: String,
    /// Sub-questions decomposition.
    pub details: Vec<Question>,
}

/// Structured research brief.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Brief {
    /// Research title.
    pub title: String,
    /// Result language.
    pub language: String,
    /// Root research questions.
    pub questions: Vec<Question>,
}

/// Flat item with depth level for intermediate parsing.
struct Flat {
    depth: usize,
    text: String,
}

/// Normalize brief question.
pub fn question(item: &Question) -> Question {
    let scope = item.scope.trim().to_string();
    let details: Vec<Question> = item
        .details
        .iter()
        .map(question)
        .filter(|n| !(n.scope.is_empty() && n.details.is_empty()))
        .collect();
    Question { scope, details }
}

/// Create question from plain text string.
pub fn leaf(text: &str) -> Question {
    Question {
        scope: text.trim().to_string(),
        details: Vec::new(),
    }
}

/// Encode brief into RON string.
pub fn encode(brief: &Brief) -> String {
    ron::ser::to_string_pretty(brief, ron::ser::PrettyConfig::default())
        .expect("Brief RON serialization must not fail")
}

/// Check if line is a numbered or bullet item.
fn marker(line: &str) -> bool {
    let trim = line.trim();
    let numbered = Regex::new(r"^(\d+(?:\.\d+)*)[.)]\s+.+").unwrap();
    let bullet = Regex::new(r"^[*+\-]\s+.+").unwrap();
    numbered.is_match(trim) || bullet.is_match(trim)
}

/// Parse list line into depth item.
fn point(line: &str) -> Option<Flat> {
    let raw = line;
    let tabs = raw.chars().take_while(|c| *c == '\t').count();
    let replaced = raw.replace('\t', " ");
    let trim = replaced.trim_start();
    let pad = replaced.len() - trim.len();
    let num = Regex::new(r"^(\d+(?:\.\d+)*)[.)]\s+(.+)$").unwrap();
    let bul = Regex::new(r"^[*+\-]\s+(.+)$").unwrap();
    let num_caps = num.captures(trim);
    let bul_caps = bul.captures(trim);
    let plain = num_caps.is_none()
        && bul_caps.is_none()
        && !trim.trim().is_empty()
        && (tabs > 0 || pad == 0);
    let text = if let Some(ref caps) = num_caps {
        Some(caps[2].to_string())
    } else if let Some(ref caps) = bul_caps {
        Some(caps[1].to_string())
    } else if plain {
        Some(trim.to_string())
    } else {
        None
    };
    let base = if let Some(ref caps) = num_caps {
        Some(caps[1].split('.').count())
    } else if bul_caps.is_some() {
        Some(1 + pad / 2)
    } else if plain {
        Some(1 + tabs)
    } else {
        None
    };
    let depth = if let Some(ref caps) = num_caps {
        let b = caps[1].split('.').count();
        if b > 1 {
            Some(b)
        } else if pad > 0 {
            Some(1 + pad / 4)
        } else {
            Some(b)
        }
    } else if bul_caps.is_some() || plain {
        base
    } else {
        None
    };
    let depth = depth.map(|d| d.clamp(1, 3));
    let text = text.unwrap_or_default().trim().to_string();
    if let Some(d) = depth {
        if !text.is_empty() {
            return Some(Flat { depth: d, text });
        }
    }
    None
}

/// Parse list lines into flat items.
fn scan(lines: &[&str]) -> Vec<Flat> {
    let mut list: Vec<Flat> = Vec::new();
    for raw in lines {
        let item = point(raw);
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(item) = item {
            list.push(item);
        } else if let Some(last) = list.last_mut() {
            last.text = format!("{} {}", last.text, line);
        }
    }
    list
}

/// Insert item at depth.
fn place(items: &mut Vec<Question>, depth: usize, item: Question) {
    let depth = if depth > 1 && items.is_empty() {
        1
    } else {
        depth
    };
    if depth == 1 {
        items.push(item);
    } else {
        if items.is_empty() {
            items.push(Question {
                scope: String::new(),
                details: Vec::new(),
            });
        }
        let last = items.last_mut().unwrap();
        place(&mut last.details, depth - 1, item);
    }
}

/// Nest flat items into tree.
fn nest(list: &[Flat]) -> Vec<Question> {
    let mut items: Vec<Question> = Vec::new();
    for flat in list {
        let n = Question {
            scope: flat.text.clone(),
            details: Vec::new(),
        };
        place(&mut items, flat.depth, n);
    }
    items
}

/// Render nested items into numbered list.
fn lines(items: &[Question], prefix: &str) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        let text = item.scope.trim();
        let nested = &item.details;
        let num = if prefix.is_empty() {
            format!("{}", idx + 1)
        } else {
            format!("{}.{}", prefix, idx + 1)
        };
        let rows = lines(nested, &num);
        let line = if text.is_empty() {
            None
        } else {
            Some(format!("{}. {}", num, text))
        };
        match (line, rows.is_empty()) {
            (None, false) => list.extend(rows),
            (None, true) => {}
            (Some(l), false) => {
                list.push(l);
                list.extend(rows);
            }
            (Some(l), true) => list.push(l),
        }
    }
    list
}

/// Render brief into query text.
pub fn render(brief: &Brief) -> String {
    let lang = brief.language.trim();
    let lead = if lang.is_empty() {
        String::new()
    } else {
        format!("\u{042f}\u{0437}\u{044b}\u{043a} \u{043e}\u{0442}\u{0432}\u{0435}\u{0442}\u{0430}: {}.", lang)
    };
    let title = &brief.title;
    let items: Vec<Question> = brief.questions.iter().map(question).collect();
    let rows = lines(&items, "");
    let tail = rows.join("\n");
    let body = if !rows.is_empty() {
        if title.is_empty() {
            format!("Research:\n{}", tail)
        } else {
            format!("{}\n\nResearch:\n{}", title, tail)
        }
    } else {
        title.clone()
    };
    if !lead.is_empty() && !body.is_empty() {
        format!("{}\n\n{}", lead, body)
    } else if !lead.is_empty() {
        lead
    } else {
        body
    }
}

/// Parse query text into Brief structure.
pub fn parse(
    query: &str,
    language: &str,
    explicit_title: Option<&str>,
    explicit_questions: Option<&[Question]>,
) -> Brief {
    let rows: Vec<&str> = query.lines().collect();
    let label = "Research:";
    let spot = rows.iter().position(|line| line.trim() == label);
    let edge = rows.iter().position(|line| marker(line));
    let cut = spot.or(edge);
    let head: Vec<&str> = match cut {
        Some(c) => rows[..c].to_vec(),
        None => rows.clone(),
    };
    let tail: Vec<&str> = match cut {
        Some(c) => {
            let start = if spot.is_some() { c + 1 } else { c };
            if start < rows.len() {
                rows[start..].to_vec()
            } else {
                Vec::new()
            }
        }
        None => Vec::new(),
    };
    let list = nest(&scan(&tail));
    let top = head.iter().fold(String::new(), |acc, line| {
        if line.trim().is_empty() {
            acc
        } else {
            line.trim().to_string()
        }
    });
    let title = explicit_title.map(|s| s.to_string()).unwrap_or(top);
    let questions = match explicit_questions {
        Some(explicit) if !explicit.is_empty() => explicit.to_vec(),
        _ => list,
    };
    let questions: Vec<Question> = questions.iter().map(question).collect();
    Brief {
        title,
        language: language.to_string(),
        questions,
    }
}

/// Serialize brief for data output.
pub fn data(brief: &Brief) -> serde_json::Value {
    let questions: Vec<serde_json::Value> = brief.questions.iter().map(question_to_value).collect();
    serde_json::json!({
        "title": brief.title,
        "language": brief.language,
        "questions": questions,
    })
}

/// Convert question to JSON value.
fn question_to_value(n: &Question) -> serde_json::Value {
    let details: Vec<serde_json::Value> = n.details.iter().map(question_to_value).collect();
    serde_json::json!({
        "scope": n.scope,
        "details": details,
    })
}
