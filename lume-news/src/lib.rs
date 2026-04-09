use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::EGA_8x8));

use serde::{Deserialize, Serialize};

const MAX_ROWS_PER_VIEW: usize = 4;
#[allow(dead_code)]
const VIEW_ROTATION_SECS: u64 = 10;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Headline {
    pub title: String,
    pub source: String,
    pub category: String,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewsPayload {
    pub updated: String,
    pub headlines: Vec<Headline>,
}

static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| text_slide::serialize_spec(&news_slide_spec()));

pub fn news_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec("news_scene", build_overlay(None, 0), palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

pub fn truncate_headline(title: &str, max_chars: usize) -> String {
    let normalized = text_slide::normalize_text(title);
    let count = normalized.chars().count();
    if count <= max_chars {
        return normalized;
    }

    let cutoff = max_chars.saturating_sub(3);
    let mut char_indices = normalized.char_indices().collect::<Vec<_>>();
    char_indices.push((normalized.len(), '\0'));

    let cutoff_idx = char_indices
        .get(cutoff)
        .map(|(idx, _)| *idx)
        .unwrap_or(normalized.len());
    let candidate = &normalized[..cutoff_idx];
    let trimmed = candidate.trim_end();
    let boundary = trimmed.rfind(' ').filter(|idx| *idx >= cutoff / 2);
    let prefix = boundary
        .map(|idx| trimmed[..idx].trim_end())
        .filter(|prefix| !prefix.is_empty())
        .unwrap_or(trimmed);

    let mut result = prefix.to_string();
    result.push_str("...");
    result
}

pub fn updated_label(epoch_secs: u64) -> String {
    format!("Updated {}", hhmm_from_unix(epoch_secs))
}

fn hhmm_from_unix(epoch_secs: u64) -> String {
    let seconds_today = epoch_secs % 86_400;
    let hours = seconds_today / 3_600;
    let minutes = (seconds_today / 60) % 60;
    format!("{hours:02}:{minutes:02}")
}

fn build_overlay(payload: Option<&NewsPayload>, view_index: usize) -> RuntimeOverlay<Vertex> {
    let (category, title, subtitle) = match view_index % 3 {
        0 => ("tech", "TECH NEWS", "hacker news + rss"),
        1 => ("world", "WORLD NEWS", "reddit world feed"),
        _ => ("australia", "AUSTRALIA", "reddit australia feed"),
    };

    if let Some(payload) = payload {
        return build_category_view(payload, category, title, subtitle);
    }

    text_slide::compose_overlay(&[
        title_block("NEWS WIRE"),
        TextBlock {
            text: "Loading aggregated feeds...",
            x: 160.0,
            y: 112.0,
            scale: 0.96,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        },
    ], &FONT)
}

fn build_category_view(
    payload: &NewsPayload,
    category: &str,
    title: &str,
    subtitle: &str,
) -> RuntimeOverlay<Vertex> {
    let headlines = headlines_for_category(payload, category);
    let mut blocks = vec![
        title_block(title),
        TextBlock {
            text: subtitle,
            x: 160.0,
            y: 46.0,
            scale: 0.78,
            color: [0.72, 0.82, 0.92, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
    ];

    if headlines.is_empty() {
        blocks.push(TextBlock {
            text: "No headlines available from the current sources.",
            x: 160.0,
            y: 112.0,
            scale: 0.88,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(28),
        });
        blocks.push(footer_block(&payload.updated));
        return text_slide::compose_overlay(&blocks, &FONT);
    }

    let now_secs = text_slide::now_unix_secs();
    let rows = headlines
        .into_iter()
        .take(MAX_ROWS_PER_VIEW)
        .map(|headline| {
            (
                format!(
                    "{}  {}",
                    headline.source.to_uppercase(),
                    relative_time_label(now_secs, headline.timestamp)
                ),
                headline.title.clone(),
                source_color(&headline.source),
            )
        })
        .collect::<Vec<_>>();

    for (idx, (meta, title, color)) in rows.iter().enumerate() {
        let y = 64.0 + idx as f32 * 34.0;
        blocks.push(TextBlock {
            text: meta,
            x: 34.0,
            y,
            scale: 0.68,
            color: *color,
            align: TextAlign::Left,
            wrap_cols: None,
        });
        blocks.push(TextBlock {
            text: title,
            x: 34.0,
            y: y + 10.0,
            scale: 0.80,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Left,
            wrap_cols: Some(34),
        });
    }

    blocks.push(footer_block(&payload.updated));
    text_slide::compose_overlay(&blocks, &FONT)
}

fn headlines_for_category<'a>(payload: &'a NewsPayload, category: &str) -> Vec<&'a Headline> {
    let mut headlines = payload
        .headlines
        .iter()
        .filter(|headline| headline.category == category)
        .collect::<Vec<_>>();
    headlines.sort_by(|left, right| {
        right
            .timestamp
            .cmp(&left.timestamp)
            .then_with(|| left.source.cmp(&right.source))
    });
    headlines
}

fn relative_time_label(now_secs: u64, timestamp: i64) -> String {
    let age = (now_secs as i64).saturating_sub(timestamp).max(0);
    if age < 3_600 {
        return format!("{}m ago", (age / 60).max(1));
    }
    if age < 86_400 {
        return format!("{}h ago", age / 3_600);
    }
    format!("{}d ago", age / 86_400)
}

fn title_block<'a>(text: &'a str) -> TextBlock<'a> {
    TextBlock {
        text,
        x: 160.0,
        y: 26.0,
        scale: 1.08,
        color: [0.94, 0.84, 0.44, 1.0],
        align: TextAlign::Center,
        wrap_cols: None,
    }
}

