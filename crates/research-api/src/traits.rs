use research_domain::result::CitationSource;

/// Object that can execute research.
pub trait Researchable {
    /// Start research and return run identifier.
    fn start(&self, query: &str, processor: &str) -> String;
    /// Stream progress updates.
    fn stream(&self, id: &str);
    /// Finish research and return response.
    fn finish(&self, id: &str) -> serde_json::Value;
}

/// Object that can build citation basis.
pub trait Grounded {
    /// Return basis entries.
    fn basis(&self, sources: &[CitationSource]) -> Vec<serde_json::Value>;
}
