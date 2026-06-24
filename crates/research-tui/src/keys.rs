//! Read-only inventory of the env vars the program cares about.

/// One row on the keys screen.
#[derive(Clone, Debug)]
pub struct KeyRow {
    pub name: &'static str,
    pub required: bool,
    pub set: bool,
    pub preview: String,
}

const ENTRIES: &[(&str, bool)] = &[
    ("PARALLEL_API_KEY", true),
    ("VALYU_API_KEY", true),
    ("XAI_API_KEY", false),
    ("GEMINI_API_KEY", false),
    ("REPORT_FOR", false),
];

/// Read every relevant env var and produce display rows.
pub fn load() -> Vec<KeyRow> {
    ENTRIES
        .iter()
        .map(|(name, required)| {
            let value = std::env::var(name).unwrap_or_default();
            let set = !value.is_empty();
            let preview = if set { mask(&value) } else { String::new() };
            KeyRow {
                name,
                required: *required,
                set,
                preview,
            }
        })
        .collect()
}

/// Mask a secret-looking value, showing the last four characters.
fn mask(value: &str) -> String {
    let trimmed = value.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    let n = chars.len();
    if n <= 4 {
        return "•".repeat(n);
    }
    let tail: String = chars[n - 4..].iter().collect();
    format!("{}{}", "•".repeat(12), tail)
}
