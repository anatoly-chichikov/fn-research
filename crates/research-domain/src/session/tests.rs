use std::collections::HashMap;

use crate::ids;
use crate::pending::{self, Pendinged};
use crate::processor::{ParallelMode, Processor, ValyuMode};
use crate::provider::Provider;
use crate::result::{self, Serialized};
use crate::session::{self, Sessioned};
use crate::task::{self, Tasked};

#[test]
fn the_session_generates_unique_id() {
    let mut rng = ids::ids(12001);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let item = session::session(&serde_json::json!({
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    assert_eq!(
        36,
        item.id().len(),
        "Session identifier length is incorrect"
    );
}

#[test]
fn the_session_returns_provided_topic() {
    let mut rng = ids::ids(12003);
    let time = ids::time(&mut rng);
    let topic = ids::hiragana(&mut rng, 7);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    assert_eq!(
        topic,
        item.topic(),
        "Session topic did not match provided value"
    );
}

#[test]
fn the_session_extend_adds_task() {
    let mut rng = ids::ids(12005);
    let time = ids::time(&mut rng);
    let topic = ids::greek(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    let query = ids::cyrillic(&mut rng, 6);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 6);
    let value = result::ResearchReport::new(&summary, vec![]);
    let tid = ids::uuid(&mut rng);
    let run = task::task(&serde_json::json!({
        "id": tid,
        "query": query,
        "status": status,
        "language": language,
        "service": "parallel.ai",
        "result": value.data(),
        "created": time
    }));
    let output = item.extend(run);
    assert_eq!(
        1,
        output.tasks().len(),
        "Extended session did not contain one task"
    );
}

#[test]
fn the_session_extend_preserves_id() {
    let mut rng = ids::ids(12007);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let ident = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": ident,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    let query = ids::cyrillic(&mut rng, 6);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let summary = ids::cyrillic(&mut rng, 6);
    let value = result::ResearchReport::new(&summary, vec![]);
    let tid = ids::uuid(&mut rng);
    let run = task::task(&serde_json::json!({
        "id": tid,
        "query": query,
        "status": status,
        "language": language,
        "service": "valyu.ai",
        "result": value.data(),
        "created": time
    }));
    let output = item.extend(run);
    assert_eq!(
        ident,
        output.id(),
        "Extended session ID did not match original"
    );
}

#[test]
fn the_session_serializes_topic() {
    let mut rng = ids::ids(12009);
    let time = ids::time(&mut rng);
    let topic = ids::hiragana(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    let data = item.data();
    assert_eq!(
        topic,
        data.get("topic").unwrap().as_str().unwrap(),
        "Serialized topic did not match original"
    );
}

#[test]
fn the_session_deserializes_correctly() {
    let mut rng = ids::ids(12011);
    let time = ids::time(&mut rng);
    let topic = ids::hiragana(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    assert_eq!(topic, item.topic(), "Deserialized topic did not match");
}

#[test]
fn the_session_pending_returns_empty() {
    let mut rng = ids::ids(12013);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    assert!(
        item.pending().is_none(),
        "Pending run was present for new session"
    );
}

#[test]
fn the_session_start_sets_pending() {
    let mut rng = ids::ids(12015);
    let time = ids::time(&mut rng);
    let run = ids::cyrillic(&mut rng, 6);
    let query = ids::hiragana(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": ids::cyrillic(&mut rng, 5),
        "tasks": [],
        "created": time
    }));
    let hold = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "ultra",
        "language": language,
        "provider": "parallel"
    }));
    let output = item.start(hold);
    assert_eq!(
        run,
        output.pending().unwrap().id(),
        "Pending run identifier did not match"
    );
}

#[test]
fn the_session_clear_removes_pending() {
    let mut rng = ids::ids(12017);
    let time = ids::time(&mut rng);
    let run = ids::cyrillic(&mut rng, 6);
    let query = ids::hiragana(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": ids::cyrillic(&mut rng, 5),
        "tasks": [],
        "created": time,
        "pending": {
            "run_id": run,
            "query": query,
            "processor": "pro",
            "language": language,
            "provider": "parallel"
        }
    }));
    let output = item.reset();
    assert!(output.pending().is_none(), "Pending run was not cleared");
}

#[test]
fn the_session_serializes_pending() {
    let mut rng = ids::ids(12019);
    let time = ids::time(&mut rng);
    let run = ids::cyrillic(&mut rng, 6);
    let query = ids::hiragana(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let hold = pending::pending(&serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": "ultra8x",
        "language": language,
        "provider": "parallel"
    }));
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": ids::cyrillic(&mut rng, 5),
        "tasks": [],
        "created": time
    }));
    let started = item.start(hold);
    let data = started.data();
    let pending_data = data.get("pending").unwrap();
    assert_eq!(
        run,
        pending_data.get("run_id").unwrap().as_str().unwrap(),
        "Serialized pending run_id did not match"
    );
}

