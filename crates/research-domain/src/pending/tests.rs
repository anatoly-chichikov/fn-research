use crate::ids;
use crate::pending::{self, Pendinged};
use crate::processor::{ParallelMode, Processor};
use crate::provider::Provider;

#[test]
fn the_pending_returns_identifier() {
    let mut rng = ids::ids(13001);
    let run = ids::cyrillic(&mut rng, 6);
    let topic_part = ids::hiragana(&mut rng, 6);
    let item_part = ids::greek(&mut rng, 4);
    let query = format!("{}\n\nResearch:\n{}", topic_part, item_part);
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "ultra",
        "language": "English",
        "provider": "parallel"
    }));
    assert_eq!(
        run,
        item.id(),
        "Pending identifier did not match provided value"
    );
}

#[test]
fn the_pending_returns_query() {
    let mut rng = ids::ids(13003);
    let run = ids::cyrillic(&mut rng, 6);
    let topic = ids::hiragana(&mut rng, 6);
    let item_text = ids::greek(&mut rng, 4);
    let query = format!("{}\n\nResearch:\n{}", topic, item_text);
    let pend = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "pro",
        "language": "Russian",
        "provider": "parallel"
    }));
    let text = pend.query();
    let ok = text.contains("Russian") && text.contains(&topic) && text.contains(&item_text);
    assert!(ok, "Pending query did not include language and query");
}

#[test]
fn the_pending_returns_processor() {
    let mut rng = ids::ids(13005);
    let run = ids::cyrillic(&mut rng, 6);
    let topic_part = ids::hiragana(&mut rng, 6);
    let item_part = ids::greek(&mut rng, 4);
    let query = format!("{}\n\nResearch:\n{}", topic_part, item_part);
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "ultra8x",
        "language": "English",
        "provider": "parallel"
    }));
    assert_eq!(
        &Processor::Parallel(ParallelMode::Ultra8x),
        item.processor(),
        "Pending processor did not match provided value"
    );
}

#[test]
fn the_pending_returns_language() {
    let mut rng = ids::ids(13007);
    let run = ids::cyrillic(&mut rng, 6);
    let topic_part = ids::hiragana(&mut rng, 6);
    let item_part = ids::greek(&mut rng, 4);
    let query = format!("{}\n\nResearch:\n{}", topic_part, item_part);
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "pro",
        "language": "Greek",
        "provider": "parallel"
    }));
    assert_eq!(
        "Greek",
        item.language(),
        "Pending language did not match provided value"
    );
}

#[test]
fn the_pending_serializes_correctly() {
    let mut rng = ids::ids(13009);
    let run = ids::cyrillic(&mut rng, 6);
    let topic_part = ids::hiragana(&mut rng, 6);
    let item_part = ids::greek(&mut rng, 4);
    let query = format!("{}\n\nResearch:\n{}", topic_part, item_part);
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "ultra",
        "language": "English",
        "provider": "parallel"
    }));
    let data = item.data();
    let brief_val = data.get("brief").unwrap();
    let brief_obj: &serde_json::Map<String, serde_json::Value> = brief_val.as_object().unwrap();
    let questions = brief_obj.get("questions").unwrap().as_array().unwrap();
    let node = &questions[0];
    let ok = data.contains_key("run_id")
        && data.contains_key("processor")
        && data.contains_key("language")
        && data.contains_key("brief")
        && brief_obj.contains_key("title")
        && brief_obj.contains_key("questions")
        && node.get("scope").is_some()
        && node.get("details").is_some()
        && !brief_obj.contains_key("text")
        && !data.contains_key("query");
    assert!(
        ok,
        "Pending serialize did not include brief or still included query"
    );
}

#[test]
fn the_pending_parses_nested_query_items() {
    let mut rng = ids::ids(13010);
    let run = ids::cyrillic(&mut rng, 6);
    let head = ids::greek(&mut rng, 5);
    let inner = ids::armenian(&mut rng, 5);
    let tail = ids::hiragana(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n{}\n\t{}\n{}",
        topic_part, head, inner, tail
    );
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "pro",
        "language": "English",
        "provider": "parallel"
    }));
    let brief = item.brief();
    let questions = &brief.questions;
    let node = &questions[0];
    let peer = &questions[1];
    let ok = node.scope == head && node.details[0].scope == inner && peer.scope == tail;
    assert!(ok, "Pending nested query items were not parsed");
}

#[test]
fn the_pending_deserializes_correctly() {
    let mut rng = ids::ids(13011);
    let run = ids::cyrillic(&mut rng, 6);
    let query = ids::hiragana(&mut rng, 6);
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "fast",
        "language": "English",
        "provider": "valyu"
    }));
    assert_eq!(
        run,
        item.id(),
        "Pending deserialize did not restore identifier"
    );
}

#[test]
fn the_pending_returns_provider() {
    let mut rng = ids::ids(13013);
    let run = ids::cyrillic(&mut rng, 6);
    let query = ids::hiragana(&mut rng, 6);
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "social",
        "language": "English",
        "provider": "xai"
    }));
    assert_eq!(
        &Provider::Xai,
        item.provider(),
        "Pending provider did not match provided value"
    );
}

