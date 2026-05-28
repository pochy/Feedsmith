use std::sync::Arc;

use axum::Router;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{config::Config, db::FeedRepository, routes, services::fetcher::Fetcher};

#[derive(Clone)]
pub struct AppState {
    pub feeds: FeedRepository,
    pub fetcher: Arc<Fetcher>,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let options: SqliteConnectOptions = config.database_url.parse()?;
        let db =
            SqlitePool::connect_with(options.create_if_missing(true).foreign_keys(true)).await?;
        sqlx::migrate!("./migrations").run(&db).await?;
        let feeds = FeedRepository::new(db.clone());
        let fetcher = Arc::new(Fetcher::new(
            config.fetch_timeout,
            config.fetch_max_body_bytes,
            config.fetch_max_redirects,
            config.user_agent.clone(),
        )?);
        Ok(Self { feeds, fetcher })
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(routes::router())
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
