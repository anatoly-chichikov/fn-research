use crate::palette::Colored;

/// Object that can render to string.
pub trait Rendered {
    /// Return rendered string.
    fn render(&self) -> String;
}

/// Wave SVG decoration.
pub struct Wave<'a> {
    pal: &'a dyn Colored,
}

impl<'a> Wave<'a> {
    /// Create wave from palette.
    pub fn new(pal: &'a dyn Colored) -> Self {
        Self { pal }
    }
}

impl Rendered for Wave<'_> {
    fn render(&self) -> String {
        format!(
            concat!(
                "<svg viewBox=\"0 0 1200 200\" preserveAspectRatio=\"none\" ",
                "xmlns=\"http://www.w3.org/2000/svg\">",
                "<defs><linearGradient id=\"waveGradient\" x1=\"0%\" y1=\"0%\" ",
                "x2=\"0%\" y2=\"100%\">",
                "<stop offset=\"0%\" style=\"stop-color:{};stop-opacity:0.9\"/>",
                "<stop offset=\"100%\" style=\"stop-color:{};stop-opacity:1\"/>",
                "</linearGradient></defs>",
                "<path d=\"M0,100 C100,150 200,50 300,100 C400,150 500,50 600,",
                "100 C700,150 800,50 900,100 C1000,150 1100,50 1200,100 ",
                "L1200,200 L0,200 Z\" fill=\"url(#waveGradient)\"/>",
                "<path d=\"M0,120 C150,80 250,160 400,120 C550,80 650,160 800,",
                "120 C950,80 1050,160 1200,120 L1200,200 L0,200 Z\" fill=\"",
                "{}\" opacity=\"0.7\"/>",
                "<path d=\"M0,140 C200,180 400,100 600,140 C800,180 1000,100 ",
                "1200,140 L1200,200 L0,200 Z\" fill=\"",
                "{}\" opacity=\"0.5\"/>",
                "<circle cx=\"100\" cy=\"90\" r=\"8\" fill=\"",
                "{}\" opacity=\"0.8\"/>",
                "<circle cx=\"350\" cy=\"70\" r=\"6\" fill=\"",
                "{}\" opacity=\"0.6\"/>",
                "<circle cx=\"600\" cy=\"85\" r=\"10\" fill=\"",
                "{}\" opacity=\"0.7\"/>",
                "<circle cx=\"850\" cy=\"75\" r=\"7\" fill=\"",
                "{}\" opacity=\"0.5\"/>",
                "<circle cx=\"1100\" cy=\"80\" r=\"9\" fill=\"",
                "{}\" opacity=\"0.8\"/>",
                "</svg>",
            ),
            self.pal.link(),
            self.pal.heading(),
            self.pal.heading(),
            self.pal.text(),
            self.pal.bg(),
            self.pal.bg(),
            self.pal.bg(),
            self.pal.bg(),
            self.pal.bg(),
        )
    }
}

/// Footer SVG decoration.
pub struct Footer<'a> {
    pal: &'a dyn Colored,
}

impl<'a> Footer<'a> {
    /// Create footer from palette.
    pub fn new(pal: &'a dyn Colored) -> Self {
        Self { pal }
    }
}

impl Rendered for Footer<'_> {
    fn render(&self) -> String {
        format!(
            concat!(
                "<svg viewBox=\"0 0 1200 100\" preserveAspectRatio=\"none\" ",
                "xmlns=\"http://www.w3.org/2000/svg\">",
                "<path d=\"M0,0 L0,50 C150,80 350,20 600,50 C850,80 1050,20 ",
                "1200,50 L1200,0 Z\" fill=\"",
                "{}\" opacity=\"0.3\"/>",
                "<path d=\"M0,0 L0,30 C200,60 400,10 600,30 C800,50 1000,10 ",
                "1200,30 L1200,0 Z\" fill=\"",
                "{}\" opacity=\"0.5\"/>",
                "</svg>",
            ),
            self.pal.link(),
            self.pal.heading(),
        )
    }
}

/// Create wave from palette.
pub fn wave(pal: &dyn Colored) -> Wave<'_> {
    Wave::new(pal)
}

/// Create footer from palette.
pub fn footer(pal: &dyn Colored) -> Footer<'_> {
    Footer::new(pal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette;

    #[test]
    fn the_wave_render_returns_svg() {
        let pal = palette::palette();
        let item = wave(&pal);
        let html = item.render();
        assert!(
            html.contains("<svg"),
            "Rendered wave did not contain svg tag"
        );
    }

    #[test]
    fn the_wave_render_contains_path() {
        let pal = palette::palette();
        let item = wave(&pal);
        let html = item.render();
        assert!(
            html.contains("<path"),
            "Rendered wave did not contain path tag"
        );
    }

    #[test]
    fn the_wave_render_contains_palette_color() {
        let pal = palette::palette();
        let item = wave(&pal);
        let html = item.render();
        assert!(
            html.contains(pal.link()),
            "Rendered wave did not contain palette link color"
        );
    }

    #[test]
    fn the_wave_render_contains_gradient() {
        let pal = palette::palette();
        let item = wave(&pal);
        let html = item.render();
        assert!(
            html.contains("linearGradient"),
            "Rendered wave did not contain gradient"
        );
    }

    #[test]
    fn the_footer_render_returns_svg() {
        let pal = palette::palette();
        let item = footer(&pal);
        let html = item.render();
        assert!(
            html.contains("<svg"),
            "Rendered footer did not contain svg tag"
        );
    }

    #[test]
    fn the_footer_render_contains_path() {
        let pal = palette::palette();
        let item = footer(&pal);
        let html = item.render();
        assert!(
            html.contains("<path"),
            "Rendered footer did not contain path tag"
        );
    }
}
