use std::fs;

use research_domain::ids;
use research_domain::result::{self, Serialized};
use research_domain::session::{self, Sessioned};
use research_domain::task::{self, Tasked};

use super::{repo, Loadable, Mutable, Savable};
use tempfile::TempDir;

#[test]
fn the_repository_returns_empty_for_empty_folder() {
    let mut rng = ids::ids(24001);
    let dir = TempDir::new().unwrap();
    let mark = ids::ascii(&mut rng, 4);
    let name = format!("{}-{}", mark, ids::uuid(&mut rng));
    let path = dir.path().join(&name);
    fs::create_dir_all(&path).unwrap();
    let item = repo(dir.path());
    assert_eq!(
        0,
        item.load().len(),
        "Load did not return empty list for empty folder"
    );
}

#[test]
fn the_repository_saves_and_loads_session() {
    let mut rng = ids::ids(24003);
    let dir = TempDir::new().unwrap();
    let item = repo(dir.path());
    let topic = ids::cyrillic(&mut rng, 6);
    let ident = ids::uuid(&mut rng);
    let time = ids::time(&mut rng);
    let entry = session::session(&serde_json::json!({
        "id": ident,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    item.save(&[entry]);
    let loaded = item.load();
    assert_eq!(
        topic,
        loaded[0].topic(),
        "Loaded session topic did not match saved"
    );
}

#[test]
fn the_repository_append_adds_session() {
    let mut rng = ids::ids(24005);
    let dir = TempDir::new().unwrap();
    let item = repo(dir.path());
    let topic = ids::cyrillic(&mut rng, 6);
    let label = ids::cyrillic(&mut rng, 6);
    let time = ids::time(&mut rng);
    let alpha = session::session(&serde_json::json!({
        "id": ids::uuid(&mut rng),
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    let beta = session::session(&serde_json::json!({
        "id": ids::uuid(&mut rng),
        "topic": label,
        "tasks": [],
        "created": time
    }));
    item.append(alpha);
    item.append(beta);
    assert_eq!(
        2,
        item.load().len(),
        "Repository did not contain two sessions after append"
    );
}

#[test]
fn the_repository_find_returns_matching_session() {
    let mut rng = ids::ids(24007);
    let dir = TempDir::new().unwrap();
    let item = repo(dir.path());
    let topic = ids::cyrillic(&mut rng, 6);
    let ident = ids::uuid(&mut rng);
    let time = ids::time(&mut rng);
    let entry = session::session(&serde_json::json!({
        "id": ident,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    item.append(entry);
    let hit = item.find(&ident);
    assert_eq!(
        topic,
        hit.unwrap().topic(),
        "Found session topic did not match"
    );
}

#[test]
fn the_repository_find_returns_none_for_missing() {
    let mut rng = ids::ids(24009);
    let dir = TempDir::new().unwrap();
    let item = repo(dir.path());
    let code = ids::cyrillic(&mut rng, 6);
    let hit = item.find(&code);
    assert!(hit.is_none(), "Find returned value for missing ID");
}

#[test]
fn the_repository_update_modifies_session() {
    let mut rng = ids::ids(24011);
    let dir = TempDir::new().unwrap();
    let item = repo(dir.path());
    let topic = ids::cyrillic(&mut rng, 6);
    let ident = ids::uuid(&mut rng);
    let time = ids::time(&mut rng);
    let entry = session::session(&serde_json::json!({
        "id": ident,
        "topic": topic,
        "tasks": [],
        "created": time
    }));
    item.append(entry.clone());
    let query = ids::cyrillic(&mut rng, 6);
    let status = ids::cyrillic(&mut rng, 6);
    let language = ids::cyrillic(&mut rng, 5);
    let service = ids::cyrillic(&mut rng, 4);
    let summary = ids::cyrillic(&mut rng, 6);
    let value = result::ResearchReport::new(&summary, vec![]);
    let run = task::task(&serde_json::json!({
        "id": ids::uuid(&mut rng),
        "query": query,
        "status": status,
        "language": language,
        "service": service,
        "result": value.data(),
        "created": time
    }));
    let revision = entry.extend(run);
    item.update(revision);
    let hit = item.find(&ident);
    assert_eq!(
        1,
        hit.unwrap().tasks().len(),
        "Updated session did not contain task"
    );
}

#[test]
fn the_repository_migrates_legacy_folders() {
    let mut rng = ids::ids(24013);
    let dir = TempDir::new().unwrap();
    let day = 1 + (rng.next_u32() % 8) as u32;
    let date = format!("2026-01-0{}", day);
    let slug = ids::ascii(&mut rng, 6);
    let code = &ids::uuid(&mut rng)[..8];
    let name = format!("{}_{}_{}", date, slug, code);
    let path = dir.path().join(&name);
    fs::create_dir_all(&path).unwrap();
    let tag = ids::ascii(&mut rng, 4);
    let response = path.join(format!("response-{}.json", tag));
    fs::write(&response, "{}").unwrap();
    let r = repo(dir.path());
    r.load();
    let file = path.join("session.edn");
    assert!(file.exists(), "Migration did not create session edn");
}

#[test]
fn the_repository_builds_tasks_from_responses() {
    let mut rng = ids::ids(24015);
    let dir = TempDir::new().unwrap();
    let day = 1 + (rng.next_u32() % 8) as u32;
    let date = format!("2026-01-0{}", day);
    let slug = ids::ascii(&mut rng, 6);
    let code = &ids::uuid(&mut rng)[..8];
    let name = format!("{}_{}_{}", date, slug, code);
    let path = dir.path().join(&name);
    fs::create_dir_all(&path).unwrap();
    let alpha = ids::ascii(&mut rng, 4);
    let beta = ids::ascii(&mut rng, 4);
    let left = path.join(format!("response-{}.json", alpha));
    let right = path.join(format!("response-{}.json", beta));
    fs::write(&left, "{}").unwrap();
    fs::write(&right, "{}").unwrap();
    let r = repo(dir.path());
    let list = r.load();
    let item = &list[0];
    assert_eq!(
        2,
        item.tasks().len(),
        "Migration did not build tasks from responses"
    );
}

use rand::Rng;
