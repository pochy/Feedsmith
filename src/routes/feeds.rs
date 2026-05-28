use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};

use crate::{
    app::AppState,
    domain::feed::{Feed, FeedForm},
    error::AppResult,
};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    feeds: Vec<Feed>,
}

#[derive(Template)]
#[template(path = "feeds/new.html")]
struct NewFeedTemplate;

#[derive(Template)]
#[template(path = "feeds/edit.html")]
struct EditFeedTemplate {
    feed: Feed,
}

#[derive(Template)]
#[template(path = "feeds/_feed_table.html")]
struct FeedTableTemplate {
    feeds: Vec<Feed>,
}

pub async fn index(State(state): State<AppState>) -> AppResult<Html<String>> {
    let feeds = state.feeds.list().await?;
    Ok(Html(IndexTemplate { feeds }.render()?))
}

pub async fn new_feed() -> AppResult<Html<String>> {
    Ok(Html(NewFeedTemplate.render()?))
}

pub async fn create_feed(
    State(state): State<AppState>,
    Form(form): Form<FeedForm>,
) -> AppResult<Response> {
    state.feeds.create(form).await?;
    let feeds = state.feeds.list().await?;
    Ok(Html(FeedTableTemplate { feeds }.render()?).into_response())
}

pub async fn edit_feed(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Html<String>> {
    let feed = state.feeds.get(id).await?;
    Ok(Html(EditFeedTemplate { feed }.render()?))
}

pub async fn update_feed(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(form): Form<FeedForm>,
) -> AppResult<Response> {
    state.feeds.update(id, form).await?;
    Ok(Redirect::to("/").into_response())
}

pub async fn delete_feed(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    state.feeds.delete(id).await?;
    let feeds = state.feeds.list().await?;
    Ok(Html(FeedTableTemplate { feeds }.render()?).into_response())
}
