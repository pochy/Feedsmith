use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue},
    response::IntoResponse,
};
use chrono::Utc;

use crate::{
    app::AppState,
    domain::extracted_item::RssItemData,
    error::AppResult,
    services::{extractor, rss_builder},
};

pub async fn feed_rss(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let feed = state.feeds.get(id).await?;
    let html = state.fetcher.fetch_html(&feed.site_url).await?;
    let extracted = extractor::extract_items(&feed, &html)?;
    let mut items = Vec::new();
    for item in extracted {
        let existing = state.feeds.first_seen(feed.id, &item.link).await?;
        let data = extractor::apply_date_fallback(item, existing, Utc::now());
        let first_seen = state
            .feeds
            .upsert_seen(feed.id, &data.link, &data.title, data.pub_date)
            .await?;
        items.push(RssItemData {
            pub_date: first_seen,
            ..data
        });
    }
    let xml = rss_builder::build_rss(&feed, &items);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/rss+xml; charset=utf-8"),
    );
    Ok((headers, xml))
}
