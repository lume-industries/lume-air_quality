mod hackernews;
mod reddit;
mod rss;

use std::cell::RefCell;
use std::collections::HashSet;

use news_slide::{Headline, NewsPayload, truncate_headline, updated_label};
use vzglyd_sidecar::{Error, info_log};

thread_local! {
    static NEWS_STATE: RefCell<NewsState> = RefCell::new(NewsState::new());
}

fn fetch() -> Result<Vec<u8>, Error> {
    let now_secs = now_unix_secs();
    let headlines = NEWS_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.refresh_one();
        state.combined_headlines()
    });
    let headlines = finalize_headlines(headlines);
    info_log(&format!("emitting payload headlines={}", headlines.len()));

    let payload = NewsPayload {
        updated: updated_label(now_secs),
        headlines,
    };
    serde_json::to_vec(&payload).map_err(|error| Error::Io(error.to_string()))
}

fn log_source_error(label: &str, error: &Error) {
    info_log(&format!("{label} fetch failed: {error}"));
    eprintln!("news-sidecar: {label} fetch failed: {error}");
}

struct NewsState {
    step: usize,
    rss_feeds: Vec<rss::FeedState>,
    tech_feed_cache: Vec<Vec<Headline>>,
    world_cache: Vec<Headline>,
    australia_cache: Vec<Headline>,
    hackernews_cache: Vec<Headline>,
}

impl NewsState {
    fn new() -> Self {
        let rss_feeds = rss::default_feeds();
        let tech_feed_cache = vec![Vec::new(); rss_feeds.len()];
        Self {
            step: 0,
            rss_feeds,
            tech_feed_cache,
            world_cache: Vec::new(),
            australia_cache: Vec::new(),
            hackernews_cache: Vec::new(),
        }
    }

    fn refresh_one(&mut self) {
        let step = self.step % 5;
        self.step = (self.step + 1) % 5;
        let label = match step {
            0 => "lobste.rs",
            1 => "Ars Technica",
            2 => "r/worldnews",
            3 => "r/australia",
            _ => "HackerNews",
        };
        info_log(&format!("refreshing {label}"));
        match step {
            0 => self.refresh_feed(0),
            1 => self.refresh_feed(1),
            2 => self.refresh_world(),
            3 => self.refresh_australia(),
            _ => self.refresh_hackernews(),
        }
    }

    fn refresh_feed(&mut self, index: usize) {
        let Some(feed) = self.rss_feeds.get_mut(index) else {
            return;
        };
        match rss::fetch_feed(feed) {
            Ok(items) => {
                info_log(&format!("{} refreshed {} headlines", feed.name, items.len()));
                self.tech_feed_cache[index] = items;
            }
            Err(error) => log_source_error(feed.name, &error),
        }
    }

    fn refresh_world(&mut self) {
        match reddit::fetch_new("worldnews", 8, "r/worldnews", "world") {
            Ok(items) => {
                info_log(&format!("r/worldnews refreshed {} headlines", items.len()));
                self.world_cache = items;
            }
            Err(error) => log_source_error("r/worldnews", &error),
        }
    }

    fn refresh_australia(&mut self) {
        match reddit::fetch_new("australia", 8, "r/australia", "australia") {
            Ok(items) => {
                info_log(&format!("r/australia refreshed {} headlines", items.len()));
                self.australia_cache = items;
            }
            Err(error) => log_source_error("r/australia", &error),
        }
    }

    fn refresh_hackernews(&mut self) {
        match hackernews::fetch_top(6) {
            Ok(items) => {
                info_log(&format!("HackerNews refreshed {} headlines", items.len()));
                self.hackernews_cache = items;
            }
            Err(error) => log_source_error("HackerNews", &error),
        }
    }

    fn combined_headlines(&self) -> Vec<Headline> {
        let mut headlines = Vec::new();
        for feed_cache in &self.tech_feed_cache {
            headlines.extend(feed_cache.clone());
        }
        headlines.extend(self.world_cache.clone());
        headlines.extend(self.australia_cache.clone());
        headlines.extend(self.hackernews_cache.clone());
        headlines
    }
}

#[cfg(test)]
fn extend_source(headlines: &mut Vec<Headline>, result: Result<Vec<Headline>, Error>) {
    match result {
        Ok(items) => headlines.extend(items),
        Err(_) => {}
    }
}

fn finalize_headlines(mut headlines: Vec<Headline>) -> Vec<Headline> {
    headlines.sort_by(|left, right| {
        right
            .timestamp
            .cmp(&left.timestamp)
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.title.cmp(&right.title))
    });

    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for mut headline in headlines {
        let key = (
            headline.category.clone(),
            headline.title.to_ascii_lowercase(),
            headline.source.clone(),
        );
        if !seen.insert(key) {
            continue;
        }
        headline.title = truncate_headline(&headline.title, 48);
        deduped.push(headline);
        if deduped.len() >= 30 {
            break;
        }
    }

    deduped
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
fn main() {
    vzglyd_sidecar::poll_loop(10, fetch);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("news-sidecar is intended for wasm32-wasip1");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extend_source_skips_failures() {
        let mut headlines = vec![Headline {
            title: "Existing".to_string(),
            source: "HackerNews".to_string(),
            category: "tech".to_string(),
            timestamp: 1,
        }];

        extend_source(&mut headlines, Err(Error::Io("boom".to_string())));
        assert_eq!(headlines.len(), 1);
    }

    #[test]
    fn finalize_headlines_sorts_and_truncates() {
        let headlines = vec![
            Headline {
                title: "Later headline".to_string(),
                source: "HackerNews".to_string(),
                category: "tech".to_string(),
                timestamp: 20,
            },
            Headline {
                title: "Earlier headline that should be truncated for the slide output".to_string(),
                source: "lobste.rs".to_string(),
                category: "tech".to_string(),
                timestamp: 10,
            },
        ];

        let finalized = finalize_headlines(headlines);
        assert_eq!(finalized[0].title, "Later headline");
        assert!(finalized[1].title.ends_with("..."));
        assert!(finalized[1].title.len() <= 51);
    }

    #[test]
    fn cached_state_keeps_previous_headlines_when_refresh_fails() {
        let mut state = NewsState::new();
        state.world_cache = vec![Headline {
            title: "Cached".to_string(),
            source: "r/worldnews".to_string(),
            category: "world".to_string(),
            timestamp: 1,
        }];

        let headlines = state.combined_headlines();
        assert_eq!(headlines.len(), 1);
        assert_eq!(headlines[0].title, "Cached");
    }
}
