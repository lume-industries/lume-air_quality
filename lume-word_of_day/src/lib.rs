use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::Apricot_Mono));

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WordPayload {
    pub word: String,
    pub part: String,
    pub definition: String,
    pub etymology: String,
    pub updated: String,
}

#[derive(Deserialize)]
struct DictionaryEntry {
    #[serde(default)]
    origin: Option<String>,
    #[serde(default)]
    etymology: Option<String>,
    #[serde(default)]
    meanings: Vec<Meaning>,
}

#[derive(Deserialize)]
struct Meaning {
    #[serde(rename = "partOfSpeech", default)]
    part_of_speech: String,
    #[serde(default)]
    etymology: Option<String>,
    #[serde(default)]
    definitions: Vec<Definition>,
}

#[derive(Deserialize)]
struct Definition {
    #[serde(default)]
    definition: String,
}

static SPEC_BYTES: Lazy<Vec<u8>> =
    Lazy::new(|| text_slide::serialize_spec(&word_of_day_slide_spec()));

pub fn parse_rss_item(body: &str) -> Result<(String, String), String> {
    let item = extract_section(body, "item").ok_or_else(|| "No <item> in RSS feed".to_string())?;
    let title = extract_tag(item, "title").ok_or_else(|| "RSS item missing title".to_string())?;
    let description = extract_tag(item, "description")
        .ok_or_else(|| "RSS item missing description".to_string())?;

    let title = text_slide::normalize_text(&strip_tags(&decode_entities(&title)));
    let word = title
        .strip_prefix("Word of the Day:")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(title.as_str())
        .to_string();
    let raw_desc = collapse_whitespace(&strip_tags(&decode_entities(&description)));
    Ok((word, raw_desc))
}

pub fn parse_dictionary_payload(
    word: &str,
    raw_desc: &str,
    body: &str,
    now_secs: u64,
) -> Result<WordPayload, String> {
    let entries: Vec<DictionaryEntry> =
        serde_json::from_str(body).map_err(|error| format!("invalid dictionary JSON: {error}"))?;
    let entry = entries
        .into_iter()
        .next()
        .ok_or_else(|| "dictionary API returned no entries".to_string())?;
    let meaning = entry.meanings.into_iter().next();
    let part = meaning
        .as_ref()
        .map(|meaning| meaning.part_of_speech.as_str())
        .unwrap_or("");
    let definition = meaning
        .as_ref()
        .and_then(|meaning| meaning.definitions.first())
        .map(|definition| definition.definition.as_str())
        .filter(|definition| !definition.is_empty())
        .unwrap_or(raw_desc);
    let etymology = entry
        .origin
        .or(entry.etymology)
        .or_else(|| {
            meaning
                .as_ref()
                .and_then(|meaning| meaning.etymology.clone())
        })
        .unwrap_or_default();

    Ok(WordPayload {
        word: text_slide::normalize_text(word),
        part: truncate(&text_slide::normalize_text(part), 20),
        definition: truncate(&text_slide::normalize_text(definition), 100),
        etymology: truncate(&text_slide::normalize_text(&etymology), 120),
        updated: format!("Updated {}", text_slide::date_utils::utc_hhmm_from_unix(now_secs)),
    })
}

pub fn fallback_payload(word: &str, raw_desc: &str, now_secs: u64) -> WordPayload {
    WordPayload {
        word: text_slide::normalize_text(word),
        part: String::new(),
        definition: truncate(&text_slide::normalize_text(raw_desc), 100),
        etymology: String::new(),
        updated: format!("Updated {}", text_slide::date_utils::utc_hhmm_from_unix(now_secs)),
    }
}

pub fn word_of_day_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec("word_of_day_scene", build_overlay(None), palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn build_overlay(payload: Option<&WordPayload>) -> RuntimeOverlay<Vertex> {
    if let Some(payload) = payload {
        let part_line = if payload.part.is_empty() {
            "dictionary headline".to_string()
        } else {
            payload.part.to_uppercase()
        };
        let etymology_line = if payload.etymology.is_empty() {
            "No etymology available from the current source.".to_string()
        } else {
            payload.etymology.clone()
        };
        return text_slide::compose_overlay(&[
            title_block("WORD OF THE DAY"),
            TextBlock {
                text: &payload.word,
                x: 160.0,
                y: 54.0,
                scale: 1.32,
                color: [1.0, 0.92, 0.62, 1.0],
                align: TextAlign::Center,
                wrap_cols: Some(18),
            },
            TextBlock {
                text: &part_line,
                x: 160.0,
                y: 78.0,
                scale: 0.76,
                color: [0.72, 0.82, 0.92, 1.0],
                align: TextAlign::Center,
                wrap_cols: Some(20),
            },
            TextBlock {
                text: &payload.definition,
                x: 40.0,
                y: 104.0,
                scale: 0.88,
                color: [1.0, 1.0, 1.0, 1.0],
                align: TextAlign::Left,
                wrap_cols: Some(34),
            },
            TextBlock {
                text: &etymology_line,
                x: 40.0,
                y: 162.0,
                scale: 0.72,
                color: [0.78, 0.84, 0.90, 1.0],
                align: TextAlign::Left,
                wrap_cols: Some(36),
            },
            footer_block(&payload.updated),
        ], &FONT);
    }

    text_slide::compose_overlay(&[
        title_block("WORD OF THE DAY"),
        TextBlock {
            text: "Loading Merriam-Webster feed...",
            x: 160.0,
            y: 112.0,
            scale: 0.96,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        },
    ], &FONT)
}

fn title_block(text: &'static str) -> TextBlock<'static> {
    TextBlock {
        text,
        x: 160.0,
        y: 26.0,
        scale: 1.05,
        color: [0.90, 0.76, 0.34, 1.0],
        align: TextAlign::Center,
        wrap_cols: None,
    }
}

fn footer_block<'a>(text: &'a str) -> TextBlock<'a> {
    TextBlock {
        text,
        x: 160.0,
        y: 198.0,
        scale: 0.78,
        color: [0.72, 0.82, 0.92, 1.0],
        align: TextAlign::Center,
        wrap_cols: None,
    }
}

