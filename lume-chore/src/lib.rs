use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::Acer710_CGA));

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ChoreEntry {
    pub name: String,
    pub category: String,
    pub duration: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

#[derive(Deserialize)]
struct ChoreFile {
    chores: Vec<ChoreEntry>,
}

static CHORES: Lazy<Vec<ChoreEntry>> = Lazy::new(load_chores);
static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| text_slide::serialize_spec(&chore_slide_spec()));

pub fn build_overlay(epoch_secs: u64) -> RuntimeOverlay<Vertex> {
    let chore = pick_chore(epoch_secs);
    text_slide::compose_overlay(&[
        TextBlock {
            text: "CHORE OF THE HOUR",
            x: 160.0,
            y: 28.0,
            scale: 1.05,
            color: [1.0, 0.80, 0.56, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
        TextBlock {
            text: "Just do this one thing:",
            x: 160.0,
            y: 56.0,
            scale: 0.85,
            color: [0.80, 0.84, 0.92, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
        TextBlock {
            text: &chore.name,
            x: 160.0,
            y: 92.0,
            scale: 1.20,
            color: category_color(&chore.category),
            align: TextAlign::Center,
            wrap_cols: Some(22),
        },
        TextBlock {
            text: &chore.duration,
            x: 160.0,
            y: 176.0,
            scale: 0.90,
            color: [0.86, 0.90, 0.96, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
        TextBlock {
            text: &chore.category,
            x: 160.0,
            y: 194.0,
            scale: 0.80,
            color: category_color(&chore.category),
            align: TextAlign::Center,
            wrap_cols: None,
        },
    ], &FONT)
}

pub fn chore_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec(
        "chore_scene",
        build_overlay(text_slide::now_unix_secs()),
        palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn load_chores() -> Vec<ChoreEntry> {
    let file: ChoreFile =
        serde_yaml::from_str(include_str!("../data/chores.yaml")).expect("decode chore yaml");
    file.chores
        .into_iter()
        .map(|entry| ChoreEntry {
            name: text_slide::normalize_text(&entry.name),
            category: text_slide::normalize_text(&entry.category),
            duration: text_slide::normalize_text(&entry.duration),
            weight: entry.weight.max(1),
        })
        .collect()
}

fn default_weight() -> u32 {
    1
}

fn palette() -> Palette {
    Palette {
        background: [0.06, 0.03, 0.08, 1.0],
        panel: [0.11, 0.06, 0.13, 0.96],
        accent: [0.84, 0.42, 0.16, 0.96],
        accent_soft: [0.32, 0.12, 0.08, 0.96],
    }
}

fn pick_chore(epoch_secs: u64) -> &'static ChoreEntry {
    let hour_bucket = epoch_secs / 3600;
    let mut weighted_indices = Vec::new();
    for (idx, chore) in CHORES.iter().enumerate() {
        for _ in 0..chore.weight {
            weighted_indices.push(idx);
        }
    }
    let selected = weighted_indices[(mix64(hour_bucket) as usize) % weighted_indices.len().max(1)];
    &CHORES[selected]
}

fn mix64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn category_color(category: &str) -> [f32; 4] {
    match category {
        "kitchen" => [1.0, 0.72, 0.38, 1.0],
        "waste" => [0.58, 0.92, 0.54, 1.0],
        "floors" => [0.50, 0.88, 1.0, 1.0],
        "tidying" => [0.98, 0.92, 0.54, 1.0],
        "bathroom" => [0.58, 0.80, 1.0, 1.0],
        "laundry" => [0.96, 0.58, 0.98, 1.0],
        "bedroom" => [1.0, 0.68, 0.86, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
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
    runtime_state::state().refresh();
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
fn slide_update(_dt: f32) -> i32 {
    runtime_state::state().refresh();
    1
}

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    pub struct RuntimeState {
        pub overlay_bytes: Vec<u8>,
    }

    impl RuntimeState {
        fn new() -> Self {
            let mut state = Self {
                overlay_bytes: Vec::new(),
            };
            state.refresh();
            state
        }

        pub fn refresh(&mut self) {
            self.overlay_bytes = super::text_slide::serialize_overlay(&super::build_overlay(
                super::text_slide::now_unix_secs(),
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
        chore_slide_spec().validate().unwrap();
    }

    #[test]
    fn chore_archive_loaded() {
        assert!(CHORES.len() > 10);
    }

    #[test]
    fn pick_is_stable_for_same_hour() {
        let a = pick_chore(3_600);
        let b = pick_chore(3_600 + 120);
        assert_eq!(a.name, b.name);
    }
}
