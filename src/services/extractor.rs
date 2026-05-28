use chrono::{DateTime, Utc};
use scraper::{ElementRef, Html, Selector};
use url::Url;

use crate::{
    domain::{
        extracted_item::{ExtractedItem, RssItemData},
        feed::Feed,
        selector::SelectorType,
    },
    error::{AppError, AppResult},
};

pub fn extract_items(feed: &Feed, html: &str) -> AppResult<Vec<ExtractedItem>> {
    if SelectorType::parse(&feed.selector_type) != Some(SelectorType::Css) {
        return Err(AppError::BadRequest(
            "only css selectors are implemented".to_string(),
        ));
    }
    let document = Html::parse_document(html);
    let item_selector = parse_selector(&feed.item_selector)?;
    let title_selector = parse_selector(&feed.title_selector)?;
    let link_selector = optional_selector(feed.link_selector.as_deref())?;
    let date_selector = optional_selector(feed.date_selector.as_deref())?;
    let content_selector = optional_selector(feed.content_selector.as_deref())?;
    let base = Url::parse(&feed.site_url)
        .map_err(|_| AppError::BadRequest("invalid feed site_url".to_string()))?;

    let mut items = Vec::new();
    for item in document.select(&item_selector) {
        let title = item
            .select(&title_selector)
            .next()
            .map(text_content)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| text_content(item));
        if title.is_empty() {
            continue;
        }
        let Some(link) = extract_link(item, link_selector.as_ref(), &base)? else {
            continue;
        };
        let date_text = date_selector
            .as_ref()
            .and_then(|selector| item.select(selector).next())
            .map(text_content)
            .filter(|v| !v.is_empty());
        let content = content_selector
            .as_ref()
            .and_then(|selector| item.select(selector).next())
            .map(text_content)
            .filter(|v| !v.is_empty());
        items.push(ExtractedItem {
            title,
            link,
            date_text,
            content,
        });
    }
    Ok(items)
}

pub fn resolve_url(base: &str, href: &str) -> AppResult<String> {
    let base =
        Url::parse(base).map_err(|_| AppError::BadRequest("invalid base URL".to_string()))?;
    Ok(base
        .join(href)
        .map_err(|_| AppError::BadRequest("invalid link URL".to_string()))?
        .to_string())
}

pub fn apply_date_fallback(
    item: ExtractedItem,
    existing_first_seen: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> RssItemData {
    let parsed = item.date_text.as_deref().and_then(parse_date);
    RssItemData {
        title: item.title,
        link: item.link,
        pub_date: parsed.or(existing_first_seen).unwrap_or(now),
        description: item.content,
    }
}

fn parse_selector(raw: &str) -> AppResult<Selector> {
    Selector::parse(raw).map_err(|_| AppError::BadRequest(format!("invalid CSS selector: {raw}")))
}

fn optional_selector(raw: Option<&str>) -> AppResult<Option<Selector>> {
    raw.filter(|v| !v.trim().is_empty())
        .map(parse_selector)
        .transpose()
}

fn text_content(element: ElementRef<'_>) -> String {
    element
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_link(
    item: ElementRef<'_>,
    link_selector: Option<&Selector>,
    base: &Url,
) -> AppResult<Option<String>> {
    let href = if let Some(selector) = link_selector {
        item.select(selector)
            .next()
            .and_then(|el| el.value().attr("href"))
    } else if item.value().name() == "a" {
        item.value().attr("href")
    } else {
        let a_selector = Selector::parse("a[href]").expect("static selector is valid");
        item.select(&a_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
    };

    href.map(|href| resolve_url(base.as_str(), href))
        .transpose()
}

fn parse_date(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .or_else(|_| DateTime::parse_from_rfc2822(raw))
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn feed() -> Feed {
        Feed {
            id: 1,
            name: "Example".to_string(),
            site_url: "https://example.com/blog/".to_string(),
            feed_description: None,
            selector_type: "css".to_string(),
            item_selector: "article".to_string(),
            title_selector: "h2".to_string(),
            link_selector: None,
            date_selector: Some("time".to_string()),
            content_selector: Some(".summary".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn resolves_relative_urls() {
        assert_eq!(
            resolve_url("https://example.com/blog/", "/post/1").unwrap(),
            "https://example.com/post/1"
        );
        assert_eq!(
            resolve_url("https://example.com/blog/", "post/1").unwrap(),
            "https://example.com/blog/post/1"
        );
    }

    #[test]
    fn extracts_items_from_html() {
        let html = r#"
          <article><h2>First</h2><a href="/one">Read</a><time>2024-01-01T00:00:00Z</time><p class="summary">Hello</p></article>
          <article><h2>Second</h2><a href="https://example.net/two">Read</a></article>
        "#;
        let items = extract_items(&feed(), html).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "First");
        assert_eq!(items[0].link, "https://example.com/one");
        assert_eq!(items[0].content.as_deref(), Some("Hello"));
        assert_eq!(items[1].link, "https://example.net/two");
    }

    #[test]
    fn falls_back_to_existing_or_now_for_missing_date() {
        let item = ExtractedItem {
            title: "T".to_string(),
            link: "https://example.com/t".to_string(),
            date_text: None,
            content: None,
        };
        let existing = Utc.with_ymd_and_hms(2023, 1, 2, 3, 4, 5).unwrap();
        let now = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
        assert_eq!(
            apply_date_fallback(item.clone(), Some(existing), now).pub_date,
            existing
        );
        assert_eq!(apply_date_fallback(item, None, now).pub_date, now);
    }
}