fn footer_block<'a>(text: &'a str) -> TextBlock<'a> {
    TextBlock {
        text,
        x: 160.0,
        y: 206.0,
        scale: 0.72,
        color: [0.72, 0.82, 0.92, 1.0],
        align: TextAlign::Center,
        wrap_cols: None,
    }
}

fn source_color(source: &str) -> [f32; 4] {
    match source {
        "HackerNews" => [1.0, 0.72, 0.34, 1.0],
        "lobste.rs" => [0.52, 0.88, 1.0, 1.0],
        "Ars Technica" => [0.68, 1.0, 0.72, 1.0],
        "r/worldnews" => [1.0, 0.64, 0.44, 1.0],
        "r/australia" => [1.0, 0.92, 0.54, 1.0],
        _ => [0.82, 0.88, 0.96, 1.0],
    }
}

fn palette() -> Palette {
    Palette {
        background: [0.03, 0.03, 0.07, 1.0],
        panel: [0.08, 0.09, 0.16, 0.96],
        accent: [0.16, 0.28, 0.44, 0.96],
        accent_soft: [0.08, 0.16, 0.26, 0.96],
    }
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
    vzglyd_text_slide::channel_runtime::info_log("vzglyd_init");
    runtime_state::state().refresh();
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
fn slide_update(_dt: f32) -> i32 {
    let mut state = runtime_state::state();
    if let Some(len) = vzglyd_text_slide::channel_runtime::poll_bytes(&mut state.response_buf) {
        match serde_json::from_slice::<NewsPayload>(&state.response_buf[..len]) {
            Ok(payload) => {
                vzglyd_text_slide::channel_runtime::info_log(&format!(
                    "received payload bytes={} headlines={}",
                    len,
                    payload.headlines.len()
                ));
                state.payload = Some(payload);
            }
            Err(error) => {
                vzglyd_text_slide::channel_runtime::info_log(&format!(
                    "failed to decode payload bytes={} error={error}",
                    len
                ));
            }
        }
    }
    state.refresh();
    1
}

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::{NewsPayload, VIEW_ROTATION_SECS, build_overlay};
    use vzglyd_text_slide::channel_runtime;
    use vzglyd_text_slide::text_slide;

    pub struct RuntimeState {
        pub payload: Option<NewsPayload>,
        pub overlay_bytes: Vec<u8>,
        pub response_buf: Vec<u8>,
    }

    impl RuntimeState {
        fn new() -> Self {
            let mut state = Self {
                payload: None,
                overlay_bytes: Vec::new(),
                response_buf: vec![0u8; vzglyd_text_slide::channel_runtime::CHANNEL_BUF_BYTES],
            };
            state.refresh();
            state
        }

        pub fn refresh(&mut self) {
            let view = ((text_slide::now_unix_secs() / VIEW_ROTATION_SECS) % 3) as usize;
            self.overlay_bytes =
                text_slide::serialize_overlay(&build_overlay(self.payload.as_ref(), view));
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
        news_slide_spec().validate().unwrap();
    }

    #[test]
    fn truncate_headline_respects_word_boundaries() {
        let truncated = truncate_headline("Rust news headline with several words", 20);
        assert_eq!(truncated, "Rust news...");
    }

    #[test]
    fn category_view_sorts_by_recency() {
        let payload = NewsPayload {
            updated: "Updated 12:00".to_string(),
            headlines: vec![
                Headline {
                    title: "Older".to_string(),
                    source: "HackerNews".to_string(),
                    category: "tech".to_string(),
                    timestamp: 10,
                },
                Headline {
                    title: "Newer".to_string(),
                    source: "lobste.rs".to_string(),
                    category: "tech".to_string(),
                    timestamp: 20,
                },
            ],
        };

        let headlines = headlines_for_category(&payload, "tech");
        assert_eq!(headlines[0].title, "Newer");
        assert_eq!(headlines[1].title, "Older");
    }

    #[test]
    fn payload_roundtrip_builds_overlay() {
        let payload = NewsPayload {
            updated: updated_label(0),
            headlines: vec![Headline {
                title: "Headline".to_string(),
                source: "HackerNews".to_string(),
                category: "tech".to_string(),
                timestamp: 0,
            }],
        };

        let bytes = serde_json::to_vec(&payload).unwrap();
        let decoded: NewsPayload = serde_json::from_slice(&bytes).unwrap();
        let overlay = build_overlay(Some(&decoded), 0);
        assert!(!overlay.vertices.is_empty());
        assert!(!overlay.indices.is_empty());
    }
}
