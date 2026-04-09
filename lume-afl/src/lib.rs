use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::EGA_8x8));

use serde::{Deserialize, Serialize};

const MAX_LADDER_ROWS: usize = 8;
const MAX_FIXTURES: usize = 5;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LadderRow {
    pub rank: String,
    pub team: String,
    pub wins: String,
    pub losses: String,
    pub points: String,
    pub percentage: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixtureRow {
    pub round: String,
    pub match_label: String,
    pub date: String,
    pub venue: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AflPayload {
    pub updated: String,
    pub ladder: Vec<LadderRow>,
    pub fixtures: Vec<FixtureRow>,
}

#[derive(Deserialize)]
struct LadderResponse {
    #[serde(default)]
    standings: Vec<LadderEntry>,
}

#[derive(Deserialize)]
struct LadderEntry {
    rank: Option<u32>,
    name: Option<String>,
    wins: Option<u32>,
    losses: Option<u32>,
    percentage: Option<f32>,
    pts: Option<u32>,
}

#[derive(Deserialize)]
struct GamesResponse {
    #[serde(default)]
    games: Vec<GameEntry>,
}

#[derive(Deserialize)]
struct GameEntry {
    round: Option<u32>,
    hteam: Option<String>,
    ateam: Option<String>,
    date: Option<String>,
    venue: Option<String>,
}

static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| text_slide::serialize_spec(&afl_slide_spec()));

pub fn parse_ladder(body: &str) -> Result<Vec<LadderRow>, String> {
    let mut response: LadderResponse =
        serde_json::from_str(body).map_err(|error| format!("invalid AFL ladder JSON: {error}"))?;
    response
        .standings
        .sort_by_key(|entry| entry.rank.unwrap_or(u32::MAX));

    Ok(response
        .standings
        .into_iter()
        .filter_map(|entry| {
            let rank = entry.rank?;
            let team = entry.name?;
            Some(LadderRow {
                rank: rank.to_string(),
                team: truncate(&text_slide::normalize_text(&team), 18),
                wins: entry.wins.unwrap_or(0).to_string(),
                losses: entry.losses.unwrap_or(0).to_string(),
                points: entry.pts.unwrap_or(0).to_string(),
                percentage: format_percentage(entry.percentage),
            })
        })
        .collect())
}

pub fn parse_fixtures(body: &str, now_label: &str) -> Result<Vec<FixtureRow>, String> {
    let mut response: GamesResponse =
        serde_json::from_str(body).map_err(|error| format!("invalid AFL games JSON: {error}"))?;
    response
        .games
        .sort_by(|left, right| left.date.cmp(&right.date));

    Ok(response
        .games
        .into_iter()
        .filter(|game| game.date.as_deref().unwrap_or_default() >= now_label)
        .filter_map(|game| {
            let home = game.hteam?;
            let away = game.ateam?;
            let date = game.date?;
            Some(FixtureRow {
                round: format!("R{}", game.round.unwrap_or(0)),
                match_label: truncate(&text_slide::normalize_text(&format!("{home} v {away}")), 24),
                date: format_game_date(&date),
                venue: truncate(
                    &text_slide::normalize_text(game.venue.as_deref().unwrap_or("TBA")),
                    22,
                ),
            })
        })
        .take(MAX_FIXTURES)
        .collect())
}

pub fn compose_payload(
    ladder_body: &str,
    fixtures_body: &str,
    now_secs: u64,
) -> Result<AflPayload, String> {
    let ladder = parse_ladder(ladder_body)?;
    let fixtures = parse_fixtures(fixtures_body, &text_slide::date_utils::utc_datetime_label(now_secs))?;
    if ladder.is_empty() && fixtures.is_empty() {
        return Err("AFL feeds returned no ladder rows or fixtures".to_string());
    }

    Ok(AflPayload {
        updated: format!("Updated {}", text_slide::date_utils::utc_hhmm_from_unix(now_secs)),
        ladder,
        fixtures,
    })
}

pub fn afl_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec("afl_scene", build_overlay(None, 0), palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn build_overlay(payload: Option<&AflPayload>, view_index: usize) -> RuntimeOverlay<Vertex> {
    if let Some(payload) = payload {
        return if view_index % 2 == 0 {
            build_ladder_overlay(payload)
        } else {
            build_fixtures_overlay(payload)
        };
    }

    text_slide::compose_overlay(&[
        title_block("AFL SNAPSHOT"),
        TextBlock {
            text: "Loading ladder and fixtures...",
            x: 160.0,
            y: 112.0,
            scale: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        },
    ], &FONT)
}

fn build_ladder_overlay(payload: &AflPayload) -> RuntimeOverlay<Vertex> {
    let mut blocks = vec![
        title_block("AFL LADDER"),
        TextBlock {
            text: "team             W  L  pts",
            x: 34.0,
            y: 56.0,
            scale: 0.76,
            color: [0.70, 0.80, 0.92, 1.0],
            align: TextAlign::Left,
            wrap_cols: None,
        },
    ];

    let rows: Vec<String> = payload
        .ladder
        .iter()
        .take(MAX_LADDER_ROWS)
        .map(|row| {
            format!(
                "{:>2}. {:<15} {:>2} {:>2} {:>3}",
                row.rank, row.team, row.wins, row.losses, row.points
            )
        })
        .collect();

    for (idx, row) in rows.iter().enumerate() {
        blocks.push(TextBlock {
            text: row,
            x: 34.0,
            y: 74.0 + idx as f32 * 16.0,
            scale: 0.82,
            color: ladder_color(idx),
            align: TextAlign::Left,
            wrap_cols: None,
        });
    }

    let percentage_lines: Vec<String> = payload
        .ladder
        .iter()
        .take(MAX_LADDER_ROWS)
        .map(|row| format!("{} pct", row.percentage))
        .collect();
    for (idx, line) in percentage_lines.iter().enumerate() {
        blocks.push(TextBlock {
            text: line,
            x: 270.0,
            y: 74.0 + idx as f32 * 16.0,
            scale: 0.62,
            color: [0.78, 0.82, 0.90, 1.0],
            align: TextAlign::Right,
            wrap_cols: None,
        });
    }

    blocks.push(footer_block(&payload.updated));
    text_slide::compose_overlay(&blocks, &FONT)
}

fn build_fixtures_overlay(payload: &AflPayload) -> RuntimeOverlay<Vertex> {
    if payload.fixtures.is_empty() {
        return text_slide::compose_overlay(&[
            title_block("NEXT ROUND"),
            TextBlock {
                text: "upcoming Squiggle fixtures",
                x: 160.0,
                y: 46.0,
                scale: 0.80,
                color: [0.70, 0.80, 0.92, 1.0],
                align: TextAlign::Center,
                wrap_cols: None,
            },
            TextBlock {
                text: "No upcoming fixtures in the current feed.",
                x: 160.0,
                y: 110.0,
                scale: 0.92,
                color: [1.0, 1.0, 1.0, 1.0],
                align: TextAlign::Center,
                wrap_cols: Some(24),
            },
            footer_block(&payload.updated),
        ], &FONT);
    }

    let mut blocks = vec![
        title_block("NEXT ROUND"),
        TextBlock {
            text: "upcoming Squiggle fixtures",
            x: 160.0,
            y: 46.0,
            scale: 0.80,
            color: [0.70, 0.80, 0.92, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
    ];
    let overlay = {
        let headers: Vec<String> = payload
            .fixtures
            .iter()
            .map(|fixture| format!("{}  {}", fixture.round, fixture.date))
            .collect();
        for (idx, fixture) in payload.fixtures.iter().enumerate() {
            let y = 68.0 + idx as f32 * 28.0;
            blocks.push(TextBlock {
                text: &headers[idx],
                x: 34.0,
                y,
                scale: 0.72,
                color: [0.98, 0.86, 0.46, 1.0],
                align: TextAlign::Left,
                wrap_cols: None,
            });
            blocks.push(TextBlock {
                text: &fixture.match_label,
                x: 34.0,
                y: y + 10.0,
                scale: 0.92,
                color: [1.0, 1.0, 1.0, 1.0],
                align: TextAlign::Left,
                wrap_cols: Some(25),
            });
            blocks.push(TextBlock {
                text: &fixture.venue,
                x: 34.0,
                y: y + 20.0,
                scale: 0.68,
                color: [0.72, 0.80, 0.90, 1.0],
                align: TextAlign::Left,
                wrap_cols: Some(26),
            });
        }
        blocks.push(footer_block(&payload.updated));
        text_slide::compose_overlay(&blocks, &FONT)
    };
    overlay
}

fn title_block(text: &'static str) -> TextBlock<'static> {
    TextBlock {
        text,
        x: 160.0,
        y: 26.0,
        scale: 1.10,
        color: [0.98, 0.86, 0.46, 1.0],
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
        color: [0.72, 0.80, 0.90, 1.0],
        align: TextAlign::Center,
        wrap_cols: None,
    }
}

fn ladder_color(index: usize) -> [f32; 4] {
    match index {
        0..=3 => [0.56, 0.92, 0.62, 1.0],
        4..=7 => [0.98, 0.86, 0.46, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

fn palette() -> Palette {
    Palette {
        background: [0.05, 0.03, 0.02, 1.0],
        panel: [0.12, 0.06, 0.05, 0.96],
        accent: [0.84, 0.28, 0.18, 0.96],
        accent_soft: [0.32, 0.10, 0.06, 0.96],
    }
}

fn truncate(text: &str, max_len: usize) -> String {
    let normalized = text_slide::normalize_text(text);
    let count = normalized.chars().count();
    if count <= max_len {
        return normalized;
    }
    let mut truncated = normalized
        .chars()
        .take(max_len.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn format_percentage(value: Option<f32>) -> String {
    value
        .map(|value| format!("{value:.1}"))
        .unwrap_or_else(|| "0.0".to_string())
}

fn format_game_date(date: &str) -> String {
    if date.len() >= 16 {
        return format!("{}/{} {}", &date[8..10], &date[5..7], &date[11..16]);
    }
    text_slide::normalize_text(date)
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
    if let Some(payload) = text_slide::channel_runtime::poll_json::<AflPayload>(&mut state.response_buf) {
        state.payload = Some(payload);
    }
    state.refresh();
    1
}

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::{AflPayload, build_overlay};
    use text_slide::channel_runtime;
    use crate::text_slide;

    pub struct RuntimeState {
        pub payload: Option<AflPayload>,
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
        afl_slide_spec().validate().unwrap();
    }

    #[test]
    fn ladder_is_sorted_and_formatted() {
        let body = r#"{
            "standings": [
                {"rank": 2, "name": "Brisbane Lions", "wins": 4, "losses": 1, "percentage": 121.5, "pts": 16},
                {"rank": 1, "name": "Collingwood", "wins": 5, "losses": 0, "percentage": 133.5, "pts": 20}
            ]
        }"#;

        let ladder = parse_ladder(body).expect("parse ladder");
        assert_eq!(ladder[0].team, "Collingwood");
        assert_eq!(ladder[0].percentage, "133.5");
    }

    #[test]
    fn fixtures_are_filtered_and_truncated() {
        let body = r#"{
            "games": [
                {"round": 1, "hteam": "Cats", "ateam": "Dogs", "date": "2025-03-19 10:00:00", "venue": "Marvel Stadium"},
                {"round": 2, "hteam": "Very Long Home Team Name", "ateam": "Very Long Away Team Name", "date": "2025-03-20 19:40:00", "venue": "Melbourne Cricket Ground"}
            ]
        }"#;

        let fixtures = parse_fixtures(body, "2025-03-19 12:00:00").expect("parse fixtures");
        assert_eq!(fixtures.len(), 1);
        assert_eq!(fixtures[0].round, "R2");
        assert!(fixtures[0].match_label.len() <= 24);
        assert_eq!(fixtures[0].date, "20/03 19:40");
    }
}
