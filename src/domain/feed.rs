use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Feed {
    pub id: i64,
    pub name: String,
    pub site_url: String,
    pub feed_description: Option<String>,
    pub selector_type: String,
    pub item_selector: String,
    pub title_selector: String,
    pub link_selector: Option<String>,
    pub date_selector: Option<String>,
    pub content_selector: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeedForm {
    pub name: String,
    pub site_url: String,
    pub feed_description: Option<String>,
    pub selector_type: Option<String>,
    pub item_selector: String,
    pub title_selector: String,
    pub link_selector: Option<String>,
    pub date_selector: Option<String>,
    pub content_selector: Option<String>,
}

impl FeedForm {
    pub fn normalize(mut self) -> Self {
        self.feed_description = blank_to_none(self.feed_description);
        self.selector_type =
            Some(blank_to_none(self.selector_type).unwrap_or_else(|| "css".to_string()));
        self.link_selector = blank_to_none(self.link_selector);
        self.date_selector = blank_to_none(self.date_selector);
        self.content_selector = blank_to_none(self.content_selector);
        self
    }
}

fn blank_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}
