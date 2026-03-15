/// Object that can parse research brief.
pub trait Briefed {
    /// Return brief parts.
    fn parts(&self, text: &str) -> BriefParts;
}

/// Parsed brief item with depth.
#[derive(Debug, Clone, PartialEq)]
pub struct BriefItem {
    /// Nesting depth (1-3).
    pub depth: usize,
    /// Item text.
    pub text: String,
}

/// Parsed brief structure.
#[derive(Debug, Clone)]
pub struct BriefParts {
    /// Lines before the research marker.
    pub head: Vec<String>,
    /// Parsed hierarchical items.
    pub items: Vec<BriefItem>,
    /// Last non-blank line of head.
    pub top: String,
}

/// Brief parser.
pub struct Brief {
    mark: String,
}

impl Brief {
    /// Create brief parser with marker.
    pub fn new(mark: &str) -> Self {
        Self {
            mark: mark.to_string(),
        }
    }
}

impl Briefed for Brief {
    fn parts(&self, text: &str) -> BriefParts {
        let lines: Vec<&str> = text.lines().collect();
        let spot = lines.iter().position(|line| line.trim() == self.mark);
        let head: Vec<String> = if let Some(idx) = spot {
            lines[..idx].iter().map(|s| s.to_string()).collect()
        } else {
            lines.iter().map(|s| s.to_string()).collect()
        };
        let tail: Vec<&str> = if let Some(idx) = spot {
            lines[idx + 1..].to_vec()
        } else {
            Vec::new()
        };
        let num_re = regex::Regex::new(r"^(\d+(?:\.\d+)*)[.)]\s+(.+)$").unwrap();
        let bul_re = regex::Regex::new(r"^[*+\-]\s+(.+)$").unwrap();
        let mut list: Vec<BriefItem> = Vec::new();
        for raw in &tail {
            let tabs = raw.chars().take_while(|c| *c == '\t').count();
            let row = raw.replace('\t', " ");
            let trim = row.trim_start();
            let pad = row.len() - trim.len();
            let num_match = num_re.captures(trim);
            let bul_match = bul_re.captures(trim);
            let plain = num_match.is_none()
                && bul_match.is_none()
                && !trim.is_empty()
                && (tabs > 0 || pad == 0);
            let item_text = if let Some(ref caps) = num_match {
                Some(caps[2].to_string())
            } else if let Some(ref caps) = bul_match {
                Some(caps[1].to_string())
            } else if plain {
                Some(trim.to_string())
            } else {
                None
            };
            let base = if let Some(ref caps) = num_match {
                let dots = caps[1].split('.').count();
                Some(dots)
            } else if bul_match.is_some() {
                Some(1 + pad / 2)
            } else if plain {
                Some(1 + tabs)
            } else {
                None
            };
            let depth = if let Some(ref caps) = num_match {
                let dots = caps[1].split('.').count();
                if pad > 0 {
                    Some(1 + pad / 4)
                } else {
                    Some(dots)
                }
            } else {
                base
            };
            let depth = depth.map(|d| d.clamp(1, 3));
            let text = item_text.unwrap_or_default().trim().to_string();
            let item = if let Some(d) = depth {
                if !text.is_empty() {
                    Some(BriefItem { depth: d, text })
                } else {
                    None
                }
            } else {
                None
            };
            let line = row.trim().to_string();
            if line.is_empty() {
                continue;
            }
            if let Some(entry) = item {
                list.push(entry);
            } else if let Some(last) = list.last_mut() {
                last.text.push(' ');
                last.text.push_str(&line);
            }
        }
        let top = head.iter().fold(String::new(), |acc, line| {
            if line.trim().is_empty() {
                acc
            } else {
                line.trim().to_string()
            }
        });
        BriefParts {
            head,
            items: list,
            top,
        }
    }
}

/// Return brief parser.
pub fn make() -> Brief {
    Brief::new("Research:")
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_brief_parses_items() {
        let mut rng = ids::ids(18307);
        let head = ids::cyrillic(&mut rng, 5);
        let left = ids::greek(&mut rng, 4);
        let right = ids::armenian(&mut rng, 4);
        let text = format!("{}\n\nResearch:\n1. {}\n2. {}", head, left, right);
        let item = make();
        let info = item.parts(&text);
        let expect = vec![
            BriefItem {
                depth: 1,
                text: left,
            },
            BriefItem {
                depth: 1,
                text: right,
            },
        ];
        assert_eq!(expect, info.items, "brief did not parse items");
    }

    #[test]
    fn the_brief_parses_nested_items() {
        let mut rng = ids::ids(18308);
        let head = ids::cyrillic(&mut rng, 5);
        let alpha = ids::greek(&mut rng, 4);
        let beta = ids::armenian(&mut rng, 4);
        let gamma = ids::arabic(&mut rng, 4);
        let delta = ids::hebrew(&mut rng, 4);
        let pad = "    ";
        let deep = "        ";
        let text = format!(
            "{}\n\nResearch:\n1. {}\n{}1. {}\n{}1. {}\n2. {}",
            head, alpha, pad, beta, deep, gamma, delta
        );
        let item = make();
        let info = item.parts(&text);
        let expect = vec![
            BriefItem {
                depth: 1,
                text: alpha,
            },
            BriefItem {
                depth: 2,
                text: beta,
            },
            BriefItem {
                depth: 3,
                text: gamma,
            },
            BriefItem {
                depth: 1,
                text: delta,
            },
        ];
        assert_eq!(expect, info.items, "nested brief items were not parsed");
    }

    #[test]
    fn the_brief_parses_tab_indented_items() {
        let mut rng = ids::ids(18309);
        let head = ids::cyrillic(&mut rng, 5);
        let left = ids::greek(&mut rng, 4);
        let child = ids::armenian(&mut rng, 4);
        let right = ids::hebrew(&mut rng, 4);
        let text = format!("{}\n\nResearch:\n{}\n\t{}\n{}", head, left, child, right);
        let item = make();
        let info = item.parts(&text);
        let expect = vec![
            BriefItem {
                depth: 1,
                text: left,
            },
            BriefItem {
                depth: 2,
                text: child,
            },
            BriefItem {
                depth: 1,
                text: right,
            },
        ];
        assert_eq!(
            expect, info.items,
            "tab-indented brief items were not parsed"
        );
    }

    #[test]
    fn the_brief_parses_double_tab_items() {
        let mut rng = ids::ids(18310);
        let head = ids::cyrillic(&mut rng, 5);
        let alpha = ids::greek(&mut rng, 4);
        let beta = ids::armenian(&mut rng, 4);
        let gamma = ids::arabic(&mut rng, 4);
        let delta = ids::hebrew(&mut rng, 4);
        let text = format!(
            "{}\n\nResearch:\n{}\n\t{}\n\t\t{}\n{}",
            head, alpha, beta, gamma, delta
        );
        let item = make();
        let info = item.parts(&text);
        let expect = vec![
            BriefItem {
                depth: 1,
                text: alpha,
            },
            BriefItem {
                depth: 2,
                text: beta,
            },
            BriefItem {
                depth: 3,
                text: gamma,
            },
            BriefItem {
                depth: 1,
                text: delta,
            },
        ];
        assert_eq!(expect, info.items, "double-tab brief items were not parsed");
    }
}
