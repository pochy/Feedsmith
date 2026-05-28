pub mod feeds;
pub mod picker;
pub mod preview;
pub mod rss;

use axum::{
    routing::{get, post},
    Router,
};

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(feeds::index))
        .route("/feeds/new", get(feeds::new_feed))
        .route("/feeds", post(feeds::create_feed))
        .route("/feeds/:id/edit", get(feeds::edit_feed))
        .route("/feeds/:id", post(feeds::update_feed))
        .route("/feeds/:id/delete", post(feeds::delete_feed))
        .route("/feeds/:id/preview", post(preview::preview_feed))
        .route("/feeds/:id/rss.xml", get(rss::feed_rss))
        .route("/picker", get(picker::picker))
        .route("/api/fetch-preview-html", post(preview::fetch_preview_html))
        .route("/api/suggest-selectors", post(preview::suggest_selectors))
}
