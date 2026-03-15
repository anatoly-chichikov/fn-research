use chrono::NaiveDateTime;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Object that organizes output files.
pub trait Organized {
    /// Return folder name.
    fn name(&self, time: &NaiveDateTime, text: &str, id: &str) -> String;
    /// Return output folder path.
    fn folder(&self, name: &str, provider: &str) -> PathBuf;
    /// Save response JSON and return path.
    fn response(&self, name: &str, provider: &str, data: &serde_json::Value) -> PathBuf;
    /// Return cover path.
    fn cover(&self, name: &str, provider: &str) -> PathBuf;
    /// Return brief RON path.
    fn brief(&self, name: &str, provider: &str) -> PathBuf;
    /// Return report path.
    fn report(&self, name: &str, provider: &str) -> PathBuf;
    /// Return html path.
    fn html(&self, name: &str, provider: &str) -> PathBuf;
    /// Return existing cover path.
    fn existing(&self, name: &str, provider: &str) -> Option<PathBuf>;
}

/// Transliterate Cyrillic into Latin.
pub fn translit(text: &str) -> String {
    let mut table: HashMap<char, &str> = HashMap::new();
    let pairs = [
        ('\u{0430}', "a"),
        ('\u{0431}', "b"),
        ('\u{0432}', "v"),
        ('\u{0433}', "g"),
        ('\u{0434}', "d"),
        ('\u{0435}', "e"),
        ('\u{0451}', "yo"),
        ('\u{0436}', "zh"),
        ('\u{0437}', "z"),
        ('\u{0438}', "i"),
        ('\u{0439}', "y"),
        ('\u{043a}', "k"),
        ('\u{043b}', "l"),
        ('\u{043c}', "m"),
        ('\u{043d}', "n"),
        ('\u{043e}', "o"),
        ('\u{043f}', "p"),
        ('\u{0440}', "r"),
        ('\u{0441}', "s"),
        ('\u{0442}', "t"),
        ('\u{0443}', "u"),
        ('\u{0444}', "f"),
        ('\u{0445}', "h"),
        ('\u{0446}', "ts"),
        ('\u{0447}', "ch"),
        ('\u{0448}', "sh"),
        ('\u{0449}', "sch"),
        ('\u{044a}', ""),
        ('\u{044b}', "y"),
        ('\u{044c}', ""),
        ('\u{044d}', "e"),
        ('\u{044e}', "yu"),
        ('\u{044f}', "ya"),
    ];
    for (k, v) in &pairs {
        table.insert(*k, v);
    }
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        let lower = ch.to_lowercase().next().unwrap_or(ch);
        if let Some(hit) = table.get(&lower) {
            if ch == lower {
                result.push_str(hit);
            } else {
                result.push_str(&hit.to_uppercase());
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Convert text into safe slug.
pub fn slug(text: &str) -> String {
    let text = translit(text);
    let text = text.to_lowercase();
    let text: String = text
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace() || *c == '-')
        .collect();
    let text = regex::Regex::new(r"\s+")
        .unwrap()
        .replace_all(&text, "-")
        .to_string();
    let text = if text.len() > 40 { &text[..40] } else { &text };
    if text.is_empty() {
        "untitled".to_string()
    } else {
        text.to_string()
    }
}

/// Output organizer for file paths.
pub struct Organizer {
    root: PathBuf,
}

impl Organizer {
    /// Create organizer from root path.
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
}

impl Organized for Organizer {
    fn name(&self, time: &NaiveDateTime, text: &str, id: &str) -> String {
        let date = time.format("%Y-%m-%d").to_string();
        let tag = slug(text);
        let code = &id[..8.min(id.len())];
        format!("{}_{}_{}", date, tag, code)
    }

    fn folder(&self, name: &str, _provider: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::create_dir_all(&path).ok();
        path
    }

    fn response(&self, name: &str, provider: &str, data: &serde_json::Value) -> PathBuf {
        let tag = slug(provider);
        let tag = if tag.is_empty() {
            "provider".to_string()
        } else {
            tag
        };
        let dir = self.folder(name, provider);
        let path = dir.join(format!("response-{}.json", tag));
        let text = serde_json::to_string_pretty(data).unwrap_or_default();
        fs::write(&path, text).ok();
        path
    }

    fn cover(&self, name: &str, provider: &str) -> PathBuf {
        let tag = slug(provider);
        let tag = if tag.is_empty() {
            "provider".to_string()
        } else {
            tag
        };
        let dir = self.folder(name, provider);
        dir.join(format!("cover-{}.jpg", tag))
    }

    fn brief(&self, name: &str, provider: &str) -> PathBuf {
        let tag = slug(provider);
        let tag = if tag.is_empty() {
            "provider".to_string()
        } else {
            tag
        };
        let dir = self.folder(name, provider);
        dir.join(format!("brief-{}.ron", tag))
    }

    fn report(&self, name: &str, provider: &str) -> PathBuf {
        let stem = match name.rfind('_') {
            Some(pos) => &name[..pos],
            None => name,
        };
        let tag = slug(provider);
        let tag = if tag.is_empty() {
            "provider".to_string()
        } else {
            tag
        };
        let dir = self.folder(name, provider);
        dir.join(format!("{}-{}.pdf", stem, tag))
    }

    fn html(&self, name: &str, provider: &str) -> PathBuf {
        let stem = match name.rfind('_') {
            Some(pos) => &name[..pos],
            None => name,
        };
        let tag = slug(provider);
        let tag = if tag.is_empty() {
            "provider".to_string()
        } else {
            tag
        };
        let dir = self.folder(name, provider);
        dir.join(format!("{}-{}.html", stem, tag))
    }

    fn existing(&self, name: &str, provider: &str) -> Option<PathBuf> {
        let tag = slug(provider);
        let tag = if tag.is_empty() {
            "provider".to_string()
        } else {
            tag
        };
        let base = self.root.join(name);
        let path = base.join(format!("cover-{}.jpg", tag));
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

/// Create organizer from root path.
pub fn organizer(root: &Path) -> Organizer {
    Organizer::new(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use research_domain::ids;
    use tempfile::TempDir;

    #[test]
    fn the_organizer_creates_folder_for_session() {
        let mut rng = ids::ids(23001);
        let dir = TempDir::new().unwrap();
        let ident = ids::uuid(&mut rng);
        let provider = ids::cyrillic(&mut rng, 6);
        let item = organizer(dir.path());
        let path = item.folder(&ident, &provider);
        assert!(path.exists(), "Folder was not created for session");
    }

    #[test]
    fn the_organizer_folder_contains_identifier() {
        let mut rng = ids::ids(23003);
        let dir = TempDir::new().unwrap();
        let ident = ids::uuid(&mut rng);
        let item = organizer(dir.path());
        let path = item.folder(&ident, "valyu");
        assert!(
            path.display().to_string().contains(&ident),
            "Folder path did not contain session identifier"
        );
    }

    #[test]
    fn the_organizer_saves_response_as_json() {
        let mut rng = ids::ids(23005);
        let dir = TempDir::new().unwrap();
        let ident = ids::uuid(&mut rng);
        let item = organizer(dir.path());
        let key = format!("test-{}", ids::uuid(&mut rng));
        let path = item.response(
            &ident,
            "valyu",
            &serde_json::json!({key.clone(): "donn\u{00e9}es"}),
        );
        let text = fs::read_to_string(&path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            "donn\u{00e9}es",
            data.get(&key).unwrap().as_str().unwrap(),
            "Response JSON did not contain expected data"
        );
    }

    #[test]
    fn the_organizer_response_creates_folder() {
        let mut rng = ids::ids(23007);
        let dir = TempDir::new().unwrap();
        let ident = ids::uuid(&mut rng);
        let item = organizer(dir.path());
        let path = item.response(&ident, "parallel", &serde_json::json!({"created": true}));
        assert!(
            path.parent().unwrap().exists(),
            "Response did not create parent folder"
        );
    }

    #[test]
    fn the_organizer_cover_returns_jpg_path() {
        let mut rng = ids::ids(23009);
        let dir = TempDir::new().unwrap();
        let ident = ids::uuid(&mut rng);
        let provider = ids::cyrillic(&mut rng, 6);
        let item = organizer(dir.path());
        let path = item.cover(&ident, &provider);
        let tag = slug(&provider);
        let goal = format!("cover-{}.jpg", tag);
        assert!(
            path.display().to_string().ends_with(&goal),
            "Cover path did not include provider suffix"
        );
    }

    #[test]
    fn the_organizer_report_returns_pdf_path() {
        let mut rng = ids::ids(23011);
        let dir = TempDir::new().unwrap();
        let ident = ids::uuid(&mut rng);
        let provider = ids::cyrillic(&mut rng, 6);
        let item = organizer(dir.path());
        let path = item.report(&ident, &provider);
        let tag = slug(&provider);
        let goal = format!("-{}.pdf", tag);
        assert!(
            path.display().to_string().ends_with(&goal),
            "Report path did not include provider suffix"
        );
    }

    #[test]
    fn the_organizer_html_returns_html_path() {
        let mut rng = ids::ids(23013);
        let dir = TempDir::new().unwrap();
        let ident = ids::uuid(&mut rng);
        let provider = ids::cyrillic(&mut rng, 6);
        let item = organizer(dir.path());
        let path = item.html(&ident, &provider);
        let tag = slug(&provider);
        let goal = format!("-{}.html", tag);
        assert!(
            path.display().to_string().ends_with(&goal),
            "HTML path did not include provider suffix"
        );
    }

    #[test]
    fn the_organizer_existing_returns_none_for_missing() {
        let mut rng = ids::ids(23015);
        let dir = TempDir::new().unwrap();
        let ident = ids::uuid(&mut rng);
        let item = organizer(dir.path());
        let result = item.existing(&ident, "valyu");
        assert!(result.is_none(), "Existing returned path for missing cover");
    }

    #[test]
    fn the_organizer_existing_returns_path_when_exists() {
        let mut rng = ids::ids(23017);
        let dir = TempDir::new().unwrap();
        let ident = ids::uuid(&mut rng);
        let item = organizer(dir.path());
        let cover = item.cover(&ident, "valyu");
        fs::create_dir_all(cover.parent().unwrap()).unwrap();
        fs::write(&cover, "fake image").unwrap();
        assert!(
            item.existing(&ident, "valyu").is_some(),
            "Existing returned empty for existing cover"
        );
    }

    #[test]
    fn the_organizer_name_formats_correctly() {
        let dir = TempDir::new().unwrap();
        let item = organizer(dir.path());
        let created = NaiveDate::from_ymd_opt(2025, 12, 7)
            .unwrap()
            .and_hms_opt(15, 30, 0)
            .unwrap();
        let topic = "Coffee vs Tea";
        let ident = "589a125c-8ae7-4c28-ac95-7c1127b601d3";
        let name = item.name(&created, topic, ident);
        assert_eq!(
            "2025-12-07_coffee-vs-tea_589a125c", name,
            "Name format did not match expected pattern"
        );
    }

    #[test]
    fn the_organizer_name_handles_special_characters() {
        let mut rng = ids::ids(23023);
        let dir = TempDir::new().unwrap();
        let item = organizer(dir.path());
        let created = NaiveDate::from_ymd_opt(2025, 1, 15)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let topic = "What's the deal with: \u{00e9}mojis & symbols?";
        let ident = ids::uuid(&mut rng);
        let name = item.name(&created, topic, &ident);
        let pattern = regex::Regex::new(r"^2025-01-15_[a-z0-9-]+_[a-f0-9]{8}$").unwrap();
        assert!(pattern.is_match(&name), "Name contained invalid characters");
    }

    #[test]
    fn the_organizer_name_transliterates_cyrillic() {
        let mut rng = ids::ids(23025);
        let dir = TempDir::new().unwrap();
        let item = organizer(dir.path());
        let created = NaiveDate::from_ymd_opt(2025, 12, 22)
            .unwrap()
            .and_hms_opt(19, 30, 0)
            .unwrap();
        let topic = ids::cyrillic(&mut rng, 5);
        let ident = "bb1ce2e7-1234-5678-9abc-def012345678";
        let name = item.name(&created, &topic, ident);
        let pattern = regex::Regex::new(r"^2025-12-22_[a-z0-9-]+_bb1ce2e7$").unwrap();
        assert!(
            pattern.is_match(&name),
            "Cyrillic topic was not transliterated correctly"
        );
    }

    #[test]
    fn the_organizer_name_falls_back_to_untitled() {
        let dir = TempDir::new().unwrap();
        let item = organizer(dir.path());
        let created = NaiveDate::from_ymd_opt(2025, 12, 22)
            .unwrap()
            .and_hms_opt(19, 30, 0)
            .unwrap();
        let topic = "\u{4E2D}\u{6587}\u{4E3B}\u{984C}";
        let ident = "abc12345-1234-5678-9abc-def012345678";
        let name = item.name(&created, topic, ident);
        assert_eq!(
            "2025-12-22_untitled_abc12345", name,
            "Empty slug did not fallback to untitled"
        );
    }
}
