use std::path::{Path, PathBuf};

/// Object that can generate images.
pub trait Generated {
    /// Generate image at path.
    fn generate(&self, topic: &str, path: &Path) -> Result<PathBuf, String>;
}

/// Return prompt with topic inserted.
pub fn prompt(text: &str, topic: &str) -> String {
    text.replace("%s", topic)
}

/// Extract image bytes from response.
pub fn image(data: &serde_json::Value) -> Vec<u8> {
    let items = data
        .get("candidates")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first());
    let parts = items
        .and_then(|i| i.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .and_then(|a| a.first());
    let inline = parts.and_then(|p| p.get("inlineData").or_else(|| p.get("inline_data")));
    let value = inline
        .and_then(|i| i.get("data"))
        .and_then(|d| d.as_str())
        .unwrap_or("");
    if value.is_empty() {
        Vec::new()
    } else {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(value)
            .unwrap_or_default()
    }
}

/// Compress image bytes into jpeg.
pub fn compress(data: &[u8], path: &Path, quality: u8) -> Result<PathBuf, String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Directory creation failed: {}", e))?;
    }
    let reader = image::ImageReader::new(std::io::Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| format!("Image format detection failed: {}", e))?;
    let img = reader
        .decode()
        .map_err(|e| format!("Image decode failed: {}", e))?;
    let file = std::fs::File::create(path).map_err(|e| format!("File creation failed: {}", e))?;
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
    img.write_with_encoder(encoder)
        .map_err(|e| format!("JPEG write failed: {}", e))?;
    Ok(path.to_path_buf())
}

/// Parse EDN text to JSON.
pub fn parse(text: &str) -> Result<serde_json::Value, String> {
    let edn: edn_rs::Edn = text
        .parse()
        .map_err(|e| format!("EDN parse failed: {:?}", e))?;
    let json = edn.to_json();
    serde_json::from_str(&json).map_err(|e| format!("JSON parse failed: {:?}", e))
}

/// Merge two JSON objects recursively.
fn merge(base: &mut serde_json::Value, over: &serde_json::Value) {
    match (base, over) {
        (serde_json::Value::Object(b), serde_json::Value::Object(o)) => {
            for (k, v) in o {
                merge(b.entry(k.clone()).or_insert(serde_json::Value::Null), v);
            }
        }
        (base, over) => {
            *base = over.clone();
        }
    }
}

/// Return resources path.
fn root() -> PathBuf {
    if let Ok(dir) = std::env::var("RESOURCES_DIR") {
        return PathBuf::from(dir);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources")
}

/// Read and parse resource EDN file.
fn resource(name: &str) -> Result<serde_json::Value, String> {
    let path = root().join(name);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Resource read failed path={}: {}", path.display(), e))?;
    parse(&text)
}

/// Image generator.
pub struct Generator {
    key: String,
    spec: String,
    model: String,
    quality: u8,
}

impl Generator {
    /// Create generator from key, spec, model and quality.
    pub fn new(key: &str, spec: &str, model: &str, quality: u8) -> Self {
        Self {
            key: key.to_string(),
            spec: spec.to_string(),
            model: model.to_string(),
            quality,
        }
    }
}

impl Generated for Generator {
    fn generate(&self, topic: &str, path: &Path) -> Result<PathBuf, String> {
        let text = prompt(&self.spec, topic);
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.key
        );
        let body = serde_json::json!({
            "contents": [{"parts": [{"text": text}]}],
            "generationConfig": {
                "responseModalities": ["IMAGE"],
                "imageConfig": {
                    "aspectRatio": "16:9",
                    "imageSize": "1K"
                }
            }
        });
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .timeout(std::time::Duration::from_secs(600))
            .send()
            .map_err(|e| format!("Gemini request failed: {}", e))?;
        let status = response.status().as_u16();
        if status >= 300 {
            return Err(format!(
                "Gemini image failed model={} status={}",
                self.model, status
            ));
        }
        let raw: serde_json::Value = response
            .json()
            .map_err(|e| format!("Gemini response parse failed: {}", e))?;
        let value = image(&raw);
        if value.is_empty() {
            return Err("Gemini image missing".to_string());
        }
        compress(&value, path, self.quality)
    }
}

