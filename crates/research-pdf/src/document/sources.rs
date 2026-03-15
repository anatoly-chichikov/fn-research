use regex::Regex;

use research_domain::result::Sourced;

use super::text;

/// Source entry with provider context.
pub struct SourceEntry {
    /// Citation source.
    pub source: Box<dyn Sourced>,
    /// Provider name.
    pub provider: String,
}

/// Render sources section.
pub fn section(list: &[SourceEntry]) -> String {
    if list.is_empty() {
        return String::new();
    }
    let mut rows = String::new();
    for item in list {
        let link = text::trim(item.source.url());
        let title = text::label(item.source.as_ref(), &item.provider);
        let note = if item.provider == "valyu" {
            text::excerpt(item.source.excerpt())
        } else {
            String::new()
        };
        let view = text::presentation(&link);
        let head = if title == link { view.clone() } else { title };
        let link = text::escape(&link);
        let view = text::escape(&view);
        let head = text::escape(&head);
        let note = text::escape(&note);
        let site = if view.is_empty() || view == head {
            String::new()
        } else {
            format!("<div class=\"source-url\">{}</div>", view)
        };
        let excerpt_html = if note.is_empty() {
            String::new()
        } else {
            format!("<div class=\"source-excerpt\">{}</div>", note)
        };
        rows.push_str(&format!(
            "<li class=\"ref-item\">\
             <a class=\"ref-link\" href=\"{}\" target=\"_blank\">{}</a>\
             {}{}</li>",
            link, head, site, excerpt_html
        ));
    }
    format!(
        "<section class=\"references\">\
         <h2>Sources</h2><ol class=\"ref-list\">{}</ol></section>",
        rows
    )
}

/// Wrap emoji characters in spans.
pub fn emojify(text: &str) -> String {
    let re =
        Regex::new("([\u{1F000}-\u{1FAFF}\u{2600}-\u{27BF}\u{2300}-\u{23FF}]\u{FE0F}?)").unwrap();
    re.replace_all(text, "<span class=\"emoji\">$1</span>")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;
    use research_domain::result::CitationSource;

    #[test]
    fn the_sources_section_renders_list() {
        let mut rng = ids::ids(23001);
        let head = ids::cyrillic(&mut rng, 6);
        let number = ids::digit(&mut rng, 1000);
        let link = format!("https://example.com/{}", number);
        let source = CitationSource::new(&head, &link, "");
        let list = vec![SourceEntry {
            source: Box::new(source),
            provider: "parallel".to_string(),
        }];
        let result = section(&list);
        assert!(
            result.contains("ref-list"),
            "Sources section did not render list"
        );
    }

    #[test]
    fn the_sources_section_returns_empty_for_no_sources() {
        let result = section(&[]);
        assert!(
            result.is_empty(),
            "Sources section was not empty for no sources"
        );
    }

    #[test]
    fn the_sources_emojify_wraps_emoji() {
        let mut rng = ids::ids(23005);
        let head = ids::cyrillic(&mut rng, 6);
        let tail = ids::hiragana(&mut rng, 6);
        let mark = char::from_u32(9989).unwrap();
        let text = format!("{} {} {}", head, mark, tail);
        let result = emojify(&text);
        assert!(
            result.contains("class=\"emoji\"") && result.contains(&mark.to_string()),
            "Emoji span was not rendered"
        );
    }
}