#[test]
fn the_pending_prefers_explicit_title_in_brief() {
    let mut rng = ids::ids(13017);
    let run = ids::cyrillic(&mut rng, 6);
    let mark = ids::hiragana(&mut rng, 6);
    let topic_part = ids::greek(&mut rng, 6);
    let item_part = ids::armenian(&mut rng, 4);
    let query = format!("{}\n\nResearch:\n{}", topic_part, item_part);
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "pro",
        "language": "English",
        "provider": "parallel",
        "topic": mark
    }));
    let brief = item.brief();
    assert_eq!(
        mark, brief.title,
        "Pending brief did not use explicit title"
    );
}

#[test]
fn the_pending_serializes_provider() {
    let mut rng = ids::ids(13015);
    let run = ids::cyrillic(&mut rng, 6);
    let query = ids::hiragana(&mut rng, 6);
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "standard",
        "language": "English",
        "provider": "valyu"
    }));
    let data = item.data();
    assert_eq!(
        "valyu",
        data.get("provider").unwrap().as_str().unwrap(),
        "Pending serialize did not include provider"
    );
}

#[test]
fn the_pending_parses_compound_numbered_items_at_depth_two() {
    let mut rng = ids::ids(13019);
    let run = ids::cyrillic(&mut rng, 6);
    let root = ids::greek(&mut rng, 5);
    let child = ids::armenian(&mut rng, 5);
    let sibling = ids::hiragana(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n1. {}\n  1.1. {}\n2. {}",
        topic_part, root, child, sibling
    );
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "ultra",
        "language": "English",
        "provider": "parallel"
    }));
    let brief = item.brief();
    let node = &brief.questions[0];
    assert_eq!(
        child, node.details[0].scope,
        "Compound numbered sub-item was not nested under parent"
    );
}

#[test]
fn the_pending_parses_indented_compound_items_at_depth_two() {
    let mut rng = ids::ids(13021);
    let run = ids::cyrillic(&mut rng, 6);
    let root = ids::hebrew(&mut rng, 5);
    let child = ids::armenian(&mut rng, 5);
    let tail = ids::greek(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n1. {}\n    1.1. {}\n2. {}",
        topic_part, root, child, tail
    );
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "pro",
        "language": "English",
        "provider": "parallel"
    }));
    let brief = item.brief();
    let node = &brief.questions[0];
    assert_eq!(
        child, node.details[0].scope,
        "Indented compound item was not nested at depth two"
    );
}

#[test]
fn the_pending_parses_triple_compound_items_at_depth_three() {
    let mut rng = ids::ids(13023);
    let run = ids::cyrillic(&mut rng, 6);
    let root = ids::greek(&mut rng, 5);
    let mid = ids::armenian(&mut rng, 5);
    let leaf = ids::hebrew(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n1. {}\n  1.1. {}\n    1.1.1. {}",
        topic_part, root, mid, leaf
    );
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "ultra",
        "language": "English",
        "provider": "parallel"
    }));
    let brief = item.brief();
    let node = &brief.questions[0];
    let sub = &node.details[0];
    assert_eq!(
        leaf, sub.details[0].scope,
        "Triple compound item was not nested at depth three"
    );
}

#[test]
fn the_pending_parses_tab_indented_items() {
    let mut rng = ids::ids(13025);
    let run = ids::cyrillic(&mut rng, 6);
    let root = ids::greek(&mut rng, 5);
    let child = ids::armenian(&mut rng, 5);
    let sibling = ids::hiragana(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n{}\n\t{}\n{}",
        topic_part, root, child, sibling
    );
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "pro",
        "language": "English",
        "provider": "parallel"
    }));
    let brief = item.brief();
    let node = &brief.questions[0];
    assert_eq!(
        child, node.details[0].scope,
        "Tab-indented sub-item was not nested under parent"
    );
}

#[test]
fn the_pending_renders_numbered_brief() {
    let mut rng = ids::ids(13029);
    let run = ids::cyrillic(&mut rng, 6);
    let root = ids::greek(&mut rng, 5);
    let child = ids::armenian(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!("{}\n\nResearch:\n{}\n\t{}", topic_part, root, child);
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "pro",
        "language": "English",
        "provider": "parallel"
    }));
    let text = item.query();
    assert!(
        text.contains("1.1. "),
        "Rendered brief did not contain hierarchical numbering"
    );
}

#[test]
fn the_pending_parses_double_tab_items_at_depth_three() {
    let mut rng = ids::ids(13027);
    let run = ids::cyrillic(&mut rng, 6);
    let root = ids::greek(&mut rng, 5);
    let mid = ids::armenian(&mut rng, 5);
    let leaf = ids::hebrew(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n{}\n\t{}\n\t\t{}",
        topic_part, root, mid, leaf
    );
    let item = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "pro",
        "language": "English",
        "provider": "parallel"
    }));
    let brief = item.brief();
    let node = &brief.questions[0];
    let sub = &node.details[0];
    assert_eq!(
        leaf, sub.details[0].scope,
        "Double-tab item was not nested at depth three"
    );
}
