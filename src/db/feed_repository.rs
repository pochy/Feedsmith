use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::{
    domain::feed::{Feed, FeedForm},
    error::{AppError, AppResult},
};

#[derive(Debug, Clone)]
pub struct FeedRepository {
    pool: SqlitePool,
}

impl FeedRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> AppResult<Vec<Feed>> {
        sqlx::query_as::<_, Feed>("SELECT * FROM feeds ORDER BY id DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn get(&self, id: i64) -> AppResult<Feed> {
        sqlx::query_as::<_, Feed>("SELECT * FROM feeds WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::NotFound)
    }

    pub async fn create(&self, form: FeedForm) -> AppResult<i64> {
        let form = form.normalize();
        validate_form(&form)?;
        let now = Utc::now();
        let res = sqlx::query(
            r#"
            INSERT INTO feeds (
              name, site_url, feed_description, selector_type,
              item_selector, title_selector, link_selector, date_selector, content_selector,
              created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(form.name.trim())
        .bind(form.site_url.trim())
        .bind(form.feed_description)
        .bind(form.selector_type.unwrap_or_else(|| "css".to_string()))
        .bind(form.item_selector.trim())
        .bind(form.title_selector.trim())
        .bind(form.link_selector)
        .bind(form.date_selector)
        .bind(form.content_selector)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(res.last_insert_rowid())
    }

    pub async fn update(&self, id: i64, form: FeedForm) -> AppResult<()> {
        let form = form.normalize();
        validate_form(&form)?;
        let now = Utc::now();
        let res = sqlx::query(
            r#"
            UPDATE feeds SET
              name = ?, site_url = ?, feed_description = ?, selector_type = ?,
              item_selector = ?, title_selector = ?, link_selector = ?,
              date_selector = ?, content_selector = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(form.name.trim())
        .bind(form.site_url.trim())
        .bind(form.feed_description)
        .bind(form.selector_type.unwrap_or_else(|| "css".to_string()))
        .bind(form.item_selector.trim())
        .bind(form.title_selector.trim())
        .bind(form.link_selector)
        .bind(form.date_selector)
        .bind(form.content_selector)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    pub async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM feeds WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn first_seen(
        &self,
        feed_id: i64,
        item_url: &str,
    ) -> AppResult<Option<DateTime<Utc>>> {
        let row: Option<(DateTime<Utc>,)> = sqlx::query_as(
            "SELECT first_seen_at FROM feed_items WHERE feed_id = ? AND item_url = ?",
        )
        .bind(feed_id)
        .bind(item_url)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn upsert_seen(
        &self,
        feed_id: i64,
        item_url: &str,
        title: &str,
        seen_at: DateTime<Utc>,
    ) -> AppResult<DateTime<Utc>> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO feed_items (feed_id, item_url, title, first_seen_at, last_seen_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(feed_id, item_url) DO UPDATE SET
              title = excluded.title,
              last_seen_at = excluded.last_seen_at
            "#,
        )
        .bind(feed_id)
        .bind(item_url)
        .bind(title)
        .bind(seen_at)
        .bind(now)
        .execute(&self.pool)
        .await?;
        self.first_seen(feed_id, item_url)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("failed to read first_seen_at")))
    }
}

fn validate_form(form: &FeedForm) -> AppResult<()> {
    if form.name.trim().is_empty() {
        return Err(AppError::BadRequest("name is required".to_string()));
    }
    if form.site_url.trim().is_empty() {
        return Err(AppError::BadRequest("site_url is required".to_string()));
    }
    if form.item_selector.trim().is_empty() {
        return Err(AppError::BadRequest(
            "item_selector is required".to_string(),
        ));
    }
    if form.title_selector.trim().is_empty() {
        return Err(AppError::BadRequest(
            "title_selector is required".to_string(),
        ));
    }
    match form.selector_type.as_deref().unwrap_or("css") {
        "css" => Ok(()),
        "xpath" => Err(AppError::BadRequest(
            "xpath is reserved for a future version".to_string(),
        )),
        other => Err(AppError::BadRequest(format!(
            "unsupported selector_type: {other}"
        ))),
    }
}
