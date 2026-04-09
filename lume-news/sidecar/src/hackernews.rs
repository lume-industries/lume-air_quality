use news_slide::Headline;
use serde::Deserialize;
use vzglyd_sidecar::{Error, https_get_text};

const HOST: &str = "hacker-news.firebaseio.com";
const TOP_PATH: &str = "/v0/topstories.json";

#[derive(Deserialize)]
struct StoryItem {
    #[serde(default)]
    title: String,
    #[serde(default)]
    time: i64,
}

pub fn fetch_top(limit: usize) -> Result<Vec<Headline>, Error> {
    fetch_top_with(limit, https_get_text)
}

pub(crate) fn fetch_top_with<F>(limit: usize, get_text: F) -> Result<Vec<Headline>, Error>
where
    F: Fn(&str, &str) -> Result<String, Error>,
{
    let ids = parse_top_ids(&get_text(HOST, TOP_PATH)?)?;
    let mut headlines = Vec::new();

    for story_id in ids.into_iter().take(limit.saturating_mul(3).max(limit)) {
        let path = format!("/v0/item/{story_id}.json");
        match parse_item(&get_text(HOST, &path)?)? {
            Some(headline) => {
                headlines.push(headline);
                if headlines.len() >= limit {
                    break;
                }
            }
            None => continue,
        }
    }

    Ok(headlines)
}

fn parse_top_ids(body: &str) -> Result<Vec<u64>, Error> {
    serde_json::from_str(body).map_err(|error| Error::Io(format!("invalid HackerNews ID list: {error}")))
}

fn parse_item(body: &str) -> Result<Option<Headline>, Error> {
    let item: StoryItem = serde_json::from_str(body)
        .map_err(|error| Error::Io(format!("invalid HackerNews item payload: {error}")))?;
    if item.title.trim().is_empty() || item.time <= 0 {
        return Ok(None);
    }

    Ok(Some(Headline {
        title: item.title.trim().to_string(),
        source: "HackerNews".to_string(),
        category: "tech".to_string(),
        timestamp: item.time,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_top_uses_listing_then_story_payloads() {
        let headlines = fetch_top_with(2, |host, path| {
            assert_eq!(host, HOST);
            match path {
                TOP_PATH => Ok("[101,102,103]".to_string()),
                "/v0/item/101.json" => {
                    Ok("{\"title\":\"Story One\",\"time\":1742371200}".to_string())
                }
                "/v0/item/102.json" => {
                    Ok("{\"title\":\"Story Two\",\"time\":1742372200}".to_string())
                }
                _ => unreachable!("unexpected path {path}"),
            }
        })
        .unwrap();

        assert_eq!(headlines.len(), 2);
        assert_eq!(headlines[0].source, "HackerNews");
        assert_eq!(headlines[1].title, "Story Two");
    }

    #[test]
    fn parse_item_skips_missing_titles() {
        let headline = parse_item("{\"time\":1742371200}").unwrap();
        assert!(headline.is_none());
    }
}
