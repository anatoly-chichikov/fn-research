/// Object with palette colors.
pub trait Colored {
    /// Return background color.
    fn bg(&self) -> &str;
    /// Return text color.
    fn text(&self) -> &str;
    /// Return heading color.
    fn heading(&self) -> &str;
    /// Return link color.
    fn link(&self) -> &str;
    /// Return muted color.
    fn muted(&self) -> &str;
    /// Return quote background color.
    fn quote(&self) -> &str;
    /// Return accent color.
    fn accent(&self) -> &str;
    /// Return code block color.
    fn codebg(&self) -> &str;
    /// Return inline code color.
    fn codeinline(&self) -> &str;
    /// Return border color.
    fn border(&self) -> &str;
}

/// Hokusai-themed color palette.
pub struct Palette {
    colors: Colors,
}

/// Color values container.
struct Colors {
    bg: String,
    text: String,
    heading: String,
    link: String,
    muted: String,
    quote: String,
    accent: String,
    codebg: String,
    codeinline: String,
    border: String,
}

impl Palette {
    /// Create palette from color values.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bg: &str,
        text: &str,
        heading: &str,
        link: &str,
        muted: &str,
        quote: &str,
        accent: &str,
        codebg: &str,
        codeinline: &str,
        border: &str,
    ) -> Self {
        Self {
            colors: Colors {
                bg: bg.to_string(),
                text: text.to_string(),
                heading: heading.to_string(),
                link: link.to_string(),
                muted: muted.to_string(),
                quote: quote.to_string(),
                accent: accent.to_string(),
                codebg: codebg.to_string(),
                codeinline: codeinline.to_string(),
                border: border.to_string(),
            },
        }
    }
}

impl Colored for Palette {
    fn bg(&self) -> &str {
        &self.colors.bg
    }
    fn text(&self) -> &str {
        &self.colors.text
    }
    fn heading(&self) -> &str {
        &self.colors.heading
    }
    fn link(&self) -> &str {
        &self.colors.link
    }
    fn muted(&self) -> &str {
        &self.colors.muted
    }
    fn quote(&self) -> &str {
        &self.colors.quote
    }
    fn accent(&self) -> &str {
        &self.colors.accent
    }
    fn codebg(&self) -> &str {
        &self.colors.codebg
    }
    fn codeinline(&self) -> &str {
        &self.colors.codeinline
    }
    fn border(&self) -> &str {
        &self.colors.border
    }
}

/// Create palette with Hokusai defaults.
pub fn palette() -> Palette {
    Palette::new(
        "#F6EFE3", "#1C2430", "#193D5E", "#3A5F88", "#6B645A", "#E3D9C6", "#D04A35", "#1C2833",
        "#DDD5C5", "#BFB5A3",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_palette_bg_returns_hex() {
        let mut rng = ids::ids(19001);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.bg().starts_with('#'),
            "Bg color did not start with hash"
        );
    }

    #[test]
    fn the_palette_bg_returns_seven_characters() {
        let mut rng = ids::ids(19003);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert_eq!(7, pal.bg().len(), "Bg color was not seven characters");
    }

    #[test]
    fn the_palette_text_returns_hex() {
        let mut rng = ids::ids(19005);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.text().starts_with('#'),
            "Text color did not start with hash"
        );
    }

    #[test]
    fn the_palette_heading_returns_hex() {
        let mut rng = ids::ids(19007);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.heading().starts_with('#'),
            "Heading color did not start with hash"
        );
    }

    #[test]
    fn the_palette_link_returns_hex() {
        let mut rng = ids::ids(19009);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.link().starts_with('#'),
            "Link color did not start with hash"
        );
    }

    #[test]
    fn the_palette_muted_returns_hex() {
        let mut rng = ids::ids(19011);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.muted().starts_with('#'),
            "Muted color did not start with hash"
        );
    }

    #[test]
    fn the_palette_quote_returns_hex() {
        let mut rng = ids::ids(19013);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.quote().starts_with('#'),
            "Quote color did not start with hash"
        );
    }

    #[test]
    fn the_palette_accent_returns_hex() {
        let mut rng = ids::ids(19015);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.accent().starts_with('#'),
            "Accent color did not start with hash"
        );
    }

    #[test]
    fn the_palette_codebg_returns_hex() {
        let mut rng = ids::ids(19017);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.codebg().starts_with('#'),
            "Codebg color did not start with hash"
        );
    }

    #[test]
    fn the_palette_codeinline_returns_hex() {
        let mut rng = ids::ids(19019);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.codeinline().starts_with('#'),
            "Codeinline color did not start with hash"
        );
    }

    #[test]
    fn the_palette_border_returns_hex() {
        let mut rng = ids::ids(19021);
        let _text = ids::cyrillic(&mut rng, 3);
        let pal = palette();
        assert!(
            pal.border().starts_with('#'),
            "Border color did not start with hash"
        );
    }
}
