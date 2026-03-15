use std::collections::HashMap;

use regex::Regex;

use super::text;

/// Extract reference URLs from text.
pub fn references(text: &str) -> HashMap<u32, String> {
    let section = Regex::new(r"(?s)##\s*References\s*\n(.*?)(?:\n##|\z)").unwrap();
    let body = section
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or("");
    let line_re = Regex::new(r"(\d+)\.\s+.*?(https?://\S+)").unwrap();
    let mut map = HashMap::new();
    for line in body.lines() {
        if let Some(caps) = line_re.captures(line) {
            let num: u32 = caps[1].parse().unwrap_or(0);
            let url = caps[2].to_string();
            map.insert(num, url);
        }
    }
    map
}

/// Citation result with processed text, collected URLs, and marker map.
pub struct CitationData {
    /// Processed text with citation markers.
    pub text: String,
    /// Collected URLs.
    pub urls: Vec<String>,
    /// Marker to HTML mapping.
    pub marks: HashMap<String, String>,
}

/// Convert [N] to links.
pub fn citations(text: &str, _sources: &[research_domain::result::CitationSource]) -> CitationData {
    let refs = references(text);
    let mut hold: Vec<String> = Vec::new();
    let mut mark: HashMap<String, String> = HashMap::new();
    let mut note: u32 = 0;
    let url_pattern = r"https?://(?:[^()\s\[\]]+|\([^\s\[\]]*\))+";
    let double_bracket = Regex::new(&format!(r"\[\[(\d+)\]\]\(({})\)", url_pattern)).unwrap();
    let single_bracket = Regex::new(&format!(r"\[(\d+)\]\(({})\)", url_pattern)).unwrap();
    let bare = Regex::new(r"\[(\d+)\]").unwrap();
    let push = |num: u32,
                link: &str,
                hold: &mut Vec<String>,
                mark: &mut HashMap<String, String>,
                note: &mut u32|
     -> String {
        let token = format!("@@CITE{}@@", note);
        *note += 1;
        mark.insert(
            token.clone(),
            format!(
                "<sup class=\"cite\"><a href=\"{}\" class=\"cite\" target=\"_blank\">[{}]</a></sup>",
                link, num
            ),
        );
        if !hold.contains(&link.to_string()) {
            hold.push(link.to_string());
        }
        token
    };
    let stage = double_bracket
        .replace_all(text, |caps: &regex::Captures| {
            let num: u32 = caps[1].parse().unwrap_or(0);
            let link = text::trim(&caps[2]);
            if link.is_empty() {
                caps[0].to_string()
            } else {
                push(num, &link, &mut hold, &mut mark, &mut note)
            }
        })
        .to_string();
    let stage = single_bracket
        .replace_all(&stage, |caps: &regex::Captures| {
            let num: u32 = caps[1].parse().unwrap_or(0);
            let link = text::trim(&caps[2]);
            if link.is_empty() {
                caps[0].to_string()
            } else {
                push(num, &link, &mut hold, &mut mark, &mut note)
            }
        })
        .to_string();
    let value = bare
        .replace_all(&stage, |caps: &regex::Captures| {
            let num: u32 = caps[1].parse().unwrap_or(0);
            let link = refs.get(&num).map(|l| text::trim(l)).unwrap_or_default();
            if link.is_empty() {
                caps[0].to_string()
            } else {
                push(num, &link, &mut hold, &mut mark, &mut note)
            }
        })
        .to_string();
    CitationData {
        text: value,
        urls: hold,
        marks: mark,
    }
}

