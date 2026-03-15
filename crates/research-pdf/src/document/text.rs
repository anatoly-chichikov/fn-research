use regex::Regex;

use research_domain::brief::Question;
use research_domain::result::Sourced;

/// Escape HTML special characters.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Decode HTML entities.
pub fn decode(text: &str) -> String {
    let text = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&#x2F;", "/")
        .replace("&nbsp;", "\u{00a0}");
    let re = Regex::new(r"&#(\d+);").unwrap();
    let text = re.replace_all(&text, |caps: &regex::Captures| {
        let num: u32 = caps[1].parse().unwrap_or(0);
        char::from_u32(num).map_or(String::new(), |c| c.to_string())
    });
    let hex = Regex::new(r"&#x([0-9a-fA-F]+);").unwrap();
    hex.replace_all(&text, |caps: &regex::Captures| {
        let num = u32::from_str_radix(&caps[1], 16).unwrap_or(0);
        char::from_u32(num).map_or(String::new(), |c| c.to_string())
    })
    .to_string()
}

/// Return heading text with uppercase initial letter.
pub fn heading(text: &str) -> String {
    let text = text.trim();
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let head: String = first.to_uppercase().collect();
            format!("{}{}", head, chars.as_str())
        }
    }
}

/// Return anchor slug.
pub fn slug(text: &str) -> String {
    let text = decode(text);
    let text = text.trim().to_lowercase();
    let re = Regex::new(r"[\s\p{P}]+").unwrap();
    let text = re.replace_all(&text, "-").to_string();
    let edge = Regex::new(r"^-+|-+$").unwrap();
    let text = edge.replace_all(&text, "").to_string();
    if text.is_empty() {
        "section".to_string()
    } else {
        text
    }
}

/// Anchor data with html and toc items.
pub struct AnchorData {
    /// Processed html.
    pub html: String,
    /// Table of contents items.
    pub items: Vec<TocItem>,
}

/// Table of contents entry.
pub struct TocItem {
    /// Heading text.
    pub text: String,
    /// Anchor identifier.
    pub id: String,
    /// Heading level.
    pub level: u32,
}

/// Add anchor ids to headings and return html with toc items.
pub fn anchors(html: &str) -> AnchorData {
    let heading_re = Regex::new(r"(?s)<(h[1-6])([^>]*)>(.*?)</h[1-6]>").unwrap();
    let tag_re = Regex::new(r"<[^>]*>").unwrap();
    let mut items = Vec::new();
    let mut idx: u32 = 0;
    let result = heading_re
        .replace_all(html, |caps: &regex::Captures| {
            let tag = &caps[1];
            let attrs = &caps[2];
            let body = &caps[3];
            let text = tag_re.replace_all(body, "").to_string();
            idx += 1;
            let id = format!("section-{}", idx);
            let level: u32 = tag[1..].parse().unwrap_or(1);
            items.push(TocItem {
                text: text.clone(),
                id: id.clone(),
                level,
            });
            let back = if tag == "h2" {
                "<a class=\"toc-back\" href=\"#toc\" \
                     aria-label=\"Back to contents\"></a>"
                    .to_string()
            } else {
                String::new()
            };
            format!("<{}{} id=\"{}\">{}{}</{}>", tag, attrs, id, back, body, tag)
        })
        .to_string();
    AnchorData {
        html: result,
        items,
    }
}

