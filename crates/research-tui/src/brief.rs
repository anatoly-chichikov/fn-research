//! TUI-only brief structure with editable properties.
//!
//! The persisted `research_domain::brief::Brief` only stores the rendered
//! questions. While the user is reviewing the brief in the TUI, we hold a
//! richer structure carrying intent text and depth/novelty/applied props.
//! On approve, [`TuiBrief::to_domain`] strips both before handing the brief
//! to the headless `research run` flow.

pub mod gemini;
pub mod generator;
pub mod mock;
pub mod prompt;

pub use generator::{BriefError, BriefGenerator, BriefRequest};

/// Full editable brief — exactly 3 root angles, exactly 3 sub-angles each.
#[derive(Clone, Debug)]
pub struct TuiBrief {
    pub topic: String,
    pub language: String,
    pub intent: String,
    pub topics: [TuiRoot; 3],
}

/// A root angle.
#[derive(Clone, Debug)]
pub struct TuiRoot {
    pub title: String,
    pub note: String,
    pub depth: u8,
    pub novelty: u8,
    pub applied: u8,
    pub subs: [TuiSub; 3],
}

/// A sub-angle.
#[derive(Clone, Debug)]
pub struct TuiSub {
    pub title: String,
    pub depth: u8,
    pub novelty: u8,
    pub applied: u8,
}

impl TuiBrief {
    /// Render the plain-text brief that the headless binary parses.
    ///
    /// Format mirrors the existing `Brief::parse` expectation: tab-indented
    /// tree starting with a "Research:" header. Properties and intent are
    /// dropped — the engine sees only the questions.
    pub fn render_query(&self) -> String {
        let mut buf = format!("Язык ответа: {}.\n\nResearch:\n", self.language);
        for root in &self.topics {
            buf.push_str(&root.title);
            buf.push('\n');
            for sub in &root.subs {
                buf.push('\t');
                buf.push_str(&sub.title);
                buf.push('\n');
            }
        }
        buf
    }

    /// Convert to the persisted domain brief — used at session creation time.
    pub fn to_domain(&self) -> research_domain::brief::Brief {
        let questions = self
            .topics
            .iter()
            .map(|root| research_domain::brief::Question {
                scope: root.title.clone(),
                details: root
                    .subs
                    .iter()
                    .map(|sub| research_domain::brief::Question {
                        scope: sub.title.clone(),
                        details: Vec::new(),
                    })
                    .collect(),
            })
            .collect();
        research_domain::brief::Brief {
            title: self.topic.clone(),
            language: self.language.clone(),
            questions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_query_starts_with_a_language_header() {
        let brief = mock::sample("Quantum computing", "English");
        let q = brief.render_query();
        assert!(
            q.starts_with("Язык ответа: English."),
            "missing language header in rendered query"
        );
    }

    #[test]
    fn the_domain_brief_has_three_root_questions() {
        let brief = mock::sample("Quantum computing", "English");
        let domain = brief.to_domain();
        assert_eq!(3, domain.questions.len(), "domain brief lost a root");
    }
}
