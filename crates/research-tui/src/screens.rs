//! One module per screen — each exposes `render` and `handle_key`.

pub mod approve;
pub mod confirmation;
pub mod generating;
pub mod input;
pub mod keys;
pub mod sessions;
pub mod tune;

/// Truncate a string to a max character count, appending `…` when cut.
pub fn truncate(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max {
        text.to_string()
    } else {
        let mut s: String = chars[..max.saturating_sub(1)].iter().collect();
        s.push('\u{2026}');
        s
    }
}