/// Create generator from env.
pub fn generator() -> Result<Generator, String> {
    let key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
    if key.is_empty() {
        return Err("GEMINI_API_KEY is required".to_string());
    }
    let data = resource("cover/parts.edn")?;
    let topic_path = data
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or("Cover parts missing :topic")?;
    let entry = resource(topic_path)?;
    let value = entry
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or("Cover topic missing")?;
    if value.trim().is_empty() {
        return Err("Cover topic missing".to_string());
    }
    let items = data
        .get("image")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut combined = serde_json::Value::Object(serde_json::Map::new());
    for item in &items {
        let name = item.as_str().ok_or("Cover part path is not a string")?;
        let part = resource(name)?;
        merge(&mut combined, &part);
    }
    let spec = serde_json::json!({
        "topic": value,
        "marketing_image": combined
    });
    let body =
        serde_json::to_string(&spec).map_err(|e| format!("Spec serialization failed: {}", e))?;
    Ok(Generator::new(
        &key,
        &body,
        "gemini-3-pro-image-preview",
        85,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_generator_replaces_topic() {
        let mut rng = ids::ids(6093);
        let text = ids::cyrillic(&mut rng, 7);
        let spec = "prefix %s suffix";
        let result = prompt(spec, &text);
        assert!(
            result.contains(&text),
            "Prompt replacement did not insert topic"
        );
    }

    #[test]
    fn the_generator_extracts_image_bytes() {
        let mut rng = ids::ids(9041);
        let mark = ids::ascii(&mut rng, 12);
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(mark.as_bytes());
        let data = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "inlineData": {
                            "data": encoded
                        }
                    }]
                }
            }]
        });
        let result = image(&data);
        assert_eq!(
            mark.as_bytes(),
            result.as_slice(),
            "Image bytes were not extracted"
        );
    }

    #[test]
    fn the_generator_returns_empty_for_missing_data() {
        let mut rng = ids::ids(9043);
        let _mark = ids::cyrillic(&mut rng, 4);
        let data = serde_json::json!({"candidates": []});
        let result = image(&data);
        assert!(
            result.is_empty(),
            "Image returned bytes for empty candidates"
        );
    }

    #[test]
    fn the_generator_includes_wabi_sabi_principles() {
        let mut rng = ids::ids(9011);
        let data = resource("cover/parts.edn").unwrap();
        let parts = data.get("image").and_then(|v| v.as_array()).unwrap();
        let item = "cover/wabi_sabi.edn";
        let entry = resource(item).unwrap();
        let node = entry.get("wabi_sabi").unwrap();
        let items = node.get("principles").and_then(|v| v.as_array()).unwrap();
        let picks = [
            "\u{7c21}\u{7d20}",
            "\u{4e0d}\u{5747}\u{6b63}",
            "\u{6e0b}\u{5473}",
            "\u{81ea}\u{7136}",
            "\u{5e7d}\u{7384}",
            "\u{8131}\u{4fd7}",
            "\u{6e05}\u{5bc2}",
        ];
        let index = ids::digit(&mut rng, picks.len() as u32) as usize;
        let pick = picks[index % picks.len()];
        let names: Vec<&str> = items
            .iter()
            .filter_map(|i| i.get("name_jp").and_then(|v| v.as_str()))
            .collect();
        let ratio = node
            .get("rules")
            .and_then(|r| r.get("dominant_ratio"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tones = node
            .get("rules")
            .and_then(|r| r.get("dominant_tones"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let has_item = parts.iter().any(|p| p.as_str() == Some(item));
        let has_seven = items.len() == 7;
        let has_pick = names.contains(&pick);
        let has_ratio = ratio.contains("70_80");
        let has_tones = tones.contains("muted");
        let result = has_item && has_seven && has_pick && has_ratio && has_tones;
        assert!(result, "Wabi-sabi principles were missing");
    }

    #[test]
    fn the_generator_disallows_frames() {
        let mut rng = ids::ids(9017);
        let _mark = ids::cyrillic(&mut rng, 4);
        let base = resource("cover/quality_requirements.edn").unwrap();
        let value = base
            .get("quality_requirements")
            .and_then(|q| q.get("image_integrity"))
            .and_then(|i| i.get("borders"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(
            "full_bleed_edge_to_edge_artwork", value,
            "Frames were permitted"
        );
    }

    #[test]
    fn the_generator_requires_edge_to_edge() {
        let mut rng = ids::ids(9021);
        let _mark = ids::cyrillic(&mut rng, 4);
        let comp = resource("cover/composition_guidelines.edn").unwrap();
        let qual = resource("cover/quality_requirements.edn").unwrap();
        let edge = comp
            .get("composition_guidelines")
            .and_then(|c| c.get("edge_to_edge"))
            .and_then(|e| e.get("rule"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let fill = qual
            .get("quality_requirements")
            .and_then(|q| q.get("image_integrity"))
            .and_then(|i| i.get("edge_fill"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let result = edge == "full_bleed_composition_with_cropping"
            && fill == "edges_filled_with_scene_texture";
        assert!(result, "Edge to edge rules were missing");
    }

    #[test]
    fn the_generator_disallows_text() {
        let mut rng = ids::ids(9023);
        let _mark = ids::cyrillic(&mut rng, 4);
        let qual = resource("cover/quality_requirements.edn").unwrap();
        let surf = resource("cover/surface_treatment.edn").unwrap();
        let text = qual
            .get("quality_requirements")
            .and_then(|q| q.get("image_integrity"))
            .and_then(|i| i.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let rule = surf
            .get("surface_treatment")
            .and_then(|s| s.get("all_readable_surfaces"))
            .and_then(|a| a.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let result = text == "no_text_no_letters_no_symbols_no_numbers_no_logos"
            && rule == "no_text_or_glyphs_only_texture";
        assert!(result, "Text restrictions were missing");
    }
}