/// Remove trailing sources section.
pub fn strip(text: &str) -> String {
    let rows: Vec<&str> = text.split('\n').collect();
    let heading_re = Regex::new(r"^#+\s*").unwrap();
    let mut idx: i32 = -1;
    for (i, row) in rows.iter().enumerate() {
        let line = row.trim();
        let label = heading_re.replace(line, "").to_lowercase();
        let is_heading = line.starts_with('#');
        let is_source = matches!(
            label.as_str(),
            "source" | "sources" | "reference" | "references"
        );
        if is_heading && is_source {
            idx = i as i32;
        }
    }
    let value = if idx >= 0 {
        let idx = idx as usize;
        let tail: Vec<&str> = rows[idx + 1..].to_vec();
        let later = tail.iter().any(|line| line.trim().starts_with('#'));
        let body = tail.join("\n");
        let has_urls = body.contains("https://") || body.contains("http://");
        if !later && has_urls {
            rows[..idx].join("\n")
        } else {
            text.to_string()
        }
    } else {
        text.to_string()
    };
    let prepared = Regex::new(r"\\n---\\n\*Prepared using.*?\*").unwrap();
    prepared.replace_all(&value, "").to_string()
}

/// Add columns classes to tables.
pub fn tables(text: &str) -> String {
    let table_re = Regex::new(r"(?s)<table>.*?</table>").unwrap();
    let th_re = Regex::new(r"<th[^>]*>").unwrap();
    let td_re = Regex::new(r"<td[^>]*>").unwrap();
    table_re
        .replace_all(text, |caps: &regex::Captures| {
            let table = &caps[0];
            let head_end = table.find("</thead>").or_else(|| table.find("</tr>"));
            let head_end = head_end.unwrap_or(table.len());
            let head = &table[..head_end];
            let cols = th_re.find_iter(head).count();
            let cols = if cols > 0 {
                cols
            } else {
                td_re.find_iter(head).count()
            };
            table.replace("<table>", &format!("<table class=\"cols-{}\">", cols))
        })
        .to_string()
}

/// Add hanging indent spans for code blocks.
pub fn codeindent(text: &str) -> String {
    let code_re = Regex::new(r"(?s)<pre><code>(.*?)</code></pre>").unwrap();
    code_re
        .replace_all(text, |caps: &regex::Captures| {
            let body = &caps[1];
            let lines: Vec<&str> = body.split('\n').collect();
            let rows: Vec<String> = lines
                .iter()
                .map(|line| {
                    if line.trim().is_empty() {
                        line.to_string()
                    } else {
                        let pad = line.len() - line.trim_start().len();
                        let hang = 2;
                        let indent = pad + hang;
                        let value = line.trim_start();
                        format!(
                            "<span class=\"code-line\" style=\"padding-left: {}ch; \
                             text-indent: -{}ch; display: block;\">{}</span>",
                            indent, hang, value
                        )
                    }
                })
                .collect();
            format!("<pre><code>{}</code></pre>", rows.join(""))
        })
        .to_string()
}

/// Replace star ratings with fractions.
pub fn stars(text: &str) -> String {
    let re = Regex::new(r"[\u{2605}\u{2606}]+").unwrap();
    re.replace_all(text, |caps: &regex::Captures| {
        let value = &caps[0];
        let size = value.chars().count();
        let sum = value.chars().filter(|c| *c == '\u{2605}').count();
        format!("{}/{}", sum, size)
    })
    .to_string()
}

