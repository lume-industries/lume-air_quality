use vzglyd_sidecar::{Error, https_get_text, poll_loop};

fn fetch() -> Result<Vec<u8>, Error> {
    let now_secs = now_unix_secs();
    let (_year, month, day) = current_month_day(now_secs);
    let path = format!("/feed/v1/wikipedia/en/onthisday/events/{month:02}/{day:02}");
    let body = https_get_text("api.wikimedia.org", &path)?;
    let payload =
        on_this_day_slide::parse_events_payload(&body, format!("{day:02} {}", month_name(month)), now_secs)
            .map_err(Error::Io)?;
    serde_json::to_vec(&payload).map_err(|error| Error::Io(error.to_string()))
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn current_month_day(epoch_secs: u64) -> (i32, u8, u8) {
    let days = (epoch_secs / 86_400) as i64;
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

#[cfg(target_arch = "wasm32")]
fn main() {
    poll_loop(24 * 60 * 60, fetch);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("on-this-day-sidecar is intended for wasm32-wasip1");
}
