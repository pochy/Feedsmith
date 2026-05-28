use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    Form, Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppState,
    domain::{extracted_item::ExtractedItem, feed::FeedForm},
    error::AppResult,
    services::{
        extractor,
        sanitizer::sanitize_preview_html,
        selector_suggester::{NoopSelectorSuggester, SelectorSuggester},
    },
};

#[derive(Template)]
#[template(path = "feeds/_preview.html")]
struct PreviewTemplate {
    items: Vec<ExtractedItem>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FetchPreviewHtmlForm {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct FetchPreviewHtmlResponse {
    pub html: String,
}

#[derive(Debug, Deserialize)]
pub struct SuggestSelectorsForm {
    pub url: String,
}

pub async fn preview_feed(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<FeedForm>,
) -> AppResult<Html<String>> {
    let mut feed = state.feeds.get(id).await?;
    let form = form.normalize();
    feed.site_url = form.site_url;
    feed.feed_description = form.feed_description;
    feed.selector_type = form.selector_type.unwrap_or_else(|| "css".to_string());
    feed.item_selector = form.item_selector;
    feed.title_selector = form.title_selector;
    feed.link_selector = form.link_selector;
    feed.date_selector = form.date_selector;
    feed.content_selector = form.content_selector;

    let rendered = match state.fetcher.fetch_html(&feed.site_url).await {
        Ok(html) => match extractor::extract_items(&feed, &html) {
            Ok(items) => PreviewTemplate {
                items: items.into_iter().take(20).collect(),
                error: None,
            }
            .render()?,
            Err(err) => PreviewTemplate {
                items: vec![],
                error: Some(err.to_string()),
            }
            .render()?,
        },
        Err(err) => PreviewTemplate {
            items: vec![],
            error: Some(err.to_string()),
        }
        .render()?,
    };
    Ok(Html(rendered))
}

pub async fn fetch_preview_html(
    State(state): State<AppState>,
    Form(form): Form<FetchPreviewHtmlForm>,
) -> AppResult<Json<FetchPreviewHtmlResponse>> {
    let html = state.fetcher.fetch_html(&form.url).await?;
    Ok(Json(FetchPreviewHtmlResponse {
        html: sanitize_preview_html(&html),
    }))
}

pub async fn suggest_selectors(
    State(state): State<AppState>,
    Form(form): Form<SuggestSelectorsForm>,
) -> AppResult<Response> {
    let html = state
        .fetcher
        .fetch_html(&form.url)
        .await
        .unwrap_or_default();
    let suggester = NoopSelectorSuggester;
    let suggestion = suggester.suggest(&html, &form.url).await?;
    Ok(Json(suggestion).into_response())
}
