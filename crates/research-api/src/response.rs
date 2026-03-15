use std::collections::HashSet;

use research_domain::result::CitationSource;

use crate::link::{self, Linkable};

/// Object representing API response.
pub trait Responded {
    /// Return run identifier.
    fn id(&self) -> &str;
    /// Return total cost.
    fn cost(&self) -> f64;
    /// Return raw response map.
    fn raw(&self) -> &serde_json::Value;
    /// Return true when completed.
    fn completed(&self) -> bool;
    /// Return true when failed.
    fn failed(&self) -> bool;
    /// Return output markdown.
    fn text(&self) -> &str;
    /// Return source list.
    fn sources(&self) -> Vec<CitationSource>;
}

/// API response wrapper.
pub struct Response {
    ident: String,
    status: String,
    content: String,
    expense: f64,
    payload: serde_json::Value,
    basis: Vec<serde_json::Value>,
    policy: link::Links,
}

impl Response {
    /// Create response from components.
    pub fn new(
        id: &str,
        status: &str,
        text: &str,
        cost: f64,
        raw: serde_json::Value,
        basis: Vec<serde_json::Value>,
        policy: link::Links,
    ) -> Self {
        Self {
            ident: id.to_string(),
            status: status.to_string(),
            content: text.to_string(),
            expense: cost,
            payload: raw,
            basis,
            policy,
        }
    }
}

impl Responded for Response {
    fn id(&self) -> &str {
        &self.ident
    }

    fn cost(&self) -> f64 {
        self.expense
    }

    fn raw(&self) -> &serde_json::Value {
        &self.payload
    }

    fn completed(&self) -> bool {
        self.status == "completed"
    }

    fn failed(&self) -> bool {
        self.status == "failed"
    }

    fn text(&self) -> &str {
        &self.content
    }

    fn sources(&self) -> Vec<CitationSource> {
        let mut seen = HashSet::new();
        let mut list: Vec<CitationSource> = Vec::new();
        for field in &self.basis {
            let citations = field
                .get("citations")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            for cite in &citations {
                let raw = cite.get("url").and_then(|v| v.as_str()).unwrap_or("");
                if raw.is_empty() {
                    continue;
                }
                let cleaned = self.policy.clean(raw);
                if cleaned.is_empty() || seen.contains(&cleaned) {
                    continue;
                }
                let excerpts = cite
                    .get("excerpts")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let note = excerpts.first().and_then(|v| v.as_str()).unwrap_or("");
                let head = cite.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let head = if head.is_empty() {
                    self.policy.domain(&cleaned)
                } else {
                    head.to_string()
                };
                seen.insert(cleaned.clone());
                list.push(CitationSource::new(&head, &cleaned, note));
            }
        }
        list
    }
}

/// Remove utm params from URL.
pub fn clean(text: &str) -> String {
    link::make().clean(text)
}

/// Strip tracking URLs from output text.
pub fn strip(text: &str) -> String {
    link::make().strip(text)
}

/// Extract domain from URL string.
pub fn domain(text: &str) -> String {
    link::make().domain(text)
}

