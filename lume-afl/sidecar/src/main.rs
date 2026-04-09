use vzglyd_sidecar::{Error, https_get_text, poll_loop};

fn fetch() -> Result<Vec<u8>, Error> {
    let now_secs = now_unix_secs();
    let year = current_year(now_secs);
    let ladder_body = https_get_text("api.squiggle.com.au", "/?q=standings")?;
    let games_path = format!("/?q=games&year={year}");
    let fixtures_body = https_get_text("api.squiggle.com.au", &games_path)?;
    let payload = afl_slide::compose_payload(&ladder_body, &fixtures_body, now_secs)
        .map_err(Error::Io)?;
    serde_json::to_vec(&payload).map_err(|error| Error::Io(error.to_string()))
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn current_year(epoch_secs: u64) -> i32 {
    let days = (epoch_secs / 86_400) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    (year + if mp >= 10 { 1 } else { 0 }) as i32
}

#[cfg(target_arch = "wasm32")]
fn main() {
    poll_loop(60 * 60, fetch);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("afl-sidecar is intended for wasm32-wasip1");
}
