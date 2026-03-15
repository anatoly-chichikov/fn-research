use crate::ids;
use crate::provider::Provider;
use crate::result::{self, Serialized};
use crate::task::{self, Tasked};

#[test]
fn the_task_generates_unique_id() {
    let mut rng = ids::ids(11001);
    let time = ids::time(&mut rng);
    let query = ids::cyrillic(&mut rng, 6);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 6);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "query": query,
        "status": status,
        "language": language,
        "service": "parallel.ai",
        "result": value.data(),
        "created": time
    }));
    assert_eq!(36, item.id().len(), "Task identifier length is incorrect");
}

#[test]
fn the_task_returns_provided_query() {
    let mut rng = ids::ids(11003);
    let time = ids::time(&mut rng);
    let query = ids::hiragana(&mut rng, 7);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": "valyu.ai",
        "result": value.data(),
        "created": time
    }));
    let text = item.query();
    let ok = text.contains(&language) && text.ends_with(&query);
    assert!(ok, "Task query did not include language and query");
}

#[test]
fn the_task_returns_provided_status() {
    let mut rng = ids::ids(11005);
    let time = ids::time(&mut rng);
    let status = ids::greek(&mut rng, 6);
    let query = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": "parallel.ai",
        "result": value.data(),
        "created": time
    }));
    assert_eq!(
        status,
        item.status(),
        "Task status did not match provided value"
    );
}

#[test]
fn the_task_complete_returns_new_task() {
    let mut rng = ids::ids(11007);
    let time = ids::time(&mut rng);
    let query = ids::cyrillic(&mut rng, 6);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 5);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": "x.ai",
        "processor": "social",
        "result": value.data(),
        "created": time
    }));
    let output = item.finish(result::Report::Full(result::ResearchReport::new(
        &summary,
        vec![],
    )));
    assert_eq!(
        "completed",
        output.status(),
        "Completed task status was not completed"
    );
}

#[test]
fn the_task_complete_preserves_id() {
    let mut rng = ids::ids(11009);
    let time = ids::time(&mut rng);
    let query = ids::cyrillic(&mut rng, 6);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": "parallel.ai",
        "processor": "ultra",
        "result": value.data(),
        "created": time
    }));
    let output = item.finish(result::Report::Full(result::ResearchReport::new(
        &summary,
        vec![],
    )));
    assert_eq!(
        item.id(),
        output.id(),
        "Completed task ID did not match original"
    );
}

#[test]
fn the_task_omits_query_serialization() {
    let mut rng = ids::ids(11013);
    let time = ids::time(&mut rng);
    let topic_part = ids::hiragana(&mut rng, 6);
    let item_part = ids::greek(&mut rng, 4);
    let query = format!("{}\n\nResearch:\n{}", topic_part, item_part);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": "parallel.ai",
        "processor": "ultra",
        "result": value.data(),
        "created": time
    }));
    let data = item.data();
    let brief = data.get("brief").unwrap();
    let questions = brief.get("questions").unwrap().as_array().unwrap();
    let node = &questions[0];
    let ok = data.contains_key("brief")
        && brief.get("title").is_some()
        && brief.get("questions").is_some()
        && data.get("processor").map(|v| v.as_str().unwrap()) == Some("ultra")
        && node.get("scope").is_some()
        && node.get("details").is_some()
        && brief.get("text").is_none()
        && !data.contains_key("query")
        && !data.contains_key("result");
    assert!(
        ok,
        "Serialized task did not include brief or still included query or result"
    );
}

#[test]
fn the_task_renders_nested_brief_items() {
    let mut rng = ids::ids(11017);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let first = ids::greek(&mut rng, 5);
    let inner = ids::armenian(&mut rng, 5);
    let second = ids::hiragana(&mut rng, 5);
    let status = ids::greek(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let brief = serde_json::json!({
        "title": topic,
        "language": language,
        "questions": [
            {"scope": first, "details": [{"scope": inner, "details": []}]},
            {"scope": second, "details": []}
        ]
    });
    let item = task::task(&serde_json::json!({
        "id": id,
        "brief": brief,
        "status": status,
        "language": language,
        "service": "parallel.ai",
        "processor": "pro",
        "result": value.data(),
        "created": time
    }));
    let text = item.query();
    let ok = text.contains(&language)
        && text.contains(&topic)
        && text.contains(&first)
        && text.contains(&inner)
        && text.contains(&second);
    assert!(ok, "Nested brief was not rendered");
}

#[test]
fn the_task_prefers_explicit_title_in_brief() {
    let mut rng = ids::ids(11021);
    let time = ids::time(&mut rng);
    let mark = ids::hiragana(&mut rng, 6);
    let topic_part = ids::greek(&mut rng, 6);
    let item_part = ids::armenian(&mut rng, 4);
    let query = format!("{}\n\nResearch:\n{}", topic_part, item_part);
    let id = ids::uuid(&mut rng);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "parallel.ai",
        "created": time,
        "topic": mark
    }));
    let brief = item.brief();
    assert_eq!(mark, brief.title, "Task brief did not use explicit title");
}

#[test]
fn the_task_deserializes_correctly() {
    let mut rng = ids::ids(11015);
    let time = ids::time(&mut rng);
    let query = ids::cyrillic(&mut rng, 7);
    let status = ids::greek(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": "parallel.ai",
        "result": value.data(),
        "created": time
    }));
    let text = item.query();
    let ok = text.contains(&language) && text.ends_with(&query);
    assert!(ok, "Deserialized query did not include language and query");
}

