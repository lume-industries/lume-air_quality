use vzglyd_sidecar::{Error, env_var, https_get_text, poll_loop};

const LASTFM_HOST: &str = "ws.audioscrobbler.com";
const LASTFM_LIMIT: usize = 10;

fn fetch() -> Result<Vec<u8>, Error> {
    let api_key = env_var("LASTFM_API_KEY")
        .ok_or_else(|| Error::Io("LASTFM_API_KEY is not set in the sidecar environment".to_string()))?;
    let username = env_var("LASTFM_USERNAME")
        .ok_or_else(|| Error::Io("LASTFM_USERNAME is not set in the sidecar environment".to_string()))?;
    let path = format!(
        "/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json&limit={}",
        encode_query(&username),
        encode_query(&api_key),
        LASTFM_LIMIT
    );
    let body = https_get_text(LASTFM_HOST, &path)?;
    let payload =
        lastfm_slide::parse_recent_tracks(&username, &body, now_unix_secs()).map_err(Error::Io)?;
    serde_json::to_vec(&payload).map_err(|error| Error::Io(error.to_string()))
}

fn encode_query(input: &str) -> String {
    input.replace(' ', "%20")
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
fn main() {
    poll_loop(30, fetch);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("lastfm-sidecar is intended for wasm32-wasip1");
}
