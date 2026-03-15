use rand::RngCore;

use crate::ids;
use crate::result::{self, CitationSource, Listed, Serialized, Sourced, Summarized};

#[test]
fn the_source_returns_provided_title() {
    let mut rng = ids::ids(14001);
    let title = ids::cyrillic(&mut rng, 6);
    let item = CitationSource::new(&title, "https://example.com", "text");
    assert_eq!(
        title,
        item.title(),
        "Source title did not match provided value"
    );
}

#[test]
fn the_source_returns_provided_url() {
    let mut rng = ids::ids(14003);
    let id = ids::uuid(&mut rng);
    let url = format!("https://example.com/{}", id);
    let item = CitationSource::new("Title", &url, "text");
    assert_eq!(url, item.url(), "Source URL did not match provided value");
}

#[test]
fn the_source_serializes_all_fields() {
    let mut rng = ids::ids(14005);
    let excerpt = ids::cyrillic(&mut rng, 6);
    let item = CitationSource::new("T", "https://x.com", &excerpt);
    let data = item.data();
    assert_eq!(
        excerpt,
        data.get("excerpt").unwrap().as_str().unwrap(),
        "Serialized excerpt did not match original"
    );
}

#[test]
fn the_source_deserializes_from_map() {
    let mut rng = ids::ids(14007);
    let title = ids::cyrillic(&mut rng, 6);
    let data = serde_json::json!({
        "title": title,
        "url": "https://x.com",
        "excerpt": "e"
    });
    let item = result::source(&data);
    assert_eq!(
        title,
        item.title(),
        "Deserialized source title did not match"
    );
}

#[test]
fn the_result_returns_summary() {
    let mut rng = ids::ids(14009);
    let summary = ids::cyrillic(&mut rng, 6);
    let item = result::ResearchReport::new(&summary, vec![]);
    assert_eq!(
        summary,
        item.summary(),
        "Result summary did not match provided value"
    );
}

#[test]
fn the_result_strips_sources_section() {
    let mut rng = ids::ids(14011);
    let slug = ids::cyrillic(&mut rng, 6);
    let num = rng.next_u32() % 1000;
    let link = format!("https://example.com/{}", num);
    let summary = format!("Introduction {}\n\n## Sources\n1. {}", slug, link);
    let item = result::ResearchReport::new(&summary, vec![]);
    assert!(
        !item.summary().contains("## Sources"),
        "Sources section was not stripped"
    );
}

#[test]
fn the_result_returns_sources() {
    let mut rng = ids::ids(14013);
    let text = ids::cyrillic(&mut rng, 6);
    let item =
        result::ResearchReport::new("s", vec![CitationSource::new("T", "https://x.com", &text)]);
    assert_eq!(1, item.sources().len(), "Result sources count was not one");
}

#[test]
fn the_result_serializes_correctly() {
    let mut rng = ids::ids(14015);
    let summary = ids::cyrillic(&mut rng, 6);
    let item = result::ResearchReport::new(&summary, vec![]);
    let data = item.data();
    assert_eq!(
        summary,
        data.get("summary").unwrap().as_str().unwrap(),
        "Serialized summary did not match original"
    );
}

#[test]
fn the_result_deserializes_correctly() {
    let mut rng = ids::ids(14017);
    let summary = ids::cyrillic(&mut rng, 6);
    let val = serde_json::json!({
        "summary": summary,
        "sources": []
    });
    let item = result::result(Some(&val));
    assert_eq!(
        summary,
        item.summary(),
        "Deserialized summary did not match"
    );
}

#[test]
fn the_source_omits_confidence_on_serialization() {
    let mut rng = ids::ids(14019);
    let excerpt = ids::cyrillic(&mut rng, 5);
    let item = CitationSource::new("T", "https://example.com", &excerpt);
    let data = item.data();
    assert!(
        !data.contains_key("confidence"),
        "Serialized source contained confidence"
    );
}

#[test]
fn the_source_ignores_confidence_from_map() {
    let mut rng = ids::ids(14021);
    let confidence = ids::cyrillic(&mut rng, 5);
    let val = serde_json::json!({
        "title": "T",
        "url": "https://x.com",
        "excerpt": "e",
        "confidence": confidence
    });
    let item = result::source(&val);
    let data = item.data();
    assert!(
        !data.contains_key("confidence"),
        "Deserialized source contained confidence"
    );
}
