use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use serde::Deserialize;
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

// this works because Font::Font8x8 shows something up:
// static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::Font8x8));

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::EGA_8x8));

const ROTATE_SECS: u64 = 10;

#[derive(Clone, Debug, Deserialize)]
struct Affirmation {
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
    inline: Vec<Affirmation>,
}

static AFFIRMATIONS: Lazy<Vec<Affirmation>> = Lazy::new(load_affirmations);
static SPEC_BYTES: Lazy<Vec<u8>> =
    Lazy::new(|| text_slide::serialize_spec(&affirmations_slide_spec()));

pub fn build_overlay(start_seed: u64, elapsed: f32) -> RuntimeOverlay<Vertex> {
    let affirmation = current_affirmation(start_seed, elapsed);
    let footer = if affirmation.author.is_empty() {
        "daily affirmation".to_string()
    } else {
        format!("- {}", affirmation.author)
    };

    text_slide::compose_overlay(&[
        TextBlock {
            text: "AFFIRMATION",
            x: 160.0,
            y: 34.0,
            scale: 1.10,
            color: [1.0, 0.86, 0.96, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
        TextBlock {
            text: &affirmation.text,
            x: 160.0,
            y: 78.0,
            scale: 1.18,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(28),
        },
        TextBlock {
            text: &footer,
            x: 160.0,
            y: 192.0,
            scale: 0.95,
            color: [1.0, 0.80, 0.94, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
    ], &FONT)
}

pub fn affirmations_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec(
        "affirmations_scene",
        build_overlay(text_slide::now_unix_secs(), 0.0),
        palette(),
        FONT.atlas.clone(),
    )
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn palette() -> Palette {
    Palette {
        background: [0.10, 0.03, 0.10, 1.0],
        panel: [0.18, 0.06, 0.18, 0.96],
        accent: [0.78, 0.20, 0.58, 0.96],
        accent_soft: [0.34, 0.10, 0.28, 0.96],
    }
}

fn current_affirmation(start_seed: u64, elapsed: f32) -> &'static Affirmation {
    let len = AFFIRMATIONS.len().max(1);
    let step = (elapsed.max(0.0) as u64) / ROTATE_SECS;
    let index = ((start_seed / ROTATE_SECS) as usize + step as usize) % len;
    &AFFIRMATIONS[index]
}

fn load_affirmations() -> Vec<Affirmation> {
    let plugin: PluginFile = serde_json::from_str(include_str!("../data/plugin.json"))
        .expect("decode affirmations plugin");
    plugin
        .slides
        .into_iter()
        .flat_map(|slide| slide.data.inline)
        .map(|entry| Affirmation {
            text: text_slide::normalize_text(&entry.text),
            author: text_slide::normalize_text(&entry.author),
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
        affirmations_slide_spec().validate().unwrap();
    }

    #[test]
    fn affirmation_archive_loaded() {
        assert!(AFFIRMATIONS.len() > 50);
    }

    #[test]
    fn overlay_rotates_after_interval() {
        let first = build_overlay(0, 0.0);
        let second = build_overlay(0, ROTATE_SECS as f32);
        assert_ne!(first.vertices, second.vertices);
    }
}