/// Render table of contents html.
pub fn toc(items: &[TocItem]) -> String {
    let top: Vec<&TocItem> = items
        .iter()
        .filter(|i| i.text != "Exploration Brief")
        .collect();
    let base = if !top.is_empty() {
        top.iter().map(|i| i.level).min().unwrap_or(1)
    } else if !items.is_empty() {
        items.iter().map(|i| i.level).min().unwrap_or(1)
    } else {
        1
    };
    let levels: std::collections::HashSet<u32> = top.iter().map(|i| i.level).collect();
    let linklevels: std::collections::HashSet<u32> = if levels.contains(&1) {
        [1, 2].into()
    } else if levels.contains(&2) {
        [2].into()
    } else {
        [base].into()
    };
    struct Group<'a> {
        item: &'a TocItem,
        subs: Vec<&'a TocItem>,
    }
    let mut groups: Vec<Group<'_>> = Vec::new();
    let mut current: Option<Group<'_>> = None;
    for item in items {
        let brief = item.text == "Exploration Brief";
        let root = brief || linklevels.contains(&item.level);
        if root {
            if let Some(group) = current.take() {
                groups.push(group);
            }
            current = Some(Group {
                item,
                subs: Vec::new(),
            });
        } else if let Some(ref mut group) = current {
            group.subs.push(item);
        }
    }
    if let Some(group) = current {
        groups.push(group);
    }
    let mut rows = String::new();
    for entry in &groups {
        let name = escape(&entry.item.text);
        let id = escape(&entry.item.id);
        let link = format!(
            "<a class=\"ref-link toc-row\" href=\"#{}\"><span class=\"toc-text\">{}</span>\
             <span class=\"toc-page\" data-target=\"#{}\"></span></a>",
            id, name, id
        );
        let mut subrows = String::new();
        for sub in &entry.subs {
            let subname = escape(&sub.text);
            let subid = escape(&sub.id);
            subrows.push_str(&format!(
                "<li class=\"toc-subitem\"><span class=\"toc-subtext\">{}</span>\
                 <span class=\"toc-subpage\" data-target=\"#{}\"></span></li>",
                subname, subid
            ));
        }
        let desc = if subrows.is_empty() {
            String::new()
        } else {
            format!("<ul class=\"toc-sublist\">{}</ul>", subrows)
        };
        rows.push_str(&format!(
            "<li class=\"ref-item toc-item\">{}{}</li>",
            link, desc
        ));
    }
    if rows.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"toc\" id=\"toc\"><div class=\"container\">\
             <h1>Table of Contents</h1>\
             <ul class=\"ref-list toc-list\">{}</ul></div></div>",
            rows
        )
    }
}

/// Add blank lines before list markers.
pub fn normalize(text: &str) -> String {
    let text = text.replace("\\n", "\n");
    let rows: Vec<&str> = text.split('\n').collect();
    let mark = Regex::new(r"^\s*(?:[*+\-] |\d+\. )").unwrap();
    let size = rows.len();
    let mut out = Vec::with_capacity(size);
    for idx in 0..size {
        let row = rows[idx];
        let prev = if idx > 0 { rows[idx - 1] } else { "" };
        let list = mark.is_match(row);
        let back = mark.is_match(prev);
        let blank = prev.trim().is_empty();
        let gap = list && !back && !blank && idx > 0;
        if gap {
            out.push("");
        }
        out.push(row);
    }
    out.join("\n")
}

/// Remove blank lines between markdown table rows.
pub fn tablerows(text: &str) -> String {
    let rows: Vec<&str> = text.split('\n').collect();
    let size = rows.len();
    let pipe = Regex::new(r"^\s*[|]").unwrap();
    let dash = Regex::new(r"^\s*[|]?[\s:\-]*-[-|\s:]*$").unwrap();
    let mut out = Vec::with_capacity(size);
    let mut past = "";
    for idx in 0..size {
        let row = rows[idx];
        let tail = if idx + 1 < size { rows[idx + 1] } else { "" };
        let blank = row.trim().is_empty();
        let skip = blank && pipe.is_match(tail) && (pipe.is_match(past) || dash.is_match(past));
        if !skip {
            out.push(row);
            past = row;
        }
    }
    out.join("\n")
}

/// Move trailing citations into the last table cell.
pub fn tablecite(text: &str) -> String {
    let rows: Vec<&str> = text.split('\n').collect();
    let rule = Regex::new(r"^(\s*[|].*)[|]\s*(\[\[\d+\]\].*)$").unwrap();
    let mut out = Vec::with_capacity(rows.len());
    for row in &rows {
        if let Some(caps) = rule.captures(row) {
            let head = caps[1].trim_end();
            let tail = &caps[2];
            out.push(format!("{} {} |", head, tail));
        } else {
            out.push(row.to_string());
        }
    }
    out.join("\n")
}