/// Create response from map.
pub fn response(item: &serde_json::Value) -> Response {
    let policy = link::make();
    let output = item.get("output").and_then(|v| v.as_str()).unwrap_or("");
    let text = policy.strip(output);
    let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let cost = item.get("cost").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let raw = item.get("raw").cloned().unwrap_or(serde_json::json!({}));
    let basis = item
        .get("basis")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    Response::new(id, status, &text, cost, raw, basis, policy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;
    use research_domain::result::{Serialized, Sourced};

    #[test]
    fn the_response_returns_identifier() {
        let mut rng = ids::ids(15001);
        let ident = format!("trun_{}", ids::uuid(&mut rng));
        let item = response(&serde_json::json!({
            "id": ident,
            "status": "completed",
            "output": "",
            "basis": []
        }));
        assert_eq!(
            ident,
            item.id(),
            "Response identifier did not match provided value"
        );
    }

    #[test]
    fn the_response_detects_completed() {
        let mut rng = ids::ids(15003);
        let ident = format!("trun_{}", ids::uuid(&mut rng));
        let item = response(&serde_json::json!({
            "id": ident,
            "status": "completed",
            "output": "",
            "basis": []
        }));
        assert!(item.completed(), "Response was not detected as completed");
    }

    #[test]
    fn the_response_detects_failed() {
        let mut rng = ids::ids(15005);
        let ident = format!("trun_{}", ids::uuid(&mut rng));
        let item = response(&serde_json::json!({
            "id": ident,
            "status": "failed",
            "output": "",
            "basis": []
        }));
        assert!(item.failed(), "Response was not detected as failed");
    }

    #[test]
    fn the_response_returns_markdown() {
        let mut rng = ids::ids(15007);
        let output = format!(
            "# {}\n\n{}",
            ids::cyrillic(&mut rng, 6),
            ids::hiragana(&mut rng, 5)
        );
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": output,
            "basis": []
        }));
        assert_eq!(
            output,
            item.text(),
            "Response markdown did not match output"
        );
    }

    #[test]
    fn the_response_extracts_sources() {
        let mut rng = ids::ids(15009);
        let link = format!("https://example.com/{}", ids::uuid(&mut rng));
        let text = ids::cyrillic(&mut rng, 6);
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": "",
            "basis": [{"citations": [{"url": link, "title": "Test", "excerpts": [text]}]}]
        }));
        assert_eq!(
            1,
            item.sources().len(),
            "Response did not extract one source"
        );
    }

    #[test]
    fn the_response_deduplicates_sources() {
        let mut rng = ids::ids(15011);
        let link = format!("https://example.com/{}", ids::uuid(&mut rng));
        let text = ids::cyrillic(&mut rng, 6);
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": "",
            "basis": [
                {"citations": [{"url": link, "title": "A", "excerpts": [text]}]},
                {"citations": [{"url": link, "title": "B", "excerpts": [text]}]}
            ]
        }));
        assert_eq!(
            1,
            item.sources().len(),
            "Response did not deduplicate sources"
        );
    }

    #[test]
    fn the_response_parses_data() {
        let mut rng = ids::ids(15013);
        let ident = format!("trun_{}", ids::uuid(&mut rng));
        let item = response(&serde_json::json!({
            "id": ident,
            "status": "completed",
            "output": "markdown",
            "basis": []
        }));
        assert_eq!(ident, item.id(), "Parsed response identifier did not match");
    }

    #[test]
    fn the_response_handles_empty_basis() {
        let mut rng = ids::ids(15015);
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": "",
            "basis": []
        }));
        assert_eq!(
            0,
            item.sources().len(),
            "Response sources was not empty for empty basis"
        );
    }

    #[test]
    fn the_response_omits_confidence_from_basis() {
        let mut rng = ids::ids(15017);
        let confidence = ids::cyrillic(&mut rng, 5);
        let link = format!("https://example.com/{}", ids::uuid(&mut rng));
        let text = ids::cyrillic(&mut rng, 6);
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": "",
            "basis": [{"citations": [{"url": link, "title": "T", "excerpts": [text]}], "confidence": confidence}]
        }));
        let source = item.sources().into_iter().next().unwrap();
        let data = source.data();
        assert!(
            !data.contains_key("confidence"),
            "Response sources contained confidence"
        );
    }

    #[test]
    fn the_response_omits_confidence_when_missing() {
        let mut rng = ids::ids(15019);
        let link = format!("https://example.com/{}", ids::uuid(&mut rng));
        let text = ids::cyrillic(&mut rng, 6);
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": "",
            "basis": [{"citations": [{"url": link, "title": "T", "excerpts": [text]}]}]
        }));
        let source = item.sources().into_iter().next().unwrap();
        let data = source.data();
        assert!(
            !data.contains_key("confidence"),
            "Response sources contained confidence"
        );
    }

    #[test]
    fn the_response_returns_cost() {
        let mut rng = ids::ids(15021);
        let value = ids::digit(&mut rng, 10000) as f64 / 100.0;
        let output = ids::cyrillic(&mut rng, 6);
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": output,
            "basis": [],
            "cost": value
        }));
        assert_eq!(value, item.cost(), "Cost did not return expected value");
    }

    #[test]
    fn the_response_strips_utm_from_markdown() {
        let mut rng = ids::ids(15023);
        let slug = ids::cyrillic(&mut rng, 6);
        let num = ids::digit(&mut rng, 1000);
        let extra = ids::digit(&mut rng, 9);
        let link = format!(
            "https://example.com/{}?utm_source=valyu.ai&utm_medium=referral&x={}",
            num, extra
        );
        let output = format!("Sources {}\n1. {}", slug, link);
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": output,
            "basis": []
        }));
        assert!(
            !item.text().contains("utm_source"),
            "utm parameters were not stripped from markdown"
        );
    }

    #[test]
    fn the_response_strips_utm_from_sources() {
        let mut rng = ids::ids(15025);
        let slug = ids::hiragana(&mut rng, 5);
        let num = ids::digit(&mut rng, 1000);
        let extra = ids::digit(&mut rng, 9);
        let link = format!(
            "https://example.com/{}?utm_source=valyu.ai&utm_medium=referral&x={}",
            num, extra
        );
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": slug,
            "basis": [{"citations": [{"url": link, "title": slug, "excerpts": [slug]}]}]
        }));
        let source = item.sources().into_iter().next().unwrap();
        assert!(
            !source.url().contains("utm_source"),
            "utm parameters were not stripped from sources"
        );
    }

    #[test]
    fn the_response_preserves_signed_urls() {
        let mut rng = ids::ids(15027);
        let text = ids::cyrillic(&mut rng, 6);
        let key = ids::greek(&mut rng, 4);
        let val = ids::armenian(&mut rng, 4);
        let num = ids::digit(&mut rng, 1000);
        let sig = ids::digit(&mut rng, 1000);
        let link = format!("https://example.com/{}?{}={}&sig={}", num, key, val, sig);
        let output = format!("{} {}", text, link);
        let item = response(&serde_json::json!({
            "id": ids::uuid(&mut rng),
            "status": "completed",
            "output": output,
            "basis": []
        }));
        assert_eq!(
            output,
            item.text(),
            "URL was changed despite missing utm parameters"
        );
    }
}
