use std::path::Path;

use crate::palette::Colored;

/// Object with CSS style.
pub trait Styled {
    /// Return CSS string.
    fn css(&self) -> String;
}

/// Replace palette tokens in CSS.
pub fn fill(text: &str, item: &dyn Colored) -> String {
    text.replace("__BG__", item.bg())
        .replace("__TEXT__", item.text())
        .replace("__HEADING__", item.heading())
        .replace("__LINK__", item.link())
        .replace("__MUTED__", item.muted())
        .replace("__QUOTE__", item.quote())
        .replace("__ACCENT__", item.accent())
        .replace("__CODEBG__", item.codebg())
        .replace("__CODEINLINE__", item.codeinline())
        .replace("__BORDER__", item.border())
}

/// CSS style with palette.
pub struct Style<'a> {
    pal: &'a dyn Colored,
    root: String,
}

impl<'a> Style<'a> {
    /// Create style from palette and resource root.
    pub fn new(pal: &'a dyn Colored, root: &str) -> Self {
        Self {
            pal,
            root: root.to_string(),
        }
    }
}

impl Styled for Style<'_> {
    fn css(&self) -> String {
        let path = Path::new(&self.root).join("style.css");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Style resource missing at {}", path.display()));
        fill(&text, self.pal)
    }
}

/// Create style from palette and resource root.
pub fn style<'a>(pal: &'a dyn Colored, root: &str) -> Style<'a> {
    Style::new(pal, root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    struct ConstPalette {
        value: String,
    }

    impl ConstPalette {
        fn new(value: &str) -> Self {
            Self {
                value: value.to_string(),
            }
        }
    }

    impl Colored for ConstPalette {
        fn bg(&self) -> &str {
            &self.value
        }
        fn text(&self) -> &str {
            &self.value
        }
        fn heading(&self) -> &str {
            &self.value
        }
        fn link(&self) -> &str {
            &self.value
        }
        fn muted(&self) -> &str {
            &self.value
        }
        fn quote(&self) -> &str {
            &self.value
        }
        fn accent(&self) -> &str {
            &self.value
        }
        fn codebg(&self) -> &str {
            &self.value
        }
        fn codeinline(&self) -> &str {
            &self.value
        }
        fn border(&self) -> &str {
            &self.value
        }
    }

    fn root() -> String {
        let manifest = env!("CARGO_MANIFEST_DIR");
        format!("{}/../../resources", manifest)
    }

    #[test]
    fn the_style_includes_image_constraints() {
        let mut rng = ids::ids(20001);
        let value = ids::cyrillic(&mut rng, 6);
        let pal = ConstPalette::new(&value);
        let item = style(&pal, &root());
        let css = item.css();
        assert!(
            css.contains(".synthesis img"),
            "Image constraints were not included in stylesheet"
        );
    }

    #[test]
    fn the_style_synthesis_omits_left_border() {
        let mut rng = ids::ids(20009);
        let value = ids::cyrillic(&mut rng, 6);
        let pal = ConstPalette::new(&value);
        let item = style(&pal, &root());
        let css = item.css();
        assert!(
            css.contains("border-left: none;"),
            "Synthesis left border was present"
        );
    }

    #[test]
    fn the_style_hr_hides_divider_line() {
        let mut rng = ids::ids(20011);
        let value = ids::cyrillic(&mut rng, 6);
        let pal = ConstPalette::new(&value);
        let item = style(&pal, &root());
        let css = item.css();
        assert!(
            css.contains("border-top: 0;"),
            "Horizontal rule line was visible"
        );
    }

    #[test]
    fn the_style_brief_query_uses_quote_colors() {
        let mut rng = ids::ids(20013);
        let value = ids::cyrillic(&mut rng, 6);
        let pal = ConstPalette::new(&value);
        let item = style(&pal, &root());
        let css = item.css();
        assert!(
            css.contains("background: var(--quote-bg);"),
            "Brief query background was not quote color"
        );
    }

    #[test]
    fn the_style_serif_font_includes_japanese_fallback() {
        let mut rng = ids::ids(20019);
        let value = ids::cyrillic(&mut rng, 6);
        let pal = ConstPalette::new(&value);
        let item = style(&pal, &root());
        let css = item.css();
        assert!(
            css.contains("Noto Serif JP"),
            "Japanese fallback font was missing"
        );
    }

    #[test]
    fn the_style_includes_emoji_fallbacks() {
        let mut rng = ids::ids(20021);
        let value = ids::cyrillic(&mut rng, 6);
        let pal = ConstPalette::new(&value);
        let item = style(&pal, &root());
        let css = item.css();
        assert!(
            css.contains(".emoji"),
            "Emoji class was missing from stylesheet"
        );
    }

    #[test]
    fn the_style_raises_citations() {
        let mut rng = ids::ids(20023);
        let value = ids::cyrillic(&mut rng, 6);
        let pal = ConstPalette::new(&value);
        let item = style(&pal, &root());
        let css = item.css();
        assert!(
            css.contains("vertical-align: super;"),
            "Citation links were not raised"
        );
    }
}