/// Unescape encoded backslashes in HTML.
pub fn backslash(text: &str) -> String {
    text.replace("&amp;#92;", "&#92;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_citations_references_extract_mapping() {
        let mut rng = ids::ids(22001);
        let mark = ids::uuid(&mut rng);
        let text = format!(
            "## References\n\n1. \u{4e00} https://a.jp/{}\n2. \u{4e8c} https://b.jp/{}",
            mark, mark
        );
        let refs = references(&text);
        assert_eq!(2, refs.len(), "References did not extract all entries");
    }

    #[test]
    fn the_citations_convert_references() {
        let mut rng = ids::ids(22003);
        let mark = ids::uuid(&mut rng);
        let head = ids::hiragana(&mut rng, 5);
        let host = ids::ascii(&mut rng, 6);
        let link = format!("https://{}.com/{}", host, mark);
        let text = format!(
            "{}-{} [1]\n\n## References\n\n1. {} {}",
            head, mark, head, link
        );
        let data = citations(&text, &[]);
        let seen = data.text.contains("@@CITE") && data.marks.values().any(|v| v.contains(&link));
        assert!(seen, "Citations did not create link from reference");
    }

    #[test]
    fn the_citations_extract_urls() {
        let mut rng = ids::ids(22005);
        let mark = ids::uuid(&mut rng);
        let text = format!(
            "\u{53c2}\u{7167} [1]\n\n## References\n\n1. \u{30bd}\u{30fc}\u{30b9} https://test.jp/{}",
            mark
        );
        let data = citations(&text, &[]);
        assert_eq!(1, data.urls.len(), "Citations did not extract URL");
    }

    #[test]
    fn the_citations_handle_parentheses_in_links() {
        let mut rng = ids::ids(22007);
        let head = ids::cyrillic(&mut rng, 4);
        let host = ids::ascii(&mut rng, 6);
        let left = ids::cyrillic(&mut rng, 5);
        let right = ids::cyrillic(&mut rng, 5);
        let tail = ids::cyrillic(&mut rng, 4);
        let link = format!(
            "https://{}.org/wiki/{}_({}-{})/{}_{}",
            host, head, left, right, tail, left
        );
        let text = format!("{} [[1]]({})", head, link);
        let data = citations(&text, &[]);
        let seen = data.text.contains("@@CITE") && data.marks.values().any(|v| v.contains(&link));
        assert!(seen, "Citations did not preserve parentheses link");
    }

    #[test]
    fn the_citations_strip_removes_sources_section() {
        let mut rng = ids::ids(22009);
        let head = ids::cyrillic(&mut rng, 6);
        let number = ids::digit(&mut rng, 1000);
        let link = format!("https://example.com/{}", number);
        let text = format!("{}\n\n## Sources\n1. {}\n2. {}", head, link, link);
        let result = strip(&text);
        assert!(
            !result.contains("Sources"),
            "Sources section was not stripped"
        );
    }

    #[test]
    fn the_citations_strip_keeps_sources_without_links() {
        let mut rng = ids::ids(22011);
        let head = ids::cyrillic(&mut rng, 6);
        let note = ids::greek(&mut rng, 5);
        let text = format!("{}\n\n## Sources\n1. {}\n2. {}", head, note, note);
        let result = strip(&text);
        assert!(
            result.contains("Sources"),
            "Sources section was removed without links"
        );
    }

    #[test]
    fn the_citations_strip_keeps_sources_when_not_last_section() {
        let mut rng = ids::ids(22013);
        let head = ids::cyrillic(&mut rng, 6);
        let note = ids::greek(&mut rng, 5);
        let number = ids::digit(&mut rng, 1000);
        let url = format!("https://example.com/{}", number);
        let text = format!("{}\n\n## Sources\n1. {}\n\n## Next\n{}", head, url, note);
        let result = strip(&text);
        assert!(
            result.contains("Sources"),
            "Sources section was removed before end"
        );
    }

    #[test]
    fn the_citations_tables_adds_column_class() {
        let mut rng = ids::ids(22015);
        let head = ids::cyrillic(&mut rng, 4);
        let body = ids::hiragana(&mut rng, 4);
        let html = format!(
            "<table><thead><tr><th>{}</th><th>{}</th></tr></thead>\
             <tbody><tr><td>{}</td><td>{}</td></tr></tbody></table>",
            head, head, body, body
        );
        let result = tables(&html);
        assert!(
            result.contains("class=\"cols-3\""),
            "Tables did not add column class"
        );
    }

    #[test]
    fn the_citations_stars_replaces_ratings() {
        let mut rng = ids::ids(22017);
        let _mark = ids::cyrillic(&mut rng, 4);
        let text = "\u{2605}\u{2605}\u{2605}\u{2606}\u{2606}";
        let result = stars(text);
        assert_eq!("3/5", result, "Star ratings were not converted");
    }

    #[test]
    fn the_citations_backslash_unescapes_encoded() {
        let mut rng = ids::ids(22019);
        let _mark = ids::cyrillic(&mut rng, 4);
        let text = "path&amp;#92;file";
        let result = backslash(text);
        assert!(result.contains("&#92;"), "Backslash was not unescaped");
    }
}
