use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::EGA_8x8));

use serde::Deserialize;

const ROTATE_SECS: u64 = 12;

#[derive(Clone, Debug, Deserialize)]
struct Quote {
    text: String,
    author: String,
}

#[derive(Deserialize)]
struct PluginFile {
    slides: Vec<PluginSlide>,
}

#[derive(Deserialize)]
struct PluginSlide {
    data: PluginData,
}

#[derive(Deserialize)]
struct PluginData {
    inline: Vec<Quote>,
}

static QUOTES: Lazy<Vec<Quote>> = Lazy::new(load_quotes);
static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| text_slide::serialize_spec(&quotes_slide_spec()));

pub fn build_overlay(start_seed: u64, elapsed: f32) -> RuntimeOverlay<Vertex> {
    let quote = current_quote(start_seed, elapsed);
    let author_line = format!("- {}", quote.author);
    text_slide::compose_overlay(&[
        TextBlock {
            text: "QUOTE",
            x: 160.0,
            y: 34.0,
            scale: 1.20,
            color: [0.80, 0.94, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
        TextBlock {
            text: &quote.text,
            x: 160.0,
            y: 74.0,
            scale: 1.15,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(28),
        },
        TextBlock {
            text: &author_line,
            x: 160.0,
            y: 190.0,
            scale: 0.95,
            color: [0.74, 0.92, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
    ], &FONT)
}

pub fn quotes_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec(
        "quotes_scene",
        build_overlay(text_slide::now_unix_secs(), 0.0),
        palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn palette() -> Palette {
    Palette {
        background: [0.03, 0.03, 0.08, 1.0],
        panel: [0.08, 0.08, 0.16, 0.96],
        accent: [0.16, 0.66, 0.90, 0.96],
        accent_soft: [0.10, 0.18, 0.34, 0.96],
    }
}

fn current_quote(start_seed: u64, elapsed: f32) -> &'static Quote {
    let len = QUOTES.len().max(1);
    let step = (elapsed.max(0.0) as u64) / ROTATE_SECS;
    let index = ((start_seed / ROTATE_SECS) as usize + step as usize) % len;
    &QUOTES[index]
}

fn load_quotes() -> Vec<Quote> {
    let plugin: PluginFile =
        serde_json::from_str(include_str!("../data/plugin.json")).expect("decode quotes plugin");
    plugin
        .slides
        .into_iter()
        .flat_map(|slide| slide.data.inline)
        .map(|quote| Quote {
            text: text_slide::normalize_text(&quote.text),
            author: text_slide::normalize_text(&quote.author),
        })
        .collect()
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
        quotes_slide_spec().validate().unwrap();
    }

    #[test]
    fn quote_archive_loaded() {
        assert!(QUOTES.len() > 50);
        assert!(!QUOTES[0].author.is_empty());
    }

    #[test]
    fn overlay_rotates_after_interval() {
        let first = build_overlay(0, 0.0);
        let second = build_overlay(0, ROTATE_SECS as f32);
        assert_ne!(first.vertices, second.vertices);
    }
}
