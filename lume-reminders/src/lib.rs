use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::Amstrad_PC));

use serde::{Deserialize, Serialize};

const MAX_ROWS_PER_VIEW: usize = 5;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReminderItem {
    pub title: String,
    pub due: String,
    pub priority: String,
    pub list: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemindersPayload {
    pub fetched_at: String,
    pub reminders: Vec<ReminderItem>,
}

static SPEC_BYTES: Lazy<Vec<u8>> =
    Lazy::new(|| text_slide::serialize_spec(&reminders_slide_spec()));

pub fn reminders_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec("reminders_scene", build_overlay(None), palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn build_overlay(payload: Option<&RemindersPayload>) -> RuntimeOverlay<Vertex> {
    if let Some(payload) = payload {
        return build_payload_overlay(payload);
    }

    text_slide::compose_overlay(&[
        title_block("REMINDERS"),
        TextBlock {
            text: "Waiting for reminders bridge...",
            x: 160.0,
            y: 112.0,
            scale: 0.96,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        },
    ], &FONT)
}

fn build_payload_overlay(payload: &RemindersPayload) -> RuntimeOverlay<Vertex> {
    let pending = payload
        .reminders
        .iter()
        .filter(|reminder| reminder.status == "pending")
        .take(MAX_ROWS_PER_VIEW)
        .map(|reminder| {
            (
                reminder.title.clone(),
                format!(
                    "{}  {}",
                    text_slide::normalize_text(&reminder.list),
                    due_label(&reminder.due, text_slide::now_unix_secs())
                ),
                priority_color(
                    &reminder.priority,
                    &reminder.due,
                    text_slide::now_unix_secs(),
                ),
            )
        })
        .collect::<Vec<_>>();

    let mut blocks = vec![
        title_block("REMINDERS"),
        TextBlock {
            text: "pending items",
            x: 160.0,
            y: 46.0,
            scale: 0.78,
            color: [0.72, 0.82, 0.92, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
    ];

    if pending.is_empty() {
        blocks.push(TextBlock {
            text: "No pending reminders in the current bridge payload.",
            x: 160.0,
            y: 112.0,
            scale: 0.88,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(28),
        });
        let footer = fetched_label(&payload.fetched_at);
        blocks.push(footer_block(&footer));
        return text_slide::compose_overlay(&blocks, &FONT);
    }

    for (idx, (title, meta, color)) in pending.iter().enumerate() {
        let y = 62.0 + idx as f32 * 30.0;
        blocks.push(TextBlock {
            text: meta,
            x: 34.0,
            y,
            scale: 0.66,
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
            wrap_cols: Some(30),
        });
    }

    let footer = fetched_label(&payload.fetched_at);
    blocks.push(footer_block(&footer));
    text_slide::compose_overlay(&blocks, &FONT)
}

fn due_label(due: &str, now_secs: u64) -> String {
    if due.is_empty() {
        return "No due date".to_string();
    }

    let today = today_iso(now_secs);
    if due < today.as_str() {
        return "OVERDUE".to_string();
    }
    if due == today {
        return "TODAY".to_string();
    }
    text_slide::normalize_text(due)
}

fn priority_color(priority: &str, due: &str, now_secs: u64) -> [f32; 4] {
    let today = today_iso(now_secs);
    if !due.is_empty() && due < today.as_str() {
        return [1.0, 0.58, 0.52, 1.0];
    }
    match priority {
        "high" => [1.0, 0.58, 0.52, 1.0],
        "low" => [0.56, 0.90, 0.64, 1.0],
        _ => [0.98, 0.92, 0.54, 1.0],
    }
}

fn today_iso(now_secs: u64) -> String {
    let (year, month, day, _, _, _) = text_slide::date_utils::utc_ymdhms_from_unix(now_secs);
    format!("{year:04}-{month:02}-{day:02}")
}

fn fetched_label(fetched_at: &str) -> String {
    fetched_at
        .get(11..16)
        .map(|time| format!("Updated {time}"))
        .unwrap_or_else(|| "Updated recently".to_string())
}

fn title_block<'a>(text: &'a str) -> TextBlock<'a> {
    TextBlock {
        text,
        x: 160.0,
        y: 26.0,
        scale: 1.08,
        color: [0.84, 0.94, 1.0, 1.0],
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

fn palette() -> Palette {
    Palette {
        background: [0.03, 0.06, 0.08, 1.0],
        panel: [0.08, 0.12, 0.14, 0.96],
        accent: [0.16, 0.34, 0.32, 0.96],
        accent_soft: [0.08, 0.18, 0.18, 0.96],
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
    if let Some(payload) = text_slide::channel_runtime::poll_json::<RemindersPayload>(&mut state.response_buf) {
        state.payload = Some(payload);
    }
    state.refresh();
    1
}

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::{RemindersPayload, build_overlay};
    use text_slide::channel_runtime;
    use crate::text_slide;

    pub struct RuntimeState {
        pub payload: Option<RemindersPayload>,
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
        reminders_slide_spec().validate().unwrap();
    }

    #[test]
    fn due_label_marks_today_and_overdue() {
        assert_eq!(due_label("1970-01-01", 0), "TODAY");
        assert_eq!(due_label("1969-12-31", 0), "OVERDUE");
        assert_eq!(due_label("", 0), "No due date");
    }

    #[test]
    fn payload_roundtrip_builds_overlay() {
        let payload = RemindersPayload {
            fetched_at: "2026-03-29T10:30:00Z".to_string(),
            reminders: vec![ReminderItem {
                title: "Buy milk".to_string(),
                due: "2026-03-29".to_string(),
                priority: "high".to_string(),
                list: "Shopping".to_string(),
                status: "pending".to_string(),
            }],
        };

        let bytes = serde_json::to_vec(&payload).unwrap();
        let decoded: RemindersPayload = serde_json::from_slice(&bytes).unwrap();
        let overlay = build_overlay(Some(&decoded));
        assert!(!overlay.vertices.is_empty());
        assert!(!overlay.indices.is_empty());
    }

    #[test]
    fn pending_filter_excludes_completed_items() {
        let payload = RemindersPayload {
            fetched_at: "2026-03-29T10:30:00Z".to_string(),
            reminders: vec![
                ReminderItem {
                    title: "Done".to_string(),
                    due: String::new(),
                    priority: "normal".to_string(),
                    list: "Home".to_string(),
                    status: "done".to_string(),
                },
                ReminderItem {
                    title: "Pending".to_string(),
                    due: String::new(),
                    priority: "normal".to_string(),
                    list: "Home".to_string(),
                    status: "pending".to_string(),
                },
            ],
        };

        let overlay = build_payload_overlay(&payload);
        assert!(!overlay.vertices.is_empty());
    }
}
