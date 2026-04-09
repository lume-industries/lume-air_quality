use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::EGA_8x8));

use serde::{Deserialize, Serialize};

const HISTORY_WINDOW_SECS: u64 = 86_400;
const WARNING_MS: u32 = 1_000;
const DEGRADED_MS: u32 = 3_000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerConfig {
    pub name: String,
    pub region: String,
    pub check_type: String,
    pub url: Option<String>,
    pub host: Option<String>,
    pub port: u16,
    pub timeout_ms: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistorySample {
    pub timestamp: u64,
    pub ok: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerStatusRow {
    pub name: String,
    pub region: String,
    pub check_type: String,
    pub status: String,
    pub uptime: String,
    pub response_ms: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServersPayload {
    pub updated: String,
    pub rows: Vec<ServerStatusRow>,
}

#[derive(Deserialize)]
struct ConfigFile {
    servers: Vec<ServerConfigEntry>,
}

#[derive(Deserialize)]
struct ServerConfigEntry {
    name: String,
    region: String,
    check_type: String,
    url: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    timeout_ms: Option<u32>,
}

static CONFIG: Lazy<Vec<ServerConfig>> = Lazy::new(|| {
    let file: ConfigFile =
        serde_json::from_str(include_str!("../config/servers.json")).expect("decode servers.json");
    file.servers
        .into_iter()
        .map(|entry| ServerConfig {
            name: text_slide::normalize_text(&entry.name),
            region: text_slide::normalize_text(&entry.region),
            check_type: text_slide::normalize_text(&entry.check_type),
            url: entry.url.map(|value| text_slide::normalize_text(&value)),
            host: entry.host.map(|value| text_slide::normalize_text(&value)),
            port: entry.port.unwrap_or(80),
            timeout_ms: entry.timeout_ms.unwrap_or(5_000),
        })
        .collect()
});
static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| text_slide::serialize_spec(&servers_slide_spec()));

pub fn load_server_config() -> Vec<ServerConfig> {
    CONFIG.clone()
}

pub fn update_history(entries: &mut Vec<HistorySample>, timestamp: u64, ok: bool) {
    entries.push(HistorySample { timestamp, ok });
    let cutoff = timestamp.saturating_sub(HISTORY_WINDOW_SECS);
    entries.retain(|entry| entry.timestamp >= cutoff);
}

pub fn uptime_pct(entries: &[HistorySample]) -> String {
    if entries.is_empty() {
        return "--".to_string();
    }
    let ok = entries.iter().filter(|entry| entry.ok).count() as f32;
    format!("{:.2}%", ok / entries.len() as f32 * 100.0)
}

pub fn derive_status(ok: bool, response_ms: u32, entries: &[HistorySample]) -> String {
    if !ok {
        let trailing_failures =
            entries.iter().rev().take(3).all(|entry| !entry.ok) && entries.len() >= 3;
        return if trailing_failures {
            "down".to_string()
        } else {
            "degraded".to_string()
        };
    }
    if response_ms >= DEGRADED_MS {
        "degraded".to_string()
    } else if response_ms >= WARNING_MS {
        "warning".to_string()
    } else {
        "healthy".to_string()
    }
}

pub fn servers_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec("servers_scene", build_overlay(None, 0), palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn build_overlay(payload: Option<&ServersPayload>, view_index: usize) -> RuntimeOverlay<Vertex> {
    if let Some(payload) = payload {
        return if view_index % 2 == 0 {
            build_status_view(payload)
        } else {
            build_summary_view(payload)
        };
    }

    text_slide::compose_overlay(&[
        title_block("SYSTEM STATUS"),
        TextBlock {
            text: "Loading server checks...",
            x: 160.0,
            y: 112.0,
            scale: 0.96,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        },
    ], &FONT)
}

fn build_status_view(payload: &ServersPayload) -> RuntimeOverlay<Vertex> {
    let mut blocks = vec![
        title_block("SYSTEM STATUS"),
        TextBlock {
            text: "name          uptime   ms   region",
            x: 30.0,
            y: 50.0,
            scale: 0.68,
            color: [0.74, 0.84, 0.94, 1.0],
            align: TextAlign::Left,
            wrap_cols: None,
        },
    ];
    let rows: Vec<String> = payload
        .rows
        .iter()
        .take(7)
        .map(|row| {
            format!(
                "{:<12} {:>7} {:>4} {:<6}",
                truncate(&row.name, 12),
                row.uptime,
                row.response_ms,
                truncate(&row.region, 6)
            )
        })
        .collect();
    for (idx, row) in rows.iter().enumerate() {
        blocks.push(TextBlock {
            text: row,
            x: 30.0,
            y: 70.0 + idx as f32 * 18.0,
            scale: 0.72,
            color: status_color(payload.rows[idx].status.as_str()),
            align: TextAlign::Left,
            wrap_cols: None,
        });
    }
    blocks.push(footer_block(&payload.updated));
    text_slide::compose_overlay(&blocks, &FONT)
}

fn build_summary_view(payload: &ServersPayload) -> RuntimeOverlay<Vertex> {
    let healthy = payload
        .rows
        .iter()
        .filter(|row| row.status == "healthy")
        .count();
    let warning = payload
        .rows
        .iter()
        .filter(|row| row.status == "warning")
        .count();
    let degraded = payload
        .rows
        .iter()
        .filter(|row| row.status == "degraded")
        .count();
    let down = payload
        .rows
        .iter()
        .filter(|row| row.status == "down")
        .count();
    let slowest = payload
        .rows
        .iter()
        .max_by_key(|row| row.response_ms.parse::<u32>().unwrap_or(0));
    let slowest_label = slowest
        .map(|row| format!("Slowest {} at {}ms", row.name, row.response_ms))
        .unwrap_or_else(|| "No server samples yet".to_string());

    text_slide::compose_overlay(&[
        title_block("HEALTH SUMMARY"),
        TextBlock {
            text: &format!("Healthy   {healthy}"),
            x: 46.0,
            y: 70.0,
            scale: 0.96,
            color: status_color("healthy"),
            align: TextAlign::Left,
            wrap_cols: None,
        },
        TextBlock {
            text: &format!("Warning   {warning}"),
            x: 46.0,
            y: 94.0,
            scale: 0.96,
            color: status_color("warning"),
            align: TextAlign::Left,
            wrap_cols: None,
        },
        TextBlock {
            text: &format!("Degraded  {degraded}"),
            x: 46.0,
            y: 118.0,
            scale: 0.96,
            color: status_color("degraded"),
            align: TextAlign::Left,
            wrap_cols: None,
        },
        TextBlock {
            text: &format!("Down      {down}"),
            x: 46.0,
            y: 142.0,
            scale: 0.96,
            color: status_color("down"),
            align: TextAlign::Left,
            wrap_cols: None,
        },
        TextBlock {
            text: &slowest_label,
            x: 46.0,
            y: 174.0,
            scale: 0.72,
            color: [0.82, 0.88, 0.96, 1.0],
            align: TextAlign::Left,
            wrap_cols: Some(34),
        },
        footer_block(&payload.updated),
    ], &FONT)
}

fn title_block(text: &'static str) -> TextBlock<'static> {
    TextBlock {
        text,
        x: 160.0,
        y: 26.0,
        scale: 1.08,
        color: [0.94, 0.86, 0.52, 1.0],
        align: TextAlign::Center,
        wrap_cols: None,
    }
}

fn footer_block<'a>(text: &'a str) -> TextBlock<'a> {
    TextBlock {
        text,
        x: 160.0,
        y: 198.0,
        scale: 0.76,
        color: [0.72, 0.82, 0.92, 1.0],
        align: TextAlign::Center,
        wrap_cols: None,
    }
}

fn status_color(status: &str) -> [f32; 4] {
    match status {
        "healthy" => [0.50, 0.92, 0.60, 1.0],
        "warning" => [0.98, 0.92, 0.54, 1.0],
        "degraded" => [1.0, 0.74, 0.44, 1.0],
        "down" => [1.0, 0.58, 0.52, 1.0],
        _ => [0.82, 0.88, 0.96, 1.0],
    }
}

fn truncate(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    let mut truncated = text
        .chars()
        .take(max_len.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn palette() -> Palette {
    Palette {
        background: [0.03, 0.05, 0.07, 1.0],
        panel: [0.08, 0.11, 0.15, 0.96],
        accent: [0.16, 0.32, 0.24, 0.96],
        accent_soft: [0.08, 0.18, 0.14, 0.96],
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
    let mut state = runtime_state::state();
    if let Some(payload) = text_slide::channel_runtime::poll_json::<ServersPayload>(&mut state.response_buf) {
        state.payload = Some(payload);
    }
    state.refresh();
    1
}

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::{ServersPayload, build_overlay};
    use text_slide::channel_runtime;
    use crate::text_slide;

    pub struct RuntimeState {
        pub payload: Option<ServersPayload>,
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
            let view = ((text_slide::now_unix_secs() / 12) % 2) as usize;
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
        servers_slide_spec().validate().unwrap();
    }

    #[test]
    fn config_loads_embedded_servers() {
        let config = load_server_config();
        assert_eq!(config.len(), 3);
        assert_eq!(config[0].check_type, "http");
        assert_eq!(config[1].check_type, "tcp");
    }

    #[test]
    fn history_prunes_and_formats_uptime() {
        let mut history = vec![HistorySample {
            timestamp: 1,
            ok: true,
        }];
        update_history(&mut history, HISTORY_WINDOW_SECS + 10, false);
        assert_eq!(history.len(), 1);
        assert_eq!(uptime_pct(&history), "0.00%");
    }

    #[test]
    fn status_thresholds_match_dashboard_behavior() {
        assert_eq!(derive_status(true, 50, &[]), "healthy");
        assert_eq!(derive_status(true, WARNING_MS + 1, &[]), "warning");
        assert_eq!(derive_status(true, DEGRADED_MS + 1, &[]), "degraded");
        let failures = vec![
            HistorySample {
                timestamp: 1,
                ok: false,
            },
            HistorySample {
                timestamp: 2,
                ok: false,
            },
            HistorySample {
                timestamp: 3,
                ok: false,
            },
        ];
        assert_eq!(derive_status(false, 0, &failures), "down");
    }
}