#[test]
fn the_task_parses_nested_query_items() {
    let mut rng = ids::ids(11019);
    let topic = ids::cyrillic(&mut rng, 6);
    let head = ids::greek(&mut rng, 5);
    let inner = ids::armenian(&mut rng, 5);
    let tail = ids::hiragana(&mut rng, 5);
    let query = format!("{}\n\nResearch:\n{}\n\t{}\n{}", topic, head, inner, tail);
    let id = ids::uuid(&mut rng);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "valyu.ai",
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let questions = &brief.questions;
    let node = &questions[0];
    let peer = &questions[1];
    let ok = node.scope == head && node.details[0].scope == inner && peer.scope == tail;
    assert!(ok, "Nested query items were not parsed");
}

#[test]
fn the_task_parses_compound_numbered_items_at_depth_two() {
    let mut rng = ids::ids(11023);
    let root = ids::greek(&mut rng, 5);
    let child = ids::armenian(&mut rng, 5);
    let sibling = ids::hiragana(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n1. {}\n  1.1. {}\n2. {}",
        topic_part, root, child, sibling
    );
    let id = ids::uuid(&mut rng);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "parallel.ai",
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let node = &brief.questions[0];
    assert_eq!(
        child, node.details[0].scope,
        "Compound numbered sub-item was not nested under parent"
    );
}

#[test]
fn the_task_parses_indented_compound_items_at_depth_two() {
    let mut rng = ids::ids(11025);
    let root = ids::hebrew(&mut rng, 5);
    let child = ids::armenian(&mut rng, 5);
    let tail = ids::greek(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n1. {}\n    1.1. {}\n2. {}",
        topic_part, root, child, tail
    );
    let id = ids::uuid(&mut rng);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "x.ai",
        "processor": "social",
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let node = &brief.questions[0];
    assert_eq!(
        child, node.details[0].scope,
        "Indented compound item was not nested at depth two"
    );
}

#[test]
fn the_task_parses_triple_compound_items_at_depth_three() {
    let mut rng = ids::ids(11027);
    let root = ids::greek(&mut rng, 5);
    let mid = ids::armenian(&mut rng, 5);
    let leaf = ids::hebrew(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n1. {}\n  1.1. {}\n    1.1.1. {}",
        topic_part, root, mid, leaf
    );
    let id = ids::uuid(&mut rng);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "parallel.ai",
        "created": "2026-01-01T00:00:00"
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
fn the_task_parses_tab_indented_items() {
    let mut rng = ids::ids(11029);
    let root = ids::greek(&mut rng, 5);
    let child = ids::armenian(&mut rng, 5);
    let sibling = ids::hiragana(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n{}\n\t{}\n{}",
        topic_part, root, child, sibling
    );
    let id = ids::uuid(&mut rng);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "parallel.ai",
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let node = &brief.questions[0];
    assert_eq!(
        child, node.details[0].scope,
        "Tab-indented sub-item was not nested under parent"
    );
}

#[test]
fn the_task_renders_numbered_brief() {
    let mut rng = ids::ids(11033);
    let time = ids::time(&mut rng);
    let root = ids::greek(&mut rng, 5);
    let child = ids::armenian(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!("{}\n\nResearch:\n{}\n\t{}", topic_part, root, child);
    let id = ids::uuid(&mut rng);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "valyu.ai",
        "processor": "heavy",
        "created": time
    }));
    let text = item.query();
    assert!(
        text.contains("1.1. "),
        "Rendered brief did not contain hierarchical numbering"
    );
}

#[test]
fn the_task_parses_double_tab_items_at_depth_three() {
    let mut rng = ids::ids(11031);
    let root = ids::greek(&mut rng, 5);
    let mid = ids::armenian(&mut rng, 5);
    let leaf = ids::hebrew(&mut rng, 5);
    let topic_part = ids::cyrillic(&mut rng, 6);
    let query = format!(
        "{}\n\nResearch:\n{}\n\t{}\n\t\t{}",
        topic_part, root, mid, leaf
    );
    let id = ids::uuid(&mut rng);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "parallel.ai",
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let node = &brief.questions[0];
    let sub = &node.details[0];
    assert_eq!(
        leaf, sub.details[0].scope,
        "Double-tab item was not nested at depth three"
    );
}

#[test]
fn the_task_parses_legacy_xai_ai_service() {
    let mut rng = ids::ids(11035);
    let time = ids::time(&mut rng);
    let query = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let summary = ids::cyrillic(&mut rng, 6);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "xai.ai",
        "processor": "social",
        "result": value.data(),
        "created": time
    }));
    assert_eq!(
        &Provider::Xai,
        item.provider(),
        "Legacy xai.ai service was not parsed as Xai"
    );
}

#[test]
fn the_task_serializes_service_as_label() {
    let mut rng = ids::ids(11037);
    let time = ids::time(&mut rng);
    let query = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let summary = ids::cyrillic(&mut rng, 6);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": "completed",
        "language": "English",
        "service": "x.ai",
        "processor": "full",
        "result": value.data(),
        "created": time
    }));
    let data = item.data();
    assert_eq!(
        "x.ai",
        data.get("service").unwrap().as_str().unwrap(),
        "Serialized service was not the label form"
    );
}
