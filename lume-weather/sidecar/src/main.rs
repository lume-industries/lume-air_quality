use vzglyd_sidecar::{Error, https_get_text, poll_loop};

const SEARCH_PATH: &str = "/v1/locations?search=Kyneton";

fn fetch() -> Result<Vec<u8>, Error> {
    let search_body = https_get_text("api.weather.bom.gov.au", SEARCH_PATH)?;
    let (geohash, location) =
        weather_slide::parse_search_result(&search_body).map_err(Error::Io)?;
    let forecast_path = format!("/v1/locations/{geohash}/forecasts/daily");
    let forecast_body = https_get_text("api.weather.bom.gov.au", &forecast_path)?;
    let payload = weather_slide::parse_forecast_payload(location, &forecast_body, now_unix_secs())
        .map_err(Error::Io)?;
    serde_json::to_vec(&payload).map_err(|error| Error::Io(error.to_string()))
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
fn main() {
    poll_loop(30 * 60, fetch);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("weather-sidecar is intended for wasm32-wasip1");
}