/// Ensure table rows end with pipe.
pub fn tablepipe(text: &str) -> String {
    let rows: Vec<&str> = text.split('\n').collect();
    let head_re = Regex::new(r"^\s*[|]").unwrap();
    let dash_re = Regex::new(r"^\s*[|]?[\s:\-]*-[-|\s:]*$").unwrap();
    let trail_re = Regex::new(r"^(.*?)[|]\s*$").unwrap();
    let mut out = Vec::with_capacity(rows.len());
    for row in &rows {
        if head_re.is_match(row) {
            if dash_re.is_match(row) {
                let base = row.trim_end();
                if base.ends_with('|') {
                    out.push(base.to_string());
                } else {
                    out.push(format!("{}|", base));
                }
            } else if let Some(caps) = trail_re.captures(row) {
                let base = &caps[1];
                let base = if base.ends_with(' ') {
                    base.to_string()
                } else {
                    format!("{} ", base)
                };
                out.push(format!("{}|", base));
            } else {
                let base = if row.ends_with(' ') {
                    row.to_string()
                } else {
                    format!("{} ", row)
                };
                out.push(format!("{}|", base));
            }
        } else {
            out.push(row.to_string());
        }
    }
    out.join("\n")
}

/// Remove list markers before table rows.
pub fn tablelead(text: &str) -> String {
    let rows: Vec<&str> = text.split('\n').collect();
    let rule = Regex::new(r"^\s*[*+\-]\s+([|].*)$").unwrap();
    let mark = Regex::new(r"^\s*\d+[.)]\s+(\|.*)$").unwrap();
    let mut out = Vec::with_capacity(rows.len());
    for row in &rows {
        let trim = row.trim_start();
        if let Some(caps) = rule.captures(row) {
            out.push(caps[1].to_string());
        } else if let Some(caps) = mark.captures(row) {
            out.push(caps[1].to_string());
        } else if *row != trim && trim.starts_with('|') {
            out.push(trim.to_string());
        } else {
            out.push(row.to_string());
        }
    }
    out.join("\n")
}

/// Convert inline prompts into markdown lists.
pub fn listify(text: &str) -> String {
    let text = text.replace("\\n", "\n");
    let re1 = Regex::new(r"\s+Research:").unwrap();
    let text = re1.replace_all(&text, "\n\nResearch:").to_string();
    let re2 = Regex::new(r"(?m)(^|\n)(\s*)(\d+)\)").unwrap();
    let text = re2.replace_all(&text, "$1$2$3.").to_string();
    let re3 = Regex::new(r"[ \t]+(\d+)[.)]\s+").unwrap();
    let text = re3.replace_all(&text, "\n$1. ").to_string();
    let rows: Vec<&str> = text.split('\n').collect();
    let numbered = Regex::new(r"^\s*(?:\d+\.|[*+\-])\s+").unwrap();
    let inline = Regex::new(r"[ \t]+([*+\-])\s+").unwrap();
    let rows: Vec<String> = rows
        .iter()
        .map(|row| {
            if numbered.is_match(row) {
                row.to_string()
            } else {
                inline.replace_all(row, "\n$1 ").to_string()
            }
        })
        .collect();
    let text = rows.join("\n");
    let multi = Regex::new(r"\n{3,}").unwrap();
    multi.replace_all(&text, "\n\n").to_string()
}

/// Trim brief question.
pub fn trim_question(node: &Question) -> Question {
    let scope = node.scope.trim().to_string();
    let details: Vec<Question> = node
        .details
        .iter()
        .map(trim_question)
        .filter(|n| !(n.scope.is_empty() && n.details.is_empty()))
        .collect();
    Question { scope, details }
}

/// Render nested list html.
pub fn outline(items: &[Question]) -> String {
    let items: Vec<Question> = items.iter().map(trim_question).collect();
    let mut rows = Vec::new();
    for entry in &items {
        let text = escape(&entry.scope);
        let nest = outline(&entry.details);
        let body = if nest.is_empty() { String::new() } else { nest };
        rows.push(format!("<li>{}{}</li>", text, body));
    }
    let body: String = rows.join("");
    if body.is_empty() {
        String::new()
    } else {
        format!("<ol>{}</ol>", body)
    }
}

/// Convert markdown separators to hr tags.
pub fn rule(text: &str) -> String {
    text.replace("\n---\n", "\n\n<hr />\n\n")
}

