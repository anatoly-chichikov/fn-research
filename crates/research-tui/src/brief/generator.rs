//! Trait that abstracts brief generation from the underlying LLM.
//!
//! Two implementations: a synchronous mock that returns a sample brief
//! instantly (for the screenshot tour and `--mock`), and a Gemini-backed
//! real implementation. The TUI calls `generate` from a worker thread so the
//! spinner keeps animating while the call is in flight.
//!
//! A request is either *fresh* (no prior — produce a brief for the topic) or a
//! *retune* (carry a prior brief whose knob values are the new targets — keep
//! the structure but re-phrase every question to match the new depth / novelty
//! / applied levels). Knobs are regeneration inputs, not display state.

use crate::brief::TuiBrief;

/// What a generator is asked to produce.
pub struct BriefRequest {
    /// The user-facing topic.
    pub topic: String,
    /// Target output language.
    pub language: String,
    /// When present, regenerate from this take using its knob values as targets.
    pub prior: Option<TuiBrief>,
}

impl BriefRequest {
    /// A first-pass request for a topic.
    pub fn fresh(topic: &str, language: &str) -> Self {
        Self {
            topic: topic.to_string(),
            language: language.to_string(),
            prior: None,
        }
    }

    /// A retune request carrying the edited brief whose knobs are the targets.
    pub fn retune(prior: TuiBrief) -> Self {
        Self {
            topic: prior.topic.clone(),
            language: prior.language.clone(),
            prior: Some(prior),
        }
    }
}

/// Source of new briefs.
pub trait BriefGenerator: Send + Sync {
    /// Build (or retune) a 3×3 brief per the request.
    fn generate(&self, req: &BriefRequest) -> Result<TuiBrief, BriefError>;
}

/// Things that can go wrong during brief generation.
#[derive(Clone, Debug)]
pub enum BriefError {
    /// Network/transport failure.
    Network(String),
    /// Response parsed fine but did not match the schema.
    Parse(String),
    /// Response was empty.
    Empty,
    /// API key missing — caller should fall back to mock.
    NoApiKey,
}

impl std::fmt::Display for BriefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(m) => write!(f, "network error: {m}"),
            Self::Parse(m) => write!(f, "parse error: {m}"),
            Self::Empty => write!(f, "empty response"),
            Self::NoApiKey => write!(f, "no API key"),
        }
    }
}
