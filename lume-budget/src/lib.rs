use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::Acer710_CGA));

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct BudgetEntry {
    pub category: String,
    pub amount: String,
    pub status: String,
    #[serde(rename = "type")]
    pub entry_type: String,
}

#[derive(Deserialize)]
struct BudgetFile {
    budget: Vec<BudgetEntry>,
}

static ENTRIES: Lazy<Vec<BudgetEntry>> = Lazy::new(load_budget);
static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| text_slide::serialize_spec(&budget_slide_spec()));

pub fn budget_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec("budget_scene", build_overlay(), palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn build_overlay() -> vzglyd_text_slide::VRX_64_slide::RuntimeOverlay<Vertex> {
    let mut blocks = vec![
        TextBlock {
            text: "MONTHLY BUDGET",
            x: 160.0,
            y: 28.0,
            scale: 1.10,
            color: [0.98, 0.92, 0.54, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
        TextBlock {
            text: "embedded dashboard budget",
            x: 160.0,
            y: 48.0,
            scale: 0.85,
            color: [0.70, 0.74, 0.82, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
    ];
    let amount_lines: Vec<String> = ENTRIES
        .iter()
        .take(8)
        .map(|entry| format!("{:<15} {}", entry.category, entry.amount))
        .collect();

    for (idx, entry) in ENTRIES.iter().take(8).enumerate() {
        let y = 76.0 + idx as f32 * 18.0;
        blocks.push(TextBlock {
            text: &amount_lines[idx],
            x: 34.0,
            y,
            scale: 0.95,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Left,
            wrap_cols: None,
        });
        blocks.push(TextBlock {
            text: &entry.status,
            x: 48.0,
            y: y + 9.0,
            scale: 0.75,
            color: status_color(&entry.status, &entry.entry_type),
            align: TextAlign::Left,
            wrap_cols: None,
        });
    }

    text_slide::compose_overlay(&blocks, &FONT)
}

fn load_budget() -> Vec<BudgetEntry> {
    let file: BudgetFile =
        serde_yaml::from_str(include_str!("../data/budget.yaml")).expect("decode budget yaml");
    file.budget
        .into_iter()
        .map(|entry| BudgetEntry {
            category: text_slide::normalize_text(&entry.category),
            amount: text_slide::normalize_text(&entry.amount),
            status: text_slide::normalize_text(&entry.status),
            entry_type: text_slide::normalize_text(&entry.entry_type),
        })
        .collect()
}

fn palette() -> Palette {
    Palette {
        background: [0.02, 0.04, 0.10, 1.0],
        panel: [0.06, 0.08, 0.16, 0.96],
        accent: [0.88, 0.72, 0.18, 0.96],
        accent_soft: [0.22, 0.18, 0.06, 0.96],
    }
}

fn status_color(status: &str, entry_type: &str) -> [f32; 4] {
    match status {
        "paid" => [0.50, 0.92, 0.60, 1.0],
        "spent" => [0.78, 0.82, 0.88, 1.0],
        "due" => [1.0, 0.62, 0.54, 1.0],
        "saved" => [0.44, 0.90, 0.96, 1.0],
        _ if entry_type == "savings" => [0.44, 0.90, 0.96, 1.0],
        _ => [0.78, 0.82, 0.88, 1.0],
    }
}

#[cfg(target_arch = "wasm32")]
fn slide_init() -> i32 { 0 }

#[cfg(target_arch = "wasm32")]
fn slide_update(_dt: f32) -> i32 { 0 }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_valid() {
        budget_slide_spec().validate().unwrap();
    }

    #[test]
    fn budget_archive_loaded() {
        assert_eq!(ENTRIES.len(), 8);
        assert_eq!(ENTRIES[0].category, "Rent");
    }
}
