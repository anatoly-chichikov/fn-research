use crate::ids;
use crate::result::{self, Serialized};
use crate::task::{self, Tasked};

#[test]
fn the_task_generates_unique_id() {
    let mut rng = ids::ids(11001);
    let time = ids::time(&mut rng);
    let query = ids::cyrillic(&mut rng, 6);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 6);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "query": query,
        "status": status,
        "language": language,
        "service": service,
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
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": service,
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
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": service,
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
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 5);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": service,
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
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": service,
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
fn the_task_complete_adds_timestamp() {
    let mut rng = ids::ids(11011);
    let time = ids::time(&mut rng);
    let query = ids::cyrillic(&mut rng, 6);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": service,
        "result": value.data(),
        "created": time
    }));
    let output = item.finish(result::Report::Full(result::ResearchReport::new(
        &summary,
        vec![],
    )));
    assert!(
        output.completed().is_some(),
        "Completed task timestamp was missing"
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
    let processor = ids::greek(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": service,
        "processor": processor,
        "result": value.data(),
        "created": time
    }));
    let data = item.data();
    let brief = data.get("brief").unwrap();
    let items = brief.get("items").unwrap().as_array().unwrap();
    let node = &items[0];
    let ok = data.contains_key("brief")
        && brief.get("topic").is_some()
        && brief.get("items").is_some()
        && data.get("processor").map(|v| v.as_str().unwrap()) == Some(&processor)
        && node.get("text").is_some()
        && node.get("items").is_some()
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
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let brief = serde_json::json!({
        "topic": topic,
        "items": [
            {"text": first, "items": [{"text": inner, "items": []}]},
            {"text": second, "items": []}
        ]
    });
    let item = task::task(&serde_json::json!({
        "id": id,
        "brief": brief,
        "status": status,
        "language": language,
        "service": service,
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
fn the_task_prefers_explicit_topic_in_brief() {
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
        "status": ids::greek(&mut rng, 6),
        "language": ids::cyrillic(&mut rng, 5),
        "service": ids::cyrillic(&mut rng, 4),
        "created": time,
        "topic": mark
    }));
    let brief = item.brief();
    assert_eq!(mark, brief.topic, "Task brief did not use explicit topic");
}

#[test]
fn the_task_deserializes_correctly() {
    let mut rng = ids::ids(11015);
    let time = ids::time(&mut rng);
    let query = ids::cyrillic(&mut rng, 7);
    let status = ids::greek(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let value = result::ResearchReport::new(&summary, vec![]);
    let item = task::task(&serde_json::json!({
        "id": id,
        "query": query,
        "status": status,
        "language": language,
        "service": service,
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
        "status": ids::greek(&mut rng, 6),
        "language": ids::cyrillic(&mut rng, 5),
        "service": ids::cyrillic(&mut rng, 4),
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let items = &brief.items;
    let node = &items[0];
    let peer = &items[1];
    let ok = node.text == head && node.items[0].text == inner && peer.text == tail;
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
        "status": ids::greek(&mut rng, 6),
        "language": ids::cyrillic(&mut rng, 5),
        "service": ids::cyrillic(&mut rng, 4),
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let node = &brief.items[0];
    assert_eq!(
        child, node.items[0].text,
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
        "status": ids::greek(&mut rng, 6),
        "language": ids::cyrillic(&mut rng, 5),
        "service": ids::cyrillic(&mut rng, 4),
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let node = &brief.items[0];
    assert_eq!(
        child, node.items[0].text,
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
        "status": ids::greek(&mut rng, 6),
        "language": ids::cyrillic(&mut rng, 5),
        "service": ids::cyrillic(&mut rng, 4),
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let node = &brief.items[0];
    let sub = &node.items[0];
    assert_eq!(
        leaf, sub.items[0].text,
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
        "status": ids::greek(&mut rng, 6),
        "language": ids::cyrillic(&mut rng, 5),
        "service": ids::cyrillic(&mut rng, 4),
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let node = &brief.items[0];
    assert_eq!(
        child, node.items[0].text,
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
        "status": ids::greek(&mut rng, 6),
        "language": ids::cyrillic(&mut rng, 5),
        "service": ids::cyrillic(&mut rng, 4),
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
        "status": ids::greek(&mut rng, 6),
        "language": ids::cyrillic(&mut rng, 5),
        "service": ids::cyrillic(&mut rng, 4),
        "created": "2026-01-01T00:00:00"
    }));
    let brief = item.brief();
    let node = &brief.items[0];
    let sub = &node.items[0];
    assert_eq!(
        leaf, sub.items[0].text,
        "Double-tab item was not nested at depth three"
    );
}