#[test]
fn the_session_deserializes_pending() {
    let mut rng = ids::ids(12021);
    let time = ids::time(&mut rng);
    let run = ids::cyrillic(&mut rng, 6);
    let query = ids::hiragana(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": ids::cyrillic(&mut rng, 5),
        "tasks": [],
        "created": time,
        "pending": {
            "run_id": run,
            "query": query,
            "processor": "fast",
            "language": language,
            "provider": "valyu"
        }
    }));
    assert_eq!(
        run,
        item.pending().unwrap().id(),
        "Deserialized pending run did not match"
    );
}

#[test]
fn the_session_returns_provided_query() {
    let mut rng = ids::ids(12023);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let query = ids::greek(&mut rng, 7);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time,
        "query": query
    }));
    assert_eq!(
        query,
        item.query(),
        "Session query did not match provided value"
    );
}

#[test]
fn the_session_returns_provided_processor() {
    let mut rng = ids::ids(12025);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time,
        "processor": "ultra",
        "provider": "parallel"
    }));
    assert_eq!(
        &Processor::Parallel(ParallelMode::Ultra),
        item.processor(),
        "Session processor did not match provided value"
    );
}

#[test]
fn the_session_returns_provided_language() {
    let mut rng = ids::ids(12027);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let language = ids::hiragana(&mut rng, 5);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time,
        "language": language
    }));
    assert_eq!(
        language,
        item.language(),
        "Session language did not match provided value"
    );
}

#[test]
fn the_session_returns_provided_provider() {
    let mut rng = ids::ids(12029);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time,
        "provider": "valyu"
    }));
    assert_eq!(
        &Provider::Valyu,
        item.provider(),
        "Session provider did not match provided value"
    );
}

#[test]
fn the_session_reconfigure_updates_provider() {
    let mut rng = ids::ids(12031);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time,
        "provider": "parallel",
        "processor": "pro"
    }));
    let mut opts = HashMap::new();
    opts.insert("provider".to_string(), "valyu".to_string());
    opts.insert("processor".to_string(), "standard".to_string());
    let updated = item.reconfigure(&opts);
    assert_eq!(
        &Provider::Valyu,
        updated.provider(),
        "Reconfigured provider did not match"
    );
}

#[test]
fn the_session_reconfigure_updates_processor() {
    let mut rng = ids::ids(12035);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time,
        "provider": "valyu",
        "processor": "fast"
    }));
    let mut opts = HashMap::new();
    opts.insert("processor".to_string(), "heavy".to_string());
    let updated = item.reconfigure(&opts);
    assert_eq!(
        &Processor::Valyu(ValyuMode::Heavy),
        updated.processor(),
        "Reconfigured processor did not match"
    );
}

#[test]
fn the_session_serializes_research_params() {
    let mut rng = ids::ids(12033);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let query = ids::greek(&mut rng, 7);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time,
        "query": query,
        "processor": "ultra",
        "language": "English",
        "provider": "parallel"
    }));
    let data = item.data();
    assert_eq!(
        query,
        data.get("query").unwrap().as_str().unwrap(),
        "Serialized query did not match original"
    );
}

#[test]
fn the_session_defaults_to_parallel_provider() {
    let mut rng = ids::ids(12037);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    assert_eq!(
        &Provider::Parallel,
        item.provider(),
        "Session did not default to parallel provider"
    );
}

#[test]
fn the_session_parses_legacy_service_in_task() {
    let mut rng = ids::ids(12039);
    let time = ids::time(&mut rng);
    let topic = ids::cyrillic(&mut rng, 6);
    let id = ids::uuid(&mut rng);
    let query = ids::greek(&mut rng, 6);
    let summary = ids::cyrillic(&mut rng, 6);
    let value = result::ResearchReport::new(&summary, vec![]);
    let tid = ids::uuid(&mut rng);
    let item = session::session(&serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [{
            "id": tid,
            "query": query,
            "status": "completed",
            "language": "English",
            "service": "parallel.ai",
            "processor": "ultra",
            "result": value.data(),
            "created": time
        }],
        "created": time
    }));
    assert_eq!(
        &Provider::Parallel,
        item.tasks()[0].provider(),
        "Legacy parallel.ai service was not parsed"
    );
}
