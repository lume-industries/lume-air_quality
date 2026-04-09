use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::VGA_8x14));

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForecastDay {
    pub day: String,
    pub condition: String,
    pub high: i32,
    pub low: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherPayload {
    pub location: String,
    pub updated: String,
    pub days: Vec<ForecastDay>,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    data: Vec<LocationEntry>,
}

#[derive(Deserialize)]
struct LocationEntry {
    geohash: String,
    name: String,
    state: Option<String>,
}

#[derive(Deserialize)]
struct ForecastResponse {
    #[serde(default)]
    data: Vec<ForecastEntry>,
}

#[derive(Deserialize)]
struct ForecastEntry {
    date: String,
    temp_max: Option<f32>,
    temp_min: Option<f32>,
    icon_descriptor: Option<String>,
}

static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| text_slide::serialize_spec(&weather_slide_spec()));

pub fn parse_search_result(body: &str) -> Result<(String, String), String> {
    let response: SearchResponse = serde_json::from_str(body)
        .map_err(|error| format!("invalid weather search JSON: {error}"))?;
    let first = response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| "weather search returned no locations".to_string())?;
    let label = match first.state {
        Some(state) if !state.is_empty() => format!("{}, {}", first.name, state),
        _ => first.name,
    };
    Ok((first.geohash, label))
}

pub fn parse_forecast_payload(
    location: String,
    body: &str,
    now_secs: u64,
) -> Result<WeatherPayload, String> {
    let response: ForecastResponse = serde_json::from_str(body)
        .map_err(|error| format!("invalid weather forecast JSON: {error}"))?;
    let mut days = Vec::new();
    for entry in response.data.into_iter().take(7) {
        let (Some(high), Some(low)) = (entry.temp_max, entry.temp_min) else {
            continue;
        };
        days.push(ForecastDay {
            day: text_slide::date_utils::weekday_abbrev_from_iso(&entry.date)
                .unwrap_or("???")
                .to_string(),
            condition: map_condition(entry.icon_descriptor.as_deref().unwrap_or("cloudy")),
            high: high.round() as i32,
            low: low.round() as i32,
        });
    }

    if days.is_empty() {
        return Err("weather forecast contained no usable daily entries".to_string());
    }

    Ok(WeatherPayload {
        location,
        updated: format!("Updated {}", text_slide::date_utils::utc_hhmm_from_unix(now_secs)),
        days,
    })
}

pub fn weather_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec("weather_scene", build_overlay(None), palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn build_overlay(payload: Option<&WeatherPayload>) -> RuntimeOverlay<Vertex> {
    if let Some(payload) = payload {
        let mut blocks = vec![TextBlock {
            text: "WEATHER",
            x: 160.0,
            y: 26.0,
            scale: 1.10,
            color: [0.86, 0.96, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        }];
        blocks.push(TextBlock {
            text: &payload.location,
            x: 160.0,
            y: 46.0,
            scale: 0.85,
            color: [0.72, 0.82, 0.92, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        });

        let rows: Vec<String> = payload
            .days
            .iter()
            .map(|day| {
                format!(
                    "{:>3}  {:<14} {:>2} / {:>2}",
                    day.day, day.condition, day.high, day.low
                )
            })
            .collect();
        for (idx, row) in rows.iter().enumerate() {
            blocks.push(TextBlock {
                text: row,
                x: 38.0,
                y: 74.0 + idx as f32 * 18.0,
                scale: 0.92,
                color: [1.0, 1.0, 1.0, 1.0],
                align: TextAlign::Left,
                wrap_cols: None,
            });
        }
        blocks.push(TextBlock {
            text: &payload.updated,
            x: 160.0,
            y: 196.0,
            scale: 0.80,
            color: [0.68, 0.76, 0.86, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        });
        return text_slide::compose_overlay(&blocks, &FONT);
    }

    text_slide::compose_overlay(&[
        TextBlock {
            text: "WEATHER",
            x: 160.0,
            y: 26.0,
            scale: 1.10,
            color: [0.86, 0.96, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
        TextBlock {
            text: "Loading BOM forecast...",
            x: 160.0,
            y: 110.0,
            scale: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        },
    ], &FONT)
}

fn palette() -> Palette {
    Palette {
        background: [0.03, 0.07, 0.12, 1.0],
        panel: [0.08, 0.12, 0.20, 0.96],
        accent: [0.28, 0.56, 0.92, 0.96],
        accent_soft: [0.10, 0.20, 0.38, 0.96],
    }
}

fn map_condition(icon: &str) -> String {
    let normalized = icon.to_ascii_lowercase().replace(['-', ' '], "_");
    match normalized.as_str() {
        "sunny" | "mostly_sunny" | "clear" => "sunny".to_string(),
        "partly_cloudy" => "part cloud".to_string(),
        "cloudy" => "cloudy".to_string(),
        "haze" | "hazy" | "fog" => "fog".to_string(),
        "light_shower" | "light_showers" | "light_rain" => "light rain".to_string(),
        "shower" | "showers" | "rain" => "rain".to_string(),
        "heavy_shower" | "heavy_showers" | "heavy_rain" => "heavy rain".to_string(),
        "storm" | "storms" | "thunderstorm" => "storm".to_string(),
        "wind" | "windy" | "dust" | "dusty" => "windy".to_string(),
        "snow" => "snow".to_string(),
        "frost" => "cold".to_string(),
        "cyclone" | "tropical_cyclone" => "cyclone".to_string(),
        _ => "cloudy".to_string(),
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
    if let Some(payload) = text_slide::channel_runtime::poll_json::<WeatherPayload>(&mut state.response_buf) {
        state.payload = Some(payload);
    }
    state.refresh();
    1
}

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::{WeatherPayload, build_overlay};
    use text_slide::channel_runtime;
    use crate::text_slide;

    pub struct RuntimeState {
        pub payload: Option<WeatherPayload>,
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
        weather_slide_spec().validate().unwrap();
    }

    #[test]
    fn search_result_extracts_geohash_and_label() {
        let body = r#"{"data":[{"geohash":"r3gx2sp","name":"Kyneton","state":"VIC"}]}"#;
        let (geohash, label) = parse_search_result(body).expect("parse search response");
        assert_eq!(geohash, "r3gx2sp");
        assert_eq!(label, "Kyneton, VIC");
    }

    #[test]
    fn forecast_payload_uses_weekdays_and_conditions() {
        let body = r#"{
            "data": [
                {"date":"2026-03-19T00:00:00Z","temp_max":27,"temp_min":18,"icon_descriptor":"partly_cloudy"},
                {"date":"2026-03-20T00:00:00Z","temp_max":33,"temp_min":22,"icon_descriptor":"sunny"}
            ]
        }"#;
        let payload = parse_forecast_payload("Kyneton, VIC".to_string(), body, 0)
            .expect("parse forecast payload");
        assert_eq!(payload.days[0].day, "Thu");
        assert_eq!(payload.days[0].condition, "part cloud");
        assert_eq!(payload.days[1].high, 33);
    }
}
