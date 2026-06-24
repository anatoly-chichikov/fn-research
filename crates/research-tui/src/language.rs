//! Detect the result language from the topic text.
//!
//! The TUI's UI is English-only, but the brief gets handed to the research
//! engine in the user's apparent language. The rule is simple: if the topic
//! contains Cyrillic letters, render in Russian; otherwise, English. Greek
//! and Spanish would each need their own range — kept English-only for now.

/// Pick the result language for a topic.
pub fn detect(topic: &str) -> &'static str {
    if topic.chars().any(is_cyrillic) {
        "Russian"
    } else {
        "English"
    }
}

/// Cyrillic block in the BMP.
fn is_cyrillic(ch: char) -> bool {
    matches!(ch as u32, 0x0400..=0x052F)
}
