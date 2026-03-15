use regex::Regex;

/// Object that can normalize links.
pub trait Linkable {
    /// Return URL without tracking params.
    fn clean(&self, text: &str) -> String;
    /// Return text with tracking params removed.
    fn strip(&self, text: &str) -> String;
    /// Return domain from URL string.
    fn domain(&self, text: &str) -> String;
    /// Return URLs from text.
    fn links(&self, text: &str) -> Vec<String>;
}

/// Link normalization policy.
pub struct Links {
    pat: Regex,
    utm: Regex,
}

impl Links {
    /// Create link policy from patterns.
    pub fn new(pat: Regex, utm: Regex) -> Self {
        Self { pat, utm }
    }
}

impl Linkable for Links {
    fn clean(&self, text: &str) -> String {
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

    fn strip(&self, text: &str) -> String {
        let items: Vec<&str> = self.pat.find_iter(text).map(|m| m.as_str()).collect();
        let mut note = text.to_string();
        for found in items {
            let cleaned = self.clean(found);
            note = note.replace(found, &cleaned);
        }
        let line = self.utm.replace_all(&note, "").to_string();
        line
    }

    fn domain(&self, text: &str) -> String {
        match url::Url::parse(text) {
            Ok(parsed) => match parsed.host_str() {
                Some(host) => host.replace("www.", ""),
                None => String::new(),
            },
            Err(_) => String::new(),
        }
    }

    fn links(&self, text: &str) -> Vec<String> {
        self.pat
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

/// Return default link policy.
pub fn make() -> Links {
    Links::new(
        Regex::new(r"https?://[^\s\)\]]+").unwrap(),
        Regex::new(r"(\?utm_[^\s\)\]]+|&utm_[^\s\)\]]+)").unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_link_removes_utm_params() {
        let mut rng = ids::ids(18001);
        let host = ids::ascii(&mut rng, 6);
        let path = ids::cyrillic(&mut rng, 4);
        let token = ids::greek(&mut rng, 4);
        let mark = ids::digit(&mut rng, 9);
        let raw = format!(
            "https://{}.com/{}?utm_source={}&x={}",
            host, path, token, mark
        );
        let item = make();
        let value = item.clean(&raw);
        assert!(
            !value.contains("utm_source"),
            "utm parameters were not removed"
        );
    }

    #[test]
    fn the_link_strips_utm_from_text() {
        let mut rng = ids::ids(18003);
        let host = ids::ascii(&mut rng, 6);
        let path = ids::armenian(&mut rng, 4);
        let token = ids::hebrew(&mut rng, 4);
        let raw = format!("https://{}.org/{}?utm_medium={}", host, path, token);
        let prefix = ids::cyrillic(&mut rng, 5);
        let text = format!("{} {}", prefix, raw);
        let item = make();
        let value = item.strip(&text);
        assert!(
            !value.contains("utm_medium"),
            "utm parameters were not stripped from text"
        );
    }

    #[test]
    fn the_link_extracts_domain() {
        let mut rng = ids::ids(18005);
        let host = ids::ascii(&mut rng, 6);
        let path = ids::greek(&mut rng, 4);
        let raw = format!("https://www.{}.net/{}", host, path);
        let item = make();
        let name = item.domain(&raw);
        assert_eq!(
            format!("{}.net", host),
            name,
            "domain did not strip www prefix"
        );
    }

    #[test]
    fn the_link_collects_urls() {
        let mut rng = ids::ids(18007);
        let left = ids::ascii(&mut rng, 6);
        let right = ids::ascii(&mut rng, 5);
        let path = ids::hiragana(&mut rng, 4);
        let one = format!("https://{}.com/{}", left, path);
        let two = format!("http://{}.org/{}", right, path);
        let prefix = ids::cyrillic(&mut rng, 5);
        let text = format!("{} {} {}", prefix, one, two);
        let item = make();
        let items = item.links(&text);
        assert_eq!(vec![one, two], items, "urls were not extracted");
    }
}
