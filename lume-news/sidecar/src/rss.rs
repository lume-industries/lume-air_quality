use news_slide::Headline;
use quick_xml::Reader;
use quick_xml::events::Event;
use vzglyd_sidecar::{Error, https_get_conditional};

#[derive(Clone, Debug)]
pub struct FeedState {
    pub name: &'static str,
    category: &'static str,
    host: &'static str,
    path: &'static str,
    limit: usize,
    etag: Option<String>,
    last_modified: Option<String>,
    cached_headlines: Vec<Headline>,
}

#[derive(Default)]
struct EntryBuilder {
    title: String,
    date: String,
    link: String,
}

#[derive(Clone, Copy)]
enum Field {
    Title,
    Date,
    Link,
}

impl FeedState {
    fn new(
        name: &'static str,
        category: &'static str,
        host: &'static str,
        path: &'static str,
        limit: usize,
    ) -> Self {
        Self {
            name,
            category,
            host,
            path,
            limit,
            etag: None,
            last_modified: None,
            cached_headlines: Vec::new(),
        }
    }
}

pub fn default_feeds() -> Vec<FeedState> {
    vec![
        FeedState::new("lobste.rs", "tech", "lobste.rs", "/rss", 8),
        FeedState::new(
            "Ars Technica",
            "tech",
            "feeds.arstechnica.com",
            "/arstechnica/index",
            8,
        ),
    ]
}

pub fn fetch_feed(state: &mut FeedState) -> Result<Vec<Headline>, Error> {
    fetch_feed_with(state, https_get_conditional)
}

pub(crate) fn fetch_feed_with<F>(state: &mut FeedState, get: F) -> Result<Vec<Headline>, Error>
where
    F: Fn(
        &str,
        &str,
        Option<&str>,
        Option<&str>,
    ) -> Result<(Vec<u8>, Option<String>, Option<String>), Error>,
{
    let (body, etag, last_modified) = get(
        state.host,
        state.path,
        state.etag.as_deref(),
        state.last_modified.as_deref(),
    )?;
    state.etag = etag;
    state.last_modified = last_modified;

    if body.is_empty() {
        return Ok(state.cached_headlines.clone());
    }

    let xml = String::from_utf8(body)
        .map_err(|error| Error::Io(format!("RSS body was not valid UTF-8: {error}")))?;
    let headlines = parse_feed(&xml, state.name, state.category)?
        .into_iter()
        .take(state.limit)
        .collect::<Vec<_>>();
    state.cached_headlines = headlines.clone();
    Ok(headlines)
}

