use regex::Regex;

/// Object that can emit progress output.
pub trait Progressed {
    /// Emit progress text.
    fn emit(&self, text: &str) -> bool;
    /// Return cleaned progress text.
    fn clean(&self, text: &str) -> String;
}

/// Progress emitter.
pub struct Progress {
    dot: Regex,
}

impl Progress {
    /// Create progress emitter from pattern.
    pub fn new(dot: Regex) -> Self {
        Self { dot }
    }
}

impl Progressed for Progress {
    fn emit(&self, text: &str) -> bool {
        let note = self.clean(text);
        println!("{}", note);
        true
    }

    fn clean(&self, text: &str) -> String {
        self.dot.replace_all(text, "").to_string()
    }
}

/// Return default progress emitter.
pub fn make() -> Progress {
    Progress::new(Regex::new(r"\.").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_progress_cleans_periods() {
        let mut rng = ids::ids(18101);
        let text = ids::cyrillic(&mut rng, 6);
        let value = format!("{}.{}.", text, text);
        let item = make();
        let note = item.clean(&value);
        assert!(!note.contains('.'), "progress clean did not remove periods");
    }

    #[test]
    fn the_progress_emits_text() {
        let mut rng = ids::ids(18103);
        let text = ids::greek(&mut rng, 6);
        let item = make();
        let flag = item.emit(&text);
        assert!(flag, "progress emit did not return success");
    }
}
