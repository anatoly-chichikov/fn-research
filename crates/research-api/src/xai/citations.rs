use std::collections::HashSet;

use crate::link::{self, Linkable};

/// Return UTF-16 offset for codepoint index.
pub fn index(text: &str, point: usize) -> usize {
    let limit = text.chars().count();
    let point = point.min(limit);
    text.chars()
        .take(point)
        .map(|c| c.len_utf16())
        .sum::<usize>()
}

/// Return unique URLs from text list.
pub fn links(items: &[String]) -> Vec<String> {
    let policy = link::make();
    let mut seen: HashSet<String> = HashSet::new();
    let mut list = Vec::new();
    for item in items {
        let urls = policy.links(item);
        for url in urls {
            if !seen.contains(&url) {
                seen.insert(url.clone());
                list.push(url);
            }
        }
    }
    list
}

/// Citation data extracted from response.
#[derive(Debug, Clone)]
pub struct Citation {
    /// End index in text.
    pub end: usize,
    /// Citation identifier.
    pub id: String,
    /// Citation URL.
    pub url: String,
    /// Citation title.
    pub title: String,
}

/// Ordered citation result.
pub struct Ordered {
    /// Renumbered text.
    pub text: String,
    /// Ordered URL list.
    pub list: Vec<String>,
    /// URL to title mapping.
    pub name: std::collections::HashMap<String, String>,
}

/// Return renumbered citations and ordered URLs.
pub fn order(text: &str, marks: &[Citation]) -> Ordered {
    let rule = regex::Regex::new(r"\[\[(\d+)\]\]\((https?://[^)]+)\)").unwrap();
    let mut seen = HashSet::new();
    let mut list: Vec<String> = Vec::new();
    let mut map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for caps in rule.captures_iter(text) {
        let url = caps[2].to_string();
        if !seen.contains(&url) {
            seen.insert(url.clone());
            let num = list.len() + 1;
            list.push(url.clone());
            map.insert(url, num);
        }
    }
    let result = if !map.is_empty() {
        rule.replace_all(text, |caps: &regex::Captures| {
            let url = &caps[2];
            let num = map.get(url).copied().unwrap_or(1);
            format!("[[{}]]({})", num, url)
        })
        .to_string()
    } else {
        text.to_string()
    };
    let mut name: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for item in marks {
        let url = &item.url;
        let title = &item.title;
        if !url.is_empty() && !name.contains_key(url) && !title.is_empty() {
            name.insert(url.clone(), title.clone());
        }
    }
    Ordered {
        text: result,
        list,
        name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_citations_index_maps_codepoints() {
        let mut rng = ids::ids(18305);
        let head = ids::latin(&mut rng, 1);
        let tail = ids::cyrillic(&mut rng, 1);
        let face = '\u{1F600}';
        let text = format!("{}{}{}", head, face, tail);
        let spot = index(&text, 2);
        assert_eq!(3, spot, "index did not map codepoints");
    }

    #[test]
    fn the_py_client_renumbers_citations() {
        let mut rng = ids::ids(18321);
        let url1 = format!("https://example.com/{}", ids::ascii(&mut rng, 6));
        let url2 = format!("https://example.org/{}", ids::ascii(&mut rng, 6));
        let head = ids::greek(&mut rng, 4);
        let text = format!(
            "{} [[1]]({}) x [[1]]({}) y [[1]]({})",
            head, url1, url2, url1
        );
        let name = ids::greek(&mut rng, 5);
        let marks = vec![
            Citation {
                end: 0,
                id: String::new(),
                url: url1.clone(),
                title: name.clone(),
            },
            Citation {
                end: 0,
                id: String::new(),
                url: url2.clone(),
                title: String::new(),
            },
        ];
        let data = order(&text, &marks);
        let check = data.text.contains(&format!("[[1]]({})", url1))
            && data.text.contains(&format!("[[2]]({})", url2))
            && data.list == vec![url1.clone(), url2]
            && data.name.get(&url1).map(|s| s.as_str()) == Some(&name);
        assert!(check, "citations were not renumbered");
    }
}
