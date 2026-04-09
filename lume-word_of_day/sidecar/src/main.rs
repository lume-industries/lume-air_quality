use vzglyd_sidecar::{Error, https_get_text, poll_loop};

const MW_HOST: &str = "www.merriam-webster.com";
const MW_PATH: &str = "/wotd/feed/rss2";
const DICT_HOST: &str = "api.dictionaryapi.dev";

fn fetch() -> Result<Vec<u8>, Error> {
    let now_secs = now_unix_secs();
    let rss_body = https_get_text(MW_HOST, MW_PATH)?;
    let (word, raw_desc) = word_of_day_slide::parse_rss_item(&rss_body).map_err(Error::Io)?;
    let dict_path = format!("/api/v2/entries/en/{}", encode_path_segment(&word));
    let payload = match https_get_text(DICT_HOST, &dict_path) {
        Ok(dict_body) => word_of_day_slide::parse_dictionary_payload(
            &word,
            &raw_desc,
            &dict_body,
            now_secs,
        )
        .map_err(Error::Io)?,
        Err(_) => word_of_day_slide::fallback_payload(&word, &raw_desc, now_secs),
    };
    serde_json::to_vec(&payload).map_err(|error| Error::Io(error.to_string()))
}

fn encode_path_segment(input: &str) -> String {
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
    poll_loop(24 * 60 * 60, fetch);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("word-of-day-sidecar is intended for wasm32-wasip1");
}
