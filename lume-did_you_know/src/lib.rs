use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::Compaq_Thin));

use include_dir::{Dir, include_dir};

use serde::Deserialize;

const ROTATE_SECS: u64 = 18;
static FACTS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/data/facts");

#[derive(Clone, Debug)]
struct FactTopic {
    display_name: String,
    facts: Vec<FactEntry>,
}

#[derive(Clone, Debug)]
struct FactEntry {
    fact: String,
    note: String,
}

#[derive(Deserialize)]
struct TopicFile {
    topic: Option<String>,
    display_name: Option<String>,
    facts: Vec<TopicFact>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TopicFact {
    Text(String),
    Detailed { fact: String, note: Option<String> },
}

static TOPICS: Lazy<Vec<FactTopic>> = Lazy::new(load_topics);
static SPEC_BYTES: Lazy<Vec<u8>> =
    Lazy::new(|| text_slide::serialize_spec(&did_you_know_slide_spec()));

pub fn build_overlay(start_seed: u64, elapsed: f32) -> RuntimeOverlay<Vertex> {
    let (topic, entry) = select_fact(start_seed, elapsed);
    let mut blocks = vec![
        TextBlock {
            text: "DID YOU KNOW?",
            x: 160.0,
            y: 28.0,
            scale: 1.10,
            color: [1.0, 0.94, 0.62, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
        TextBlock {
            text: &topic.display_name,
            x: 160.0,
            y: 58.0,
            scale: 0.95,
            color: [0.92, 0.86, 0.48, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        },
        TextBlock {
            text: &entry.fact,
            x: 160.0,
            y: 88.0,
            scale: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(32),
        },
    ];

    if !entry.note.is_empty() {
        blocks.push(TextBlock {
            text: &entry.note,
            x: 160.0,
            y: 194.0,
            scale: 0.80,
            color: [0.72, 0.76, 0.82, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(34),
        });
    }

    text_slide::compose_overlay(&blocks, &FONT)
}

pub fn did_you_know_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec(
        "did_you_know_scene",
        build_overlay(text_slide::now_unix_secs(), 0.0),
        palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn palette() -> Palette {
    Palette {
        background: [0.06, 0.07, 0.10, 1.0],
        panel: [0.10, 0.12, 0.16, 0.96],
        accent: [0.80, 0.66, 0.22, 0.96],
        accent_soft: [0.28, 0.22, 0.08, 0.96],
    }
}

fn load_topics() -> Vec<FactTopic> {
    let mut topics = Vec::new();
    for file in FACTS_DIR.files() {
        let contents = file.contents_utf8().expect("facts file should be UTF-8");
        let parsed: TopicFile =
            serde_yaml::from_str(contents).expect("decode did-you-know topic file");
        let display_name = parsed.display_name.or(parsed.topic).unwrap_or_else(|| {
            file.path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("topic")
                .to_string()
        });
        let facts = parsed
            .facts
            .into_iter()
            .map(|fact| match fact {
                TopicFact::Text(fact) => FactEntry {
                    fact: text_slide::normalize_text(&fact),
                    note: String::new(),
                },
                TopicFact::Detailed { fact, note } => FactEntry {
                    fact: text_slide::normalize_text(&fact),
                    note: note
                        .map(|note| text_slide::normalize_text(&note))
                        .unwrap_or_default(),
                },
            })
            .collect();

        topics.push(FactTopic {
            display_name: text_slide::normalize_text(&display_name),
            facts,
        });
    }

    topics.sort_by(|left, right| left.display_name.cmp(&right.display_name));
    topics
}

fn select_fact(start_seed: u64, elapsed: f32) -> (&'static FactTopic, &'static FactEntry) {
    let bucket = start_seed / ROTATE_SECS + (elapsed.max(0.0) as u64) / ROTATE_SECS;
    let topic_index = (mix64(bucket ^ 0x9E37_79B9_7F4A_7C15) as usize) % TOPICS.len().max(1);
    let topic = &TOPICS[topic_index];
    let fact_index = (mix64(bucket ^ 0xC2B2_AE35_87A5_0D3B) as usize) % topic.facts.len().max(1);
    (topic, &topic.facts[fact_index])
}

fn mix64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
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
    let mut state = runtime_state::state();
    state.elapsed = 0.0;
    state.start_seed = text_slide::now_unix_secs();
    state.refresh();
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
fn slide_update(dt: f32) -> i32 {
    let mut state = runtime_state::state();
    state.elapsed += dt.max(0.0);
    state.refresh();
    1
}

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    pub struct RuntimeState {
        pub elapsed: f32,
        pub start_seed: u64,
        pub overlay_bytes: Vec<u8>,
    }

    impl RuntimeState {
        fn new() -> Self {
            let mut state = Self {
                elapsed: 0.0,
                start_seed: super::text_slide::now_unix_secs(),
                overlay_bytes: Vec::new(),
            };
            state.refresh();
            state
        }

        pub fn refresh(&mut self) {
            self.overlay_bytes = super::text_slide::serialize_overlay(&super::build_overlay(
                self.start_seed,
                self.elapsed,
            ));
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
    use super::*;

    #[test]
    fn spec_valid() {
        did_you_know_slide_spec().validate().unwrap();
    }

    #[test]
    fn facts_archive_loaded() {
        assert!(TOPICS.len() >= 10);
        assert!(TOPICS.iter().all(|topic| !topic.facts.is_empty()));
    }

    #[test]
    fn selection_changes_between_rotation_buckets() {
        let first = build_overlay(0, 0.0);
        let second = build_overlay(0, ROTATE_SECS as f32);
        assert_ne!(first.vertices, second.vertices);
    }
}
