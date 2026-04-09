use vzglyd_text_slide::{self as text_slide, Font, FontAssets, Lazy};
use text_slide::{Palette, TextAlign, TextBlock, Vertex, RuntimeOverlay, SlideSpec};

static FONT: Lazy<FontAssets> = Lazy::new(|| text_slide::make_font_assets(Font::VGA_8x14));

use serde::{Deserialize, Serialize};

const MAX_ROWS_PER_VIEW: usize = 8;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarEvent {
    pub title: String,
    pub date_iso: String,
    pub day_label: String,
    pub time_label: String,
    pub kind: String,
    pub attendees: u32,
    pub start_epoch: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarPayload {
    pub timezone: String,
    pub updated: String,
    pub events: Vec<CalendarEvent>,
}

static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| text_slide::serialize_spec(&calendar_slide_spec()));

pub fn infer_type(title: &str) -> String {
    let lower = title.to_ascii_lowercase();
    for (kind, keywords) in [
        (
            "standup",
            &[
                "standup",
                "stand-up",
                "stand up",
                "daily scrum",
                "daily sync",
            ][..],
        ),
        ("retro", &["retro", "retrospective"][..]),
        (
            "planning",
            &["planning", "sprint", "roadmap", "kickoff", "kick-off"][..],
        ),
        (
            "interview",
            &["interview", "hiring", "panel", "candidate"][..],
        ),
        (
            "presentation",
            &[
                "demo",
                "presentation",
                "showcase",
                "all hands",
                "all-hands",
                "town hall",
            ][..],
        ),
        (
            "personal",
            &["1:1", "one-on-one", "one on one", "catch up", "catch-up"][..],
        ),
        (
            "review",
            &["review", "rfc", "design review", "code review", "pr review"][..],
        ),
    ] {
        if keywords.iter().any(|keyword| lower.contains(keyword)) {
            return kind.to_string();
        }
    }
    "review".to_string()
}

pub fn parse_ics(
    ics_text: &str,
    now_secs: u64,
    days: u32,
    local_tz: &str,
) -> Result<Vec<CalendarEvent>, String> {
    let unfolded = unfold_ics(ics_text);
    let cutoff = now_secs + u64::from(days) * 86_400;
    let mut events = Vec::new();
    let mut current = EventBuilder::default();
    let mut in_event = false;

    for line in unfolded.lines() {
        match line {
            "BEGIN:VEVENT" => {
                in_event = true;
                current = EventBuilder::default();
            }
            "END:VEVENT" => {
                if in_event {
                    if let Some(event) =
                        std::mem::take(&mut current).finish(now_secs, cutoff, local_tz)?
                    {
                        events.push(event);
                    }
                }
                in_event = false;
            }
            _ if in_event => current.apply_line(line),
            _ => {}
        }
    }

    events.sort_by_key(|event| event.start_epoch);
    Ok(events)
}

pub fn parse_calendar_payload(
    ics_text: &str,
    now_secs: u64,
    local_tz: &str,
) -> Result<CalendarPayload, String> {
    let events = parse_ics(ics_text, now_secs, 7, local_tz)?;
    Ok(CalendarPayload {
        timezone: local_tz.to_string(),
        updated: format!("Updated {}", text_slide::date_utils::utc_hhmm_from_unix(now_secs)),
        events,
    })
}

pub fn calendar_slide_spec() -> SlideSpec<Vertex> {
    text_slide::default_panel_spec("calendar_scene", build_overlay(None, 0), palette(), FONT.atlas.clone())
}

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

fn build_overlay(payload: Option<&CalendarPayload>, view_index: usize) -> RuntimeOverlay<Vertex> {
    if let Some(payload) = payload {
        return if view_index % 2 == 0 {
            build_calendar_view(payload)
        } else {
            build_meetings_view(payload)
        };
    }

    text_slide::compose_overlay(&[
        title_block("CALENDAR"),
        TextBlock {
            text: "Loading calendar feed...",
            x: 160.0,
            y: 112.0,
            scale: 0.96,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        },
    ], &FONT)
}