fn parse_feed(xml: &str, source_name: &str, category: &str) -> Result<Vec<Headline>, Error> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut in_entry = false;
    let mut current = EntryBuilder::default();
    let mut capture = None;
    let mut headlines = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => match local_name(event.name().as_ref()) {
                "item" | "entry" => {
                    in_entry = true;
                    current = EntryBuilder::default();
                    capture = None;
                }
                "title" if in_entry => capture = Some(Field::Title),
                "pubDate" | "updated" | "published" if in_entry => capture = Some(Field::Date),
                "link" if in_entry => {
                    if let Some(href) = href_attr(&event) {
                        current.link = href;
                    } else {
                        capture = Some(Field::Link);
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(event)) => {
                if in_entry && local_name(event.name().as_ref()) == "link" {
                    if let Some(href) = href_attr(&event) {
                        current.link = href;
                    }
                }
            }
            Ok(Event::Text(text)) => {
                if let Some(field) = capture {
                    append_text(&mut current, field, text.as_ref());
                }
            }
            Ok(Event::CData(text)) => {
                if let Some(field) = capture {
                    append_text(&mut current, field, text.as_ref());
                }
            }
            Ok(Event::End(event)) => match local_name(event.name().as_ref()) {
                "title" | "pubDate" | "updated" | "published" | "link" => capture = None,
                "item" | "entry" if in_entry => {
                    if let Some(headline) = build_headline(&current, source_name, category) {
                        headlines.push(headline);
                    }
                    in_entry = false;
                    capture = None;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(error) => {
                return Err(Error::Io(format!("invalid RSS/Atom XML: {error}")));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(headlines)
}

fn append_text(current: &mut EntryBuilder, field: Field, bytes: &[u8]) {
    let decoded = decode_text(bytes);
    match field {
        Field::Title => current.title.push_str(&decoded),
        Field::Date => current.date.push_str(&decoded),
        Field::Link if current.link.is_empty() => current.link.push_str(&decoded),
        Field::Link => {}
    }
}

fn build_headline(current: &EntryBuilder, source_name: &str, category: &str) -> Option<Headline> {
    let title = current.title.trim();
    if title.is_empty() {
        return None;
    }

    Some(Headline {
        title: title.to_string(),
        source: source_name.to_string(),
        category: category.to_string(),
        timestamp: parse_timestamp(&current.date).unwrap_or(0),
    })
}

fn href_attr(event: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    for attr in event.attributes().flatten() {
        if local_name(attr.key.as_ref()) == "href" {
            return Some(decode_text(attr.value.as_ref()));
        }
    }
    None
}

fn decode_text(bytes: &[u8]) -> String {
    let raw = String::from_utf8_lossy(bytes);
    quick_xml::escape::unescape(&raw)
        .map(|text| text.into_owned())
        .unwrap_or_else(|_| raw.into_owned())
}

fn local_name(name: &[u8]) -> &str {
    let raw = std::str::from_utf8(name).unwrap_or_default();
    raw.rsplit(':').next().unwrap_or(raw)
}

fn parse_timestamp(raw: &str) -> Option<i64> {
    parse_rfc2822(raw).or_else(|| parse_iso8601(raw))
}

fn parse_rfc2822(raw: &str) -> Option<i64> {
    let cleaned = raw.trim().replace(',', "");
    let mut parts = cleaned.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 5 {
        return None;
    }
    if parts[0].chars().all(|ch| ch.is_ascii_alphabetic()) {
        parts.remove(0);
    }
    if parts.len() < 5 {
        return None;
    }

    let day = parts[0].parse::<u8>().ok()?;
    let month = parse_month(parts[1])?;
    let year = parts[2].parse::<i32>().ok()?;
    let (hour, minute, second) = parse_hms(parts[3])?;
    let offset = parse_offset(parts[4])?;
    Some(epoch_from_utc(year, month, day, hour, minute, second) - i64::from(offset))
}

fn parse_iso8601(raw: &str) -> Option<i64> {
    let text = raw.trim();
    let (date, rest) = text.split_once('T')?;
    let (year, month, day) = parse_ymd(date)?;

    let (time_part, offset) = if let Some(core) = rest.strip_suffix('Z') {
        (core, 0)
    } else if let Some(pos) = rest.rfind(['+', '-']) {
        let sign = if rest.as_bytes().get(pos) == Some(&b'-') {
            -1
        } else {
            1
        };
        let (time, offset_text) = rest.split_at(pos);
        (time, sign * parse_hm_offset(offset_text.get(1..)?)?)
    } else {
        (rest, 0)
    };

    let (hour, minute, second) = parse_hms(time_part)?;
    Some(epoch_from_utc(year, month, day, hour, minute, second) - i64::from(offset))
}

fn parse_ymd(raw: &str) -> Option<(i32, u8, u8)> {
    let mut parts = raw.split('-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;
    Some((year, month, day))
}

fn parse_hms(raw: &str) -> Option<(u8, u8, u8)> {
    let raw = raw.split('.').next().unwrap_or(raw);
    let mut parts = raw.split(':');
    let hour = parts.next()?.parse().ok()?;
    let minute = parts.next()?.parse().ok()?;
    let second = parts.next().unwrap_or("0").parse().ok()?;
    Some((hour, minute, second))
}

fn parse_offset(raw: &str) -> Option<i32> {
    match raw {
        "UTC" | "GMT" => Some(0),
        _ => {
            let sign = if raw.starts_with('-') { -1 } else { 1 };
            let digits = raw.trim_start_matches(['+', '-']);
            if digits.len() != 4 {
                return None;
            }
            let hours = digits[0..2].parse::<i32>().ok()?;
            let minutes = digits[2..4].parse::<i32>().ok()?;
            Some(sign * (hours * 3_600 + minutes * 60))
        }
    }
}

fn parse_hm_offset(raw: &str) -> Option<i32> {
    let mut parts = raw.split(':');
    let hours = parts.next()?.parse::<i32>().ok()?;
    let minutes = parts.next()?.parse::<i32>().ok()?;
    Some(hours * 3_600 + minutes * 60)
}

fn parse_month(raw: &str) -> Option<u8> {
    match raw {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

fn epoch_from_utc(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> i64 {
    days_from_civil(year, month, day) * 86_400
        + i64::from(hour) * 3_600
        + i64::from(minute) * 60
        + i64::from(second)
}

fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let mut year = i64::from(year);
    let month = i64::from(month);
    let day = i64::from(day);
    year -= if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSS_XML: &str = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <item>
      <title>Big Tech News</title>
      <link>https://example.com/1</link>
      <pubDate>Thu, 19 Mar 2026 10:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>"#;

    const ATOM_XML: &str = r#"<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <title>Atom Story One</title>
    <link href="https://example.com/atom/1"/>
    <updated>2026-03-19T12:00:00Z</updated>
  </entry>
</feed>"#;

    #[test]
    fn parse_feed_handles_rss_and_atom() {
        let rss_items = parse_feed(RSS_XML, "lobste.rs", "tech").unwrap();
        let atom_items = parse_feed(ATOM_XML, "Ars Technica", "tech").unwrap();

        assert_eq!(rss_items.len(), 1);
        assert_eq!(rss_items[0].title, "Big Tech News");
        assert_eq!(atom_items.len(), 1);
        assert_eq!(atom_items[0].title, "Atom Story One");
        assert!(atom_items[0].timestamp > rss_items[0].timestamp);
    }

    #[test]
    fn parse_timestamp_accepts_rfc2822_and_iso8601() {
        assert_eq!(
            parse_timestamp("Thu, 19 Mar 2026 10:30:00 +0000"),
            Some(1_773_916_200)
        );
        assert_eq!(
            parse_timestamp("2026-03-19T14:45:00+11:00"),
            Some(1_773_891_900)
        );
    }

    #[test]
    fn conditional_fetch_reuses_cached_items_on_not_modified() {
        let mut feed = FeedState::new("lobste.rs", "tech", "lobste.rs", "/rss", 8);

        let first = fetch_feed_with(&mut feed, |host, path, etag, last_modified| {
            assert_eq!(host, "lobste.rs");
            assert_eq!(path, "/rss");
            assert!(etag.is_none());
            assert!(last_modified.is_none());
            Ok((
                RSS_XML.as_bytes().to_vec(),
                Some("\"abc123\"".to_string()),
                Some("Thu, 19 Mar 2026 10:00:00 +0000".to_string()),
            ))
        })
        .unwrap();

        let second = fetch_feed_with(&mut feed, |_host, _path, etag, last_modified| {
            assert_eq!(etag, Some("\"abc123\""));
            assert_eq!(last_modified, Some("Thu, 19 Mar 2026 10:00:00 +0000"));
            Ok((Vec::new(), etag.map(ToOwned::to_owned), last_modified.map(ToOwned::to_owned)))
        })
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(first[0].source, "lobste.rs");
    }
}
