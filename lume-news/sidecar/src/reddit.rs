use news_slide::Headline;
use serde::Deserialize;
use vzglyd_sidecar::{Error, https_get_text};

const HOST: &str = "www.reddit.com";

#[derive(Deserialize)]
struct Listing {
    data: ListingData,
}

#[derive(Deserialize)]
struct ListingData {
    children: Vec<ListingChild>,
}

#[derive(Deserialize)]
struct ListingChild {
    data: PostData,
}

#[derive(Deserialize)]
struct PostData {
    #[serde(default)]
    title: String,
    #[serde(default)]
    created_utc: f64,
}

pub fn fetch_new(
    subreddit: &str,
    limit: usize,
    source_label: &str,
    category: &str,
) -> Result<Vec<Headline>, Error> {
    fetch_new_with(subreddit, limit, source_label, category, https_get_text)
}

pub(crate) fn fetch_new_with<F>(
    subreddit: &str,
    limit: usize,
    source_label: &str,
    category: &str,
    get_text: F,
) -> Result<Vec<Headline>, Error>
where
    F: Fn(&str, &str) -> Result<String, Error>,
{
    let path = format!("/r/{subreddit}/new.json?limit={limit}&sort=new");
    parse_listing(&get_text(HOST, &path)?, source_label, category)
}

fn parse_listing(body: &str, source_label: &str, category: &str) -> Result<Vec<Headline>, Error> {
    let listing: Listing = serde_json::from_str(body)
        .map_err(|error| Error::Io(format!("invalid Reddit listing payload: {error}")))?;

    let headlines = listing
        .data
        .children
        .into_iter()
        .filter_map(|child| {
            let title = child.data.title.trim().to_string();
            if title.is_empty() {
                return None;
            }
            Some(Headline {
                title,
                source: source_label.to_string(),
                category: category.to_string(),
                timestamp: child.data.created_utc as i64,
            })
        })
        .collect();

    Ok(headlines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_new_parses_listing_payload() {
        let body = r#"{
            "data": {
                "children": [
                    {"data": {"title": "World story", "created_utc": 1742371200.0}},
                    {"data": {"title": "Another story", "created_utc": 1742370000.0}}
                ]
            }
        }"#;

        let headlines = fetch_new_with("worldnews", 2, "r/worldnews", "world", |host, path| {
            assert_eq!(host, HOST);
            assert_eq!(path, "/r/worldnews/new.json?limit=2&sort=new");
            Ok(body.to_string())
        })
        .unwrap();

        assert_eq!(headlines.len(), 2);
        assert_eq!(headlines[0].category, "world");
        assert_eq!(headlines[1].source, "r/worldnews");
    }

    #[test]
    fn parse_listing_skips_blank_titles() {
        let body = r#"{
            "data": {
                "children": [
                    {"data": {"title": "", "created_utc": 1742371200.0}}
                ]
            }
        }"#;

        let headlines = parse_listing(body, "r/worldnews", "world").unwrap();
        assert!(headlines.is_empty());
    }
}