/// Normalize list indent to four spaces.
pub fn nested(text: &str) -> String {
    let re = Regex::new(r"(?m)^( {1,3})([*+\-] )").unwrap();
    re.replace_all(text, "    $2").to_string()
}

/// Wrap list item text in paragraph tags.
pub fn paragraphs(html: &str) -> String {
    if html.trim().is_empty() {
        return String::new();
    }
    let tree = dom_query::Document::from(html);
    let block_tags = [
        "p",
        "ul",
        "ol",
        "table",
        "pre",
        "div",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "blockquote",
    ];
    for item in tree.select("li").iter() {
        let body = item.inner_html().to_string();
        let has_block = block_tags
            .iter()
            .any(|tag| body.contains(&format!("<{}", tag)));
        if !has_block && !body.trim().is_empty() {
            let wrapped = format!("<p>{}</p>", body);
            item.set_html(wrapped.as_str());
        }
    }
    tree.select("body").inner_html().to_string()
}

/// Remove utm parameters from URL.
pub fn trim(text: &str) -> String {
    match url::Url::parse(text) {
        Ok(parsed) => {
            let query = parsed.query().unwrap_or("");
            if query.is_empty() {
                return text.to_string();
            }
            let pairs: Vec<(&str, &str)> = query
                .split('&')
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next()?;
                    let val = parts.next().unwrap_or("");
                    Some((key, val))
                })
                .collect();
            let kept: Vec<String> = pairs
                .iter()
                .filter(|(k, _)| !k.to_lowercase().starts_with("utm_"))
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            let line = kept.join("&");
            if line == query {
                return text.to_string();
            }
            let mut base = parsed.clone();
            base.set_query(if line.is_empty() { None } else { Some(&line) });
            base.to_string()
        }
        Err(_) => text.to_string(),
    }
}

/// Return decoded URL for display.
pub fn presentation(text: &str) -> String {
    let text = text.replace('+', "%2B");
    let bytes = percent_decode_bytes(text.as_bytes());
    String::from_utf8(bytes).unwrap_or(text)
}

/// Decode percent-encoded bytes.
fn percent_decode_bytes(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'%' && i + 2 < input.len() {
            let hi = input[i + 1];
            let lo = input[i + 2];
            if let (Some(h), Some(l)) = (hex_val(hi), hex_val(lo)) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(input[i]);
        i += 1;
    }
    out
}

/// Convert hex digit to value.
fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Remove utm fragments from text.
pub fn prune(text: &str) -> String {
    let re = Regex::new(r"(\?utm_[^\s\)\]]+|&utm_[^\s\)\]]+)").unwrap();
    re.replace_all(text, "").to_string()
}