fn palette() -> Palette {
    Palette {
        background: [0.08, 0.04, 0.02, 1.0],
        panel: [0.16, 0.09, 0.05, 0.96],
        accent: [0.44, 0.24, 0.10, 0.96],
        accent_soft: [0.26, 0.14, 0.08, 0.96],
    }
}

fn extract_section<'a>(body: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}");
    let start = body.find(&open)?;
    let start = body[start..].find('>')? + start + 1;
    let close = format!("</{tag}>");
    let end = body[start..].find(&close)? + start;
    Some(&body[start..end])
}

fn extract_tag(body: &str, tag: &str) -> Option<String> {
    extract_section(body, tag).map(ToOwned::to_owned)
}

fn decode_entities(input: &str) -> String {
    input
        .replace("<![CDATA[", "")
        .replace("]]>", "")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn strip_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut inside_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

fn collapse_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn truncate(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    let mut truncated = String::new();
    for ch in text.chars().take(max_len.saturating_sub(3)) {
        truncated.push(ch);
    }
    truncated.push_str("...");
    truncated
}

#[cfg(target_arch = "wasm32")]
vzglyd_text_slide::VRX_64_slide::export_traced_entrypoints! {
    init = slide_init,
    update = slide_update,
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_spec_ptr() -> *const u8 {
    SPEC_BYTES.as_ptr()
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_spec_len() -> u32 {
    SPEC_BYTES.len() as u32
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_abi_version() -> u32 {
    vzglyd_text_slide::VRX_64_slide::ABI_VERSION
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
fn slide_init() -> i32 {
    runtime_state::state().refresh();
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
fn slide_update(_dt: f32) -> i32 {
    let mut state = runtime_state::state();
    if let Some(payload) = text_slide::channel_runtime::poll_json::<WordPayload>(&mut state.response_buf) {
        state.payload = Some(payload);
    }
    state.refresh();
    1
}

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::{WordPayload, build_overlay};
    use text_slide::channel_runtime;
    use crate::text_slide;

    pub struct RuntimeState {
        pub payload: Option<WordPayload>,
        pub overlay_bytes: Vec<u8>,
        pub response_buf: Vec<u8>,
    }

    impl RuntimeState {
        fn new() -> Self {
            let mut state = Self {
                payload: None,
                overlay_bytes: Vec::new(),
                response_buf: vec![0u8; text_slide::channel_runtime::CHANNEL_BUF_BYTES],
            };
            state.refresh();
            state
        }

        pub fn refresh(&mut self) {
            self.overlay_bytes =
                text_slide::serialize_overlay(&build_overlay(self.payload.as_ref()));
        }
    }

    static STATE: OnceLock<Mutex<RuntimeState>> = OnceLock::new();

    pub fn state() -> MutexGuard<'static, RuntimeState> {
        STATE
            .get_or_init(|| Mutex::new(RuntimeState::new()))
            .lock()
            .unwrap()
    }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_overlay_ptr() -> *const u8 {
    runtime_state::state().overlay_bytes.as_ptr()
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_overlay_len() -> u32 {
    runtime_state::state().overlay_bytes.len() as u32
}

#[cfg(test)]
mod tests {
    use *;

    #[test]
    fn spec_valid() {
        word_of_day_slide_spec().validate().unwrap();
    }

    #[test]
    fn rss_item_extracts_word_and_plain_description() {
        let body = r#"<?xml version="1.0"?>
            <rss><channel><item>
                <title>Word of the Day: serendipity</title>
                <description>&lt;b&gt;fortunate&lt;/b&gt; accident</description>
            </item></channel></rss>"#;
        let (word, desc) = parse_rss_item(body).expect("parse rss");
        assert_eq!(word, "serendipity");
        assert_eq!(desc, "fortunate accident");
    }

    #[test]
    fn dictionary_payload_uses_origin_and_truncates() {
        let body = r#"[
            {
                "origin": "From Greek ephemeros, lasting a day.",
                "meanings": [{
                    "partOfSpeech": "adjective",
                    "definitions": [{"definition": "Lasting for a very short time."}]
                }]
            }
        ]"#;
        let payload = parse_dictionary_payload("ephemeral", "fallback", body, 0)
            .expect("parse dictionary payload");
        assert_eq!(payload.part, "adjective");
        assert!(payload.definition.contains("short time"));
        assert!(payload.etymology.contains("Greek"));
    }
}
