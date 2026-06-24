//! Gemini-backed brief generator — fan-out over focused, parallel calls.
//!
//! A fresh brief is one skeleton call (intent + 3 calibrated angles) followed
//! by 3 parallel calls, one per angle, each producing that angle's sub-angles.
//! A retune fans out 3 parallel re-phrase calls. Every call hits a fast Flash
//! model and a JSON schema, so the whole 3×3 comes back quickly and each angle
//! gets the model's full attention. Reuses `GEMINI_API_KEY`.

use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use crate::brief::generator::{BriefError, BriefGenerator, BriefRequest};
use crate::brief::prompt::{ANGLE_PROMPT, RETUNE_PROMPT, SKELETON_PROMPT};
use crate::brief::{TuiBrief, TuiRoot, TuiSub};

const ENDPOINT_TEMPLATE: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={key}";
/// Fast model for the parallel fan-out. Bump when a newer Flash ships.
const MODEL: &str = "gemini-3.5-flash";

/// Synchronous Gemini brief generator.
pub struct GeminiBriefGenerator {
    key: String,
    client: reqwest::blocking::Client,
}

impl GeminiBriefGenerator {
    /// Construct from `GEMINI_API_KEY`. Returns `None` when the key is empty.
    pub fn from_env() -> Option<Self> {
        let key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        if key.is_empty() {
            return None;
        }
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .ok()?;
        Some(Self { key, client })
    }

    /// One Gemini call: system prompt + user text + JSON schema → the JSON text.
    fn call(&self, system: &str, user: &str, schema: Value) -> Result<String, BriefError> {
        let url = ENDPOINT_TEMPLATE
            .replace("{model}", MODEL)
            .replace("{key}", &self.key);
        let body = serde_json::json!({
            "contents": [ { "role": "user", "parts": [{ "text": user }] } ],
            "systemInstruction": { "role": "system", "parts": [{ "text": system }] },
            "generationConfig": {
                "responseMimeType": "application/json",
                "responseSchema": schema,
                "temperature": 0.55
            }
        });
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .send()
            .map_err(|e| BriefError::Network(format!("send: {e}")))?;
        let status = response.status();
        let raw_text = response
            .text()
            .map_err(|e| BriefError::Network(format!("body: {e}")))?;
        if !status.is_success() {
            return Err(BriefError::Network(format!(
                "gemini status {} — {}",
                status,
                truncate(&raw_text, 240)
            )));
        }
        let payload: GeminiResponse = serde_json::from_str(&raw_text)
            .map_err(|e| BriefError::Parse(format!("envelope: {e}")))?;
        payload
            .first_text()
            .map(str::to_string)
            .ok_or(BriefError::Empty)
    }

    /// Decompose a topic into intent + 3 calibrated root angles.
    fn skeleton(&self, topic: &str, language: &str) -> Result<RawSkeleton, BriefError> {
        let user = format!(
            "Topic: {topic}\n\nReturn the skeleton JSON. Write everything in the same language as the topic ({language})."
        );
        let json = self.call(SKELETON_PROMPT, &user, skeleton_schema())?;
        let skel: RawSkeleton =
            serde_json::from_str(&json).map_err(|e| BriefError::Parse(format!("skeleton: {e}")))?;
        if skel.topics.len() != 3 {
            return Err(BriefError::Parse(format!(
                "expected 3 angles, got {}",
                skel.topics.len()
            )));
        }
        Ok(skel)
    }

    /// Produce one full root angle: the skeleton angle + its 3 sub-angles.
    fn angle(
        &self,
        topic: &str,
        language: &str,
        intent: &str,
        root: &RawRoot,
    ) -> Result<TuiRoot, BriefError> {
        let user = format!(
            "Topic: {topic}\nUser intent: {intent}\n\nThis angle:\n- title: {}\n- why: {}\n- levels: depth {}, novelty {}, applied {}\n\nReturn its 3 sub-angles as JSON, in the topic's language ({language}).",
            root.title, root.note, root.depth, root.novelty, root.applied
        );
        let json = self.call(ANGLE_PROMPT, &user, subs_schema())?;
        let parsed: RawSubs =
            serde_json::from_str(&json).map_err(|e| BriefError::Parse(format!("subs: {e}")))?;
        let subs = into_three_subs(parsed.subs)?;
        Ok(TuiRoot {
            title: root.title.clone(),
            note: root.note.clone(),
            depth: clamp(root.depth),
            novelty: clamp(root.novelty),
            applied: clamp(root.applied),
            subs,
        })
    }

    /// Re-phrase one angle to its (already edited) knob levels.
    fn rephrase(
        &self,
        topic: &str,
        language: &str,
        prior: &TuiRoot,
    ) -> Result<TuiRoot, BriefError> {
        let user = format!(
            "Topic: {topic}\n\nAngle to re-phrase, with its NEW levels:\n- title: {}\n- why: {}\n- new levels: depth {}, novelty {}, applied {}\n- current sub-angles:\n  1. {}\n  2. {}\n  3. {}\n\nRe-phrase title, note and subs to match the new levels. JSON, in the topic's language ({language}).",
            prior.title,
            prior.note,
            prior.depth,
            prior.novelty,
            prior.applied,
            prior.subs[0].title,
            prior.subs[1].title,
            prior.subs[2].title,
        );
        let json = self.call(RETUNE_PROMPT, &user, rephrase_schema())?;
        let parsed: RawRephrase =
            serde_json::from_str(&json).map_err(|e| BriefError::Parse(format!("rephrase: {e}")))?;
        let subs = into_three_subs(parsed.subs)?;
        Ok(TuiRoot {
            title: parsed.title,
            note: parsed.note,
            depth: prior.depth,
            novelty: prior.novelty,
            applied: prior.applied,
            subs,
        })
    }
}