/// Remove tracking parameters from text URLs.
pub fn clean(text: &str) -> String {
    let pattern = Regex::new(r"https?://[^\s\)\]]+").unwrap();
    let value = pattern
        .replace_all(text, |caps: &regex::Captures| trim(&caps[0]))
        .to_string();
    let value = prune(&value);
    let chars: Vec<char> = value.chars().collect();
    let mut result = String::with_capacity(value.len());
    let paren_re =
        Regex::new(r"^[ \t]*\((?:https?://[^\s\)]+|[A-Za-z0-9.\-]+\.[A-Za-z]{2,}[^\s\)]*)\)")
            .unwrap();
    let mut i = 0;
    while i < chars.len() {
        let prev = if i > 0 { chars[i - 1] } else { '\0' };
        if prev != ']' && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == '(') {
            let rest = &value[value.char_indices().nth(i).unwrap().0..];
            if let Some(m) = paren_re.find(rest) {
                i += m.as_str().chars().count();
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Replace outer italic asterisks with underscores when bold ends the span.
pub fn underscorify(text: &str) -> String {
    let bold_re = Regex::new(r"^([^*]+?)\*\*([^\n]*?)\*\*$").unwrap();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(text.len());
    let mut i = 0;
    while i < len {
        if chars[i] == '*'
            && (i == 0 || chars[i - 1] != '*')
            && (i + 1 >= len || chars[i + 1] != '*')
        {
            if let Some(end) = find_italic_bold_end(&chars, i + 1) {
                let inner: String = chars[i + 1..end + 2].iter().collect();
                if !inner.contains('\n') {
                    if let Some(caps) = bold_re.captures(&inner) {
                        result.push('_');
                        result.push_str(&caps[1]);
                        result.push_str("**");
                        result.push_str(&caps[2]);
                        result.push_str("**_");
                        i = end + 3;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Find char-index where *content*** ends.
fn find_italic_bold_end(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    let mut i = start;
    while i + 2 < len {
        if chars[i] == '*'
            && chars[i + 1] == '*'
            && chars[i + 2] == '*'
            && (i + 3 >= len || chars[i + 3] != '*')
        {
            return Some(i);
        }
        if chars[i] == '\n' {
            return None;
        }
        i += 1;
    }
    None
}

/// Return cleaned source title.
pub fn label(item: &dyn Sourced, name: &str) -> String {
    let raw = decode(&item.title().trim().replace(r"\s+", " "));
    let re = Regex::new(r"\s+").unwrap();
    let raw = re.replace_all(&raw, " ").to_string();
    let link = trim(item.url());
    let host = domain(&link);
    let text = if name == "parallel" && raw.to_lowercase() == "fetched web page" {
        if host.is_empty() {
            link.clone()
        } else {
            host.clone()
        }
    } else {
        raw
    };
    if text.is_empty() {
        if host.is_empty() {
            link
        } else {
            host
        }
    } else {
        text
    }
}

/// Return cleaned excerpt text.
pub fn excerpt(text: &str) -> String {
    let text = decode(text.trim());
    let re = Regex::new(r"\s+").unwrap();
    let text = re.replace_all(&text, " ").to_string();
    let size = 220;
    if text.len() > size {
        let boundary = text
            .char_indices()
            .take_while(|(i, _)| *i < size - 1)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(size - 1);
        format!("{}...", &text[..boundary])
    } else {
        text
    }
}

/// Return domain from URL string.
fn domain(text: &str) -> String {
    match url::Url::parse(text) {
        Ok(parsed) => match parsed.host_str() {
            Some(host) => host.replace("www.", ""),
            None => String::new(),
        },
        Err(_) => String::new(),
    }
}

/// Render markdown to HTML.
pub fn markdown(text: &str) -> String {
    let mut options = comrak::Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.render.r#unsafe = true;
    comrak::markdown_to_html(text, &options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_text_escape_converts_angle_brackets() {
        let mut rng = ids::ids(21001);
        let value = ids::cyrillic(&mut rng, 6);
        let text = format!("<{}>", value);
        let result = escape(&text);
        assert!(
            result.contains("&lt;") && result.contains("&gt;"),
            "Angle brackets were not escaped"
        );
    }

    #[test]
    fn the_text_heading_uppercases_initial_letter() {
        let mut rng = ids::ids(21003);
        let value = ids::cyrillic(&mut rng, 6);
        let result = heading(&value);
        let first: String = value.chars().next().unwrap().to_uppercase().collect();
        assert!(
            result.starts_with(&first),
            "Initial letter was not uppercased"
        );
    }

    #[test]
    fn the_text_normalize_adds_blank_line_before_list() {
        let mut rng = ids::ids(21005);
        let head = ids::hiragana(&mut rng, 6);
        let tail = ids::cyrillic(&mut rng, 4);
        let text = format!("**{}**\n* {}", head, tail);
        let result = normalize(&text);
        assert!(
            result.contains("**\n\n* "),
            "Normalize did not add blank line before list"
        );
    }

    #[test]
    fn the_text_normalize_preserves_existing_blank_lines() {
        let mut rng = ids::ids(21007);
        let head = ids::hiragana(&mut rng, 6);
        let tail = ids::cyrillic(&mut rng, 4);
        let text = format!("**{}**\n\n* {}", head, tail);
        let result = normalize(&text);
        assert_eq!(text, result, "Normalize modified already correct text");
    }

    #[test]
    fn the_text_tablerows_removes_blank_line() {
        let mut rng = ids::ids(21009);
        let alpha = ids::greek(&mut rng, 4);
        let beta = ids::armenian(&mut rng, 4);
        let gamma = ids::arabic(&mut rng, 4);
        let delta = ids::hebrew(&mut rng, 4);
        let head = format!("| {} | {} |", alpha, beta);
        let separator = "| --- | --- |";
        let row = format!("| {} | {} |", gamma, delta);
        let tail = format!("| {} | {} |", delta, gamma);
        let text = format!("{}\n{}\n{}\n\n{}", head, separator, row, tail);
        let goal = format!("{}\n{}\n{}\n{}", head, separator, row, tail);
        let result = tablerows(&text);
        assert_eq!(goal, result, "Table rows were not normalized");
    }

    #[test]
    fn the_text_tablecite_moves_citations() {
        let mut rng = ids::ids(21011);
        let head = ids::greek(&mut rng, 4);
        let tail = ids::armenian(&mut rng, 4);
        let link = format!("https://example.com/{}", ids::uuid(&mut rng));
        let text = format!(
            "| {} | {} |\n|---|---|\n| {} | 21 |[[1]]({})",
            head, tail, tail, link
        );
        let goal = format!(
            "| {} | {} |\n|---|---|\n| {} | 21 [[1]]({}) |",
            head, tail, tail, link
        );
        let result = tablecite(&text);
        assert_eq!(
            goal, result,
            "Table citations were not moved into last cell"
        );
    }

    #[test]
    fn the_text_tablepipe_adds_trailing_pipe() {
        let mut rng = ids::ids(21013);
        let head = ids::greek(&mut rng, 4);
        let tail = ids::armenian(&mut rng, 4);
        let text = format!("| {} | {} |\n|---|---|\n| {} | {}", head, tail, tail, head);
        let goal = format!(
            "| {} | {} |\n|---|---|\n| {} | {} |",
            head, tail, tail, head
        );
        let result = tablepipe(&text);
        assert_eq!(goal, result, "Table rows were not terminated with pipe");
    }

    #[test]
    fn the_text_tablelead_strips_list_marker() {
        let mut rng = ids::ids(21015);
        let head = ids::greek(&mut rng, 4);
        let tail = ids::armenian(&mut rng, 4);
        let text = format!(
            "- | {} | {} |\n|---|---|\n| {} | {} |",
            head, tail, tail, head
        );
        let goal = format!(
            "| {} | {} |\n|---|---|\n| {} | {} |",
            head, tail, tail, head
        );
        let result = tablelead(&text);
        assert_eq!(goal, result, "Table lead marker was not removed");
    }

    #[test]
    fn the_text_tablelead_preserves_list_items() {
        let mut rng = ids::ids(21017);
        let head = ids::cyrillic(&mut rng, 6);
        let left = ids::hiragana(&mut rng, 6);
        let right = ids::greek(&mut rng, 6);
        let text = format!("**{}:**\n- {}\n- {}", head, left, right);
        let result = tablelead(&text);
        assert_eq!(
            text, result,
            "Tablelead stripped markers from non-table list items"
        );
    }

    #[test]
    fn the_text_listify_converts_numbered_prompts() {
        let mut rng = ids::ids(21019);
        let head = ids::cyrillic(&mut rng, 6);
        let left = ids::hiragana(&mut rng, 6);
        let right = ids::latin(&mut rng, 6);
        let one = 1 + ids::digit(&mut rng, 8);
        let two = one + 1 + ids::digit(&mut rng, 8);
        let text = format!("{} {}) {} {}. {}", head, one, left, two, right);
        let result = listify(&text);
        assert!(
            result.contains(&format!("\n{}. ", one)) && result.contains(&format!("\n{}. ", two)),
            "Numbered prompts were not converted"
        );
    }

    #[test]
    fn the_text_rule_replaces_separators() {
        let mut rng = ids::ids(21021);
        let head = ids::cyrillic(&mut rng, 6);
        let tail = ids::hiragana(&mut rng, 6);
        let text = format!("{}\n---\n{}", head, tail);
        let result = rule(&text);
        assert!(result.contains("<hr />"), "Rule did not convert separator");
    }

    #[test]
    fn the_text_nested_converts_single_space_indent() {
        let mut rng = ids::ids(21023);
        let mark = ids::uuid(&mut rng);
        let text = format!(
            "* **\u{89aa}-{}:**\n * **\u{5b50}\u{8981}\u{7d20}:** \u{5185}\u{5bb9}",
            mark
        );
        let result = nested(&text);
        assert!(
            result.contains("    * "),
            "Nested did not convert single space to four spaces"
        );
    }

    #[test]
    fn the_text_underscorify_rewrites_nested_bold_in_bullets() {
        let mut rng = ids::ids(21025);
        let mark = ids::uuid(&mut rng);
        let text = format!(
            "- *{} **to be fed*** \u{2014} \u{03a4}\u{03bf} \u{03c0}\u{03b1}\u{03b9}\u{03b4}\u{03af}",
            mark
        );
        let goal = format!(
            "- _{} **to be fed**_ \u{2014} \u{03a4}\u{03bf} \u{03c0}\u{03b1}\u{03b9}\u{03b4}\u{03af}",
            mark
        );
        let result = underscorify(&text);
        assert_eq!(
            goal, result,
            "underscorify failed to rewrite nested bold in bullet"
        );
    }

    #[test]
    fn the_text_clean_strips_utm_parameters() {
        let mut rng = ids::ids(21027);
        let slug = ids::cyrillic(&mut rng, 5);
        let number = ids::digit(&mut rng, 1000);
        let link = format!(
            "https://example.com/{}?utm_source=valyu.ai&utm_medium=referral&x={}",
            number,
            ids::digit(&mut rng, 9)
        );
        let text = format!("Sources\n1. {}\n2. {}", link, slug);
        let result = clean(&text);
        assert!(
            !result.contains("utm_source"),
            "utm parameters were not stripped from document"
        );
    }

    #[test]
    fn the_text_trim_preserves_non_utm_params() {
        let mut rng = ids::ids(21029);
        let key = ids::greek(&mut rng, 4);
        let val = ids::armenian(&mut rng, 4);
        let number = ids::digit(&mut rng, 1000);
        let link = format!(
            "https://example.com/{}?{}={}&sig={}",
            number,
            key,
            val,
            ids::digit(&mut rng, 1000)
        );
        let result = trim(&link);
        assert_eq!(
            link, result,
            "Image URL was changed despite missing utm parameters"
        );
    }

    #[test]
    fn the_text_outline_renders_nested_list() {
        let mut rng = ids::ids(21031);
        let head = ids::cyrillic(&mut rng, 6);
        let tail = ids::hiragana(&mut rng, 6);
        let items = vec![Question {
            scope: head.clone(),
            details: vec![Question {
                scope: tail.clone(),
                details: Vec::new(),
            }],
        }];
        let result = outline(&items);
        assert!(
            result.contains("<ol>") && result.contains(&escape(&head)),
            "Outline did not render nested list"
        );
    }

    #[test]
    fn the_text_decode_converts_entities() {
        let mut rng = ids::ids(21033);
        let _mark = ids::cyrillic(&mut rng, 4);
        let text = "&gt;&gt;&gt; df = pd.DataFrame({&#x27;A&#x27; : ['test']})";
        let result = decode(text);
        assert!(
            result.contains(">>> df = pd.DataFrame({'A' : ['test']})"),
            "HTML entities were not decoded"
        );
    }

    #[test]
    fn the_text_excerpt_truncates_long_text() {
        let mut rng = ids::ids(21035);
        let text: String = (0..250)
            .map(|_| ids::cyrillic(&mut rng, 1))
            .collect::<Vec<_>>()
            .join("");
        let result = excerpt(&text);
        assert!(
            result.ends_with("..."),
            "Long text was not truncated with ellipsis"
        );
    }

    #[test]
    fn the_text_anchors_assigns_sequential_ids() {
        let mut rng = ids::ids(21037);
        let head = ids::greek(&mut rng, 6);
        let tail = ids::cyrillic(&mut rng, 6);
        let html = format!("<h2>{}</h2><h3>{}</h3>", head, tail);
        let data = anchors(&html);
        assert_eq!(
            2,
            data.items.len(),
            "Anchors did not produce correct number of toc items"
        );
    }

    #[test]
    fn the_text_toc_renders_links() {
        let mut rng = ids::ids(21039);
        let head = ids::greek(&mut rng, 6);
        let items = vec![TocItem {
            text: head.clone(),
            id: "section-1".to_string(),
            level: 2,
        }];
        let result = toc(&items);
        assert!(
            result.contains("ref-link") && result.contains(&escape(&head)),
            "Table of contents did not render links"
        );
    }
}