fn build_calendar_view(payload: &CalendarPayload) -> RuntimeOverlay<Vertex> {
    let mut blocks = vec![
        title_block("CALENDAR"),
        TextBlock {
            text: "next 7 days",
            x: 160.0,
            y: 46.0,
            scale: 0.80,
            color: [0.74, 0.84, 0.94, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
    ];

    if payload.events.is_empty() {
        blocks.push(TextBlock {
            text: "No upcoming events in the next week.",
            x: 160.0,
            y: 112.0,
            scale: 0.92,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        });
        blocks.push(footer_block(&payload.updated));
        return text_slide::compose_overlay(&blocks, &FONT);
    }

    let mut last_day = String::new();
    let mut row = 0usize;
    let mut lines = Vec::new();
    for event in payload.events.iter().take(MAX_ROWS_PER_VIEW) {
        if event.day_label != last_day {
            lines.push((Some(event.day_label.clone()), String::new()));
            row += 1;
            last_day = event.day_label.clone();
        }
        lines.push((None, format!("{}  {}", event.time_label, event.title)));
        row += 1;
        if row >= MAX_ROWS_PER_VIEW {
            break;
        }
    }

    for (idx, (header, line)) in lines.iter().enumerate() {
        let y = 66.0 + idx as f32 * 18.0;
        if let Some(header) = header {
            blocks.push(TextBlock {
                text: header,
                x: 34.0,
                y,
                scale: 0.80,
                color: [0.98, 0.82, 0.48, 1.0],
                align: TextAlign::Left,
                wrap_cols: None,
            });
        } else {
            blocks.push(TextBlock {
                text: line,
                x: 46.0,
                y,
                scale: 0.76,
                color: [1.0, 1.0, 1.0, 1.0],
                align: TextAlign::Left,
                wrap_cols: Some(30),
            });
        }
    }
    blocks.push(footer_block(&payload.updated));
    text_slide::compose_overlay(&blocks, &FONT)
}

fn build_meetings_view(payload: &CalendarPayload) -> RuntimeOverlay<Vertex> {
    let mut blocks = vec![
        title_block("MEETINGS"),
        TextBlock {
            text: "type, time, attendees",
            x: 160.0,
            y: 46.0,
            scale: 0.78,
            color: [0.74, 0.84, 0.94, 1.0],
            align: TextAlign::Center,
            wrap_cols: None,
        },
    ];

    if payload.events.is_empty() {
        blocks.push(TextBlock {
            text: "No meetings scheduled.",
            x: 160.0,
            y: 112.0,
            scale: 0.92,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Center,
            wrap_cols: Some(24),
        });
        blocks.push(footer_block(&payload.updated));
        return text_slide::compose_overlay(&blocks, &FONT);
    }

    let lines: Vec<(String, String)> = payload
        .events
        .iter()
        .take(6)
        .map(|event| {
            (
                format!(
                    "{}  {:<11}  {}",
                    event.time_label,
                    event.kind.to_uppercase(),
                    event.attendees
                ),
                event.title.clone(),
            )
        })
        .collect();

    for (idx, (meta, title)) in lines.iter().enumerate() {
        let y = 66.0 + idx as f32 * 22.0;
        blocks.push(TextBlock {
            text: meta,
            x: 34.0,
            y,
            scale: 0.64,
            color: kind_color(payload.events[idx].kind.as_str()),
            align: TextAlign::Left,
            wrap_cols: None,
        });
        blocks.push(TextBlock {
            text: title,
            x: 34.0,
            y: y + 10.0,
            scale: 0.78,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Left,
            wrap_cols: Some(30),
        });
    }
    blocks.push(footer_block(&payload.updated));
    text_slide::compose_overlay(&blocks, &FONT)
}

fn title_block(text: &'static str) -> TextBlock<'static> {
    TextBlock {
        text,
        x: 160.0,
        y: 26.0,
        scale: 1.08,
        color: [0.94, 0.88, 0.54, 1.0],
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

fn kind_color(kind: &str) -> [f32; 4] {
    match kind {
        "standup" => [0.50, 0.92, 0.60, 1.0],
        "planning" => [1.0, 0.74, 0.44, 1.0],
        "retro" => [1.0, 0.66, 0.86, 1.0],
        "interview" => [1.0, 0.58, 0.52, 1.0],
        "presentation" => [0.98, 0.92, 0.54, 1.0],
        "personal" => [0.78, 0.68, 1.0, 1.0],
        _ => [0.52, 0.88, 1.0, 1.0],
    }
}

fn palette() -> Palette {
    Palette {
        background: [0.03, 0.05, 0.10, 1.0],
        panel: [0.08, 0.10, 0.18, 0.96],
        accent: [0.18, 0.32, 0.62, 0.96],
        accent_soft: [0.08, 0.16, 0.30, 0.96],
    }
}

#[derive(Default)]
struct EventBuilder {
    summary: Option<String>,
    dtstart: Option<(String, Option<String>, bool)>,
    attendees: u32,
}

impl EventBuilder {
    fn apply_line(&mut self, line: &str) {
        let Some((lhs, rhs)) = line.split_once(':') else {
            return;
        };
        let mut key_parts = lhs.split(';');
        let key = key_parts.next().unwrap_or_default();
        match key {
            "SUMMARY" => self.summary = Some(text_slide::normalize_text(rhs.trim())),
            "DTSTART" => {
                let mut tzid = None;
                let mut value_is_date = false;
                for part in key_parts {
                    if let Some(value) = part.strip_prefix("TZID=") {
                        tzid = Some(value.to_string());
                    }
                    if part == "VALUE=DATE" {
                        value_is_date = true;
                    }
                }
                self.dtstart = Some((rhs.trim().to_string(), tzid, value_is_date));
            }
            "ATTENDEE" => self.attendees += 1,
            _ => {}
        }
    }

    fn finish(
        self,
        now_secs: u64,
        cutoff_secs: u64,
        local_tz: &str,
    ) -> Result<Option<CalendarEvent>, String> {
        let Some(summary) = self.summary else {
            return Ok(None);
        };
        let Some((raw, tzid, value_is_date)) = self.dtstart else {
            return Ok(None);
        };

        let start_epoch = parse_ics_datetime(&raw, tzid.as_deref(), value_is_date, local_tz)?;
        if !(now_secs..=cutoff_secs).contains(&start_epoch) {
            return Ok(None);
        }
        let (year, month, day, hour, minute, _) = epoch_to_local_components(start_epoch, local_tz);

        let kind = infer_type(&summary);
        Ok(Some(CalendarEvent {
            title: summary,
            date_iso: format!("{year:04}-{month:02}-{day:02}"),
            day_label: format!(
                "{} {:02} {}",
                text_slide::date_utils::weekday_abbrev(year, month, day),
                day,
                month_name(month)
            ),
            time_label: if value_is_date {
                "All day".to_string()
            } else {
                format!("{hour:02}:{minute:02}")
            },
            kind,
            attendees: self.attendees.max(1),
            start_epoch,
        }))
    }
}

fn unfold_ics(input: &str) -> String {
    let mut output = String::new();
    let mut current = String::new();
    for raw_line in input.replace("\r\n", "\n").split('\n') {
        if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
            current.push_str(raw_line.trim_start());
            continue;
        }
        if !current.is_empty() {
            output.push_str(&current);
            output.push('\n');
        }
        current = raw_line.to_string();
    }
    if !current.is_empty() {
        output.push_str(&current);
    }
    output
}

fn parse_ics_datetime(
    value: &str,
    tzid: Option<&str>,
    value_is_date: bool,
    local_tz: &str,
) -> Result<u64, String> {
    if value_is_date || value.len() == 8 {
        let (year, month, day) = parse_date(value)?;
        return local_datetime_to_epoch(year, month, day, 0, 0, 0, tzid.unwrap_or(local_tz));
    }

    let is_utc = value.ends_with('Z');
    let core = value.trim_end_matches('Z');
    if core.len() < 15 {
        return Err(format!("unsupported DTSTART value '{value}'"));
    }
    let year = core[0..4]
        .parse()
        .map_err(|_| format!("bad year in '{value}'"))?;
    let month = core[4..6]
        .parse()
        .map_err(|_| format!("bad month in '{value}'"))?;
    let day = core[6..8]
        .parse()
        .map_err(|_| format!("bad day in '{value}'"))?;
    let hour = core[9..11]
        .parse()
        .map_err(|_| format!("bad hour in '{value}'"))?;
    let minute = core[11..13]
        .parse()
        .map_err(|_| format!("bad minute in '{value}'"))?;
    let second = core[13..15]
        .parse()
        .map_err(|_| format!("bad second in '{value}'"))?;

    if is_utc {
        Ok(epoch_from_utc(year, month, day, hour, minute, second))
    } else {
        local_datetime_to_epoch(
            year,
            month,
            day,
            hour,
            minute,
            second,
            tzid.unwrap_or(local_tz),
        )
    }
}

fn parse_date(value: &str) -> Result<(i32, u8, u8), String> {
    if value.len() < 8 {
        return Err(format!("unsupported DATE value '{value}'"));
    }
    let year = value[0..4]
        .parse()
        .map_err(|_| format!("bad year in '{value}'"))?;
    let month = value[4..6]
        .parse()
        .map_err(|_| format!("bad month in '{value}'"))?;
    let day = value[6..8]
        .parse()
        .map_err(|_| format!("bad day in '{value}'"))?;
    Ok((year, month, day))
}

fn local_datetime_to_epoch(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    timezone: &str,
) -> Result<u64, String> {
    let offset = timezone_offset_seconds(timezone, year, month, day, hour)?;
    let local = days_from_civil(year, month, day) * 86_400
        + i64::from(hour) * 3_600
        + i64::from(minute) * 60
        + i64::from(second);
    Ok((local - i64::from(offset)) as u64)
}

fn epoch_from_utc(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> u64 {
    (days_from_civil(year, month, day) * 86_400
        + i64::from(hour) * 3_600
        + i64::from(minute) * 60
        + i64::from(second)) as u64
}

fn epoch_to_local_components(epoch_secs: u64, timezone: &str) -> (i32, u8, u8, u8, u8, u8) {
    let mut offset = if timezone == "Australia/Melbourne" {
        10 * 3_600
    } else {
        0
    };
    for _ in 0..2 {
        let shifted = (epoch_secs as i64 + i64::from(offset)) as u64;
        let (year, month, day, hour, minute, second) = utc_components(shifted);
        offset = timezone_offset_seconds(timezone, year, month, day, hour).unwrap_or(0);
        if timezone == "UTC" {
            return (year, month, day, hour, minute, second);
        }
    }
    utc_components((epoch_secs as i64 + i64::from(offset)) as u64)
}

fn utc_components(epoch_secs: u64) -> (i32, u8, u8, u8, u8, u8) {
    let days = (epoch_secs / 86_400) as i64;
    let seconds_today = epoch_secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    (
        year,
        month,
        day,
        (seconds_today / 3_600) as u8,
        ((seconds_today / 60) % 60) as u8,
        (seconds_today % 60) as u8,
    )
}

fn timezone_offset_seconds(
    timezone: &str,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
) -> Result<i32, String> {
    match timezone {
        "UTC" => Ok(0),
        "Australia/Melbourne" => {
            let first_sunday_october = first_sunday(year, 10);
            let first_sunday_april = first_sunday(year, 4);
            let is_dst = if !(4..10).contains(&month) {
                true
            } else if (5..=9).contains(&month) {
                false
            } else if month == 10 {
                day > first_sunday_october || (day == first_sunday_october && hour >= 2)
            } else {
                day < first_sunday_april || (day == first_sunday_april && hour < 3)
            };
            Ok(if is_dst { 11 * 3_600 } else { 10 * 3_600 })
        }
        other => Err(format!("unsupported timezone '{other}'")),
    }
}

fn first_sunday(year: i32, month: u8) -> u8 {
    (1..=7)
        .find(|day| text_slide::date_utils::weekday_abbrev(year, month, *day) == "Sun")
        .unwrap_or(1)
}

fn month_name(month: u8) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = i64::from(month);
    let day = i64::from(day);
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u8, day as u8)
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
    if let Some(payload) = text_slide::channel_runtime::poll_json::<CalendarPayload>(&mut state.response_buf) {
        state.payload = Some(payload);
    }
    state.refresh();
    1
}

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use super::{CalendarPayload, build_overlay};
    use text_slide::channel_runtime;
    use crate::text_slide;

    pub struct RuntimeState {
        pub payload: Option<CalendarPayload>,
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

    fn make_ics(events: &[(&str, &str, &str, usize)]) -> String {
        let mut out = String::from("BEGIN:VCALENDAR\r\nVERSION:2.0\r\n");
        for (start, end, summary, attendees) in events {
            out.push_str("BEGIN:VEVENT\r\n");
            out.push_str("UID:test@example.com\r\n");
            out.push_str(&format!("DTSTART:{start}\r\n"));
            out.push_str(&format!("DTEND:{end}\r\n"));
            out.push_str(&format!("SUMMARY:{summary}\r\n"));
            for idx in 0..*attendees {
                out.push_str(&format!("ATTENDEE:mailto:user{idx}@example.com\r\n"));
            }
            out.push_str("END:VEVENT\r\n");
        }
        out.push_str("END:VCALENDAR\r\n");
        out
    }

    #[test]
    fn spec_valid() {
        calendar_slide_spec().validate().unwrap();
    }

    #[test]
    fn infer_type_matches_dashboard_rules() {
        assert_eq!(infer_type("Daily Standup"), "standup");
        assert_eq!(infer_type("Sprint Planning"), "planning");
        assert_eq!(infer_type("Candidate Interview"), "interview");
        assert_eq!(infer_type("Something Else"), "review");
    }

    #[test]
    fn parse_ics_filters_and_sorts_upcoming_events() {
        let now = epoch_from_utc(2026, 3, 19, 0, 0, 0);
        let ics = make_ics(&[
            ("20260318T090000Z", "20260318T100000Z", "Old Meeting", 0),
            ("20260320T090000Z", "20260320T100000Z", "Daily Standup", 2),
            ("20260430T090000Z", "20260430T100000Z", "Far Away", 0),
            ("20260319T090000Z", "20260319T100000Z", "Design Review", 0),
        ]);
        let events = parse_ics(&ics, now, 7, "UTC").expect("parse ICS");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].title, "Design Review");
        assert_eq!(events[1].kind, "standup");
        assert_eq!(events[0].attendees, 1);
        assert_eq!(events[1].attendees, 2);
    }

    #[test]
    fn parse_ics_handles_melbourne_tz_and_all_day_events() {
        let now = epoch_from_utc(2026, 3, 19, 0, 0, 0);
        let ics = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:a\r\nDTSTART;TZID=Australia/Melbourne:20260320T090000\r\nDTEND;TZID=Australia/Melbourne:20260320T100000\r\nSUMMARY:Sprint Planning\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:b\r\nDTSTART;VALUE=DATE:20260321\r\nDTEND;VALUE=DATE:20260322\r\nSUMMARY:Offsite\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let events = parse_ics(ics, now, 7, "Australia/Melbourne").expect("parse ICS");
        assert_eq!(events[0].time_label, "09:00");
        assert_eq!(events[0].kind, "planning");
        assert_eq!(events[1].time_label, "All day");
    }
}