impl BriefGenerator for GeminiBriefGenerator {
    fn generate(&self, req: &BriefRequest) -> Result<TuiBrief, BriefError> {
        if self.key.is_empty() {
            return Err(BriefError::NoApiKey);
        }
        let topic = req.topic.as_str();
        let language = req.language.as_str();
        match &req.prior {
            None => {
                let skel = self.skeleton(topic, language)?;
                let roots =
                    fan_out(|i| self.angle(topic, language, &skel.intent, &skel.topics[i]))?;
                Ok(TuiBrief {
                    topic: topic.to_string(),
                    language: language.to_string(),
                    intent: skel.intent,
                    topics: roots,
                })
            }
            Some(prior) => {
                let roots = fan_out(|i| self.rephrase(topic, language, &prior.topics[i]))?;
                Ok(TuiBrief {
                    topic: topic.to_string(),
                    language: language.to_string(),
                    intent: prior.intent.clone(),
                    topics: roots,
                })
            }
        }
    }
}

/// Run `make(0..3)` on three threads at once and collect into `[TuiRoot; 3]`.
fn fan_out<F>(make: F) -> Result<[TuiRoot; 3], BriefError>
where
    F: Fn(usize) -> Result<TuiRoot, BriefError> + Sync,
{
    let make = &make;
    let results: Vec<Result<TuiRoot, BriefError>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..3).map(|i| s.spawn(move || make(i))).collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err(BriefError::Network("angle worker panicked".into())))
            })
            .collect()
    });
    let mut roots = Vec::with_capacity(3);
    for r in results {
        roots.push(r?);
    }
    roots
        .try_into()
        .map_err(|_| BriefError::Parse("expected 3 angles".into()))
}

fn into_three_subs(subs: Vec<RawSub>) -> Result<[TuiSub; 3], BriefError> {
    if subs.len() != 3 {
        return Err(BriefError::Parse(format!(
            "expected 3 sub-angles, got {}",
            subs.len()
        )));
    }
    let mut out = subs.into_iter().map(|s| TuiSub {
        title: s.title,
        depth: clamp(s.depth),
        novelty: clamp(s.novelty),
        applied: clamp(s.applied),
    });
    Ok([
        out.next().unwrap(),
        out.next().unwrap(),
        out.next().unwrap(),
    ])
}

fn skeleton_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "intent": { "type": "string" },
            "topics": {
                "type": "array", "minItems": 3, "maxItems": 3,
                "items": {
                    "type": "object",
                    "properties": {
                        "title":   { "type": "string" },
                        "note":    { "type": "string" },
                        "depth":   { "type": "integer", "minimum": 1, "maximum": 5 },
                        "novelty": { "type": "integer", "minimum": 1, "maximum": 5 },
                        "applied": { "type": "integer", "minimum": 1, "maximum": 5 }
                    },
                    "required": ["title", "note", "depth", "novelty", "applied"]
                }
            }
        },
        "required": ["intent", "topics"]
    })
}

fn subs_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": { "subs": sub_array() },
        "required": ["subs"]
    })
}

fn rephrase_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "note":  { "type": "string" },
            "subs":  sub_array()
        },
        "required": ["title", "note", "subs"]
    })
}

fn sub_array() -> Value {
    serde_json::json!({
        "type": "array", "minItems": 3, "maxItems": 3,
        "items": {
            "type": "object",
            "properties": {
                "title":   { "type": "string" },
                "depth":   { "type": "integer", "minimum": 1, "maximum": 5 },
                "novelty": { "type": "integer", "minimum": 1, "maximum": 5 },
                "applied": { "type": "integer", "minimum": 1, "maximum": 5 }
            },
            "required": ["title", "depth", "novelty", "applied"]
        }
    })
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize)]
struct Part {
    text: Option<String>,
}

impl GeminiResponse {
    fn first_text(&self) -> Option<&str> {
        self.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| p.text.as_deref())
    }
}

#[derive(Deserialize)]
struct RawSkeleton {
    intent: String,
    topics: Vec<RawRoot>,
}

#[derive(Deserialize)]
struct RawRoot {
    title: String,
    note: String,
    depth: u8,
    novelty: u8,
    applied: u8,
}

#[derive(Deserialize)]
struct RawSubs {
    subs: Vec<RawSub>,
}

#[derive(Deserialize)]
struct RawRephrase {
    title: String,
    note: String,
    subs: Vec<RawSub>,
}

#[derive(Deserialize)]
struct RawSub {
    title: String,
    depth: u8,
    novelty: u8,
    applied: u8,
}

fn clamp(v: u8) -> u8 {
    v.clamp(1, 5)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push('\u{2026}');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn the_real_gemini_call_returns_a_three_by_three_brief() {
        let Some(g) = GeminiBriefGenerator::from_env() else {
            eprintln!("skipping: GEMINI_API_KEY not set");
            return;
        };
        let brief = g
            .generate(&BriefRequest::fresh(
                "Основы многопоточности в Rust",
                "Russian",
            ))
            .expect("brief");
        assert_eq!(3, brief.topics.len(), "missing root topics");
        for root in &brief.topics {
            assert_eq!(3, root.subs.len(), "missing sub topics");
            assert!((1..=5).contains(&root.depth), "depth out of range");
        }
    }
}
