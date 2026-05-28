use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedItem {
    pub title: String,
    pub link: String,
    pub date_text: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RssItemData {
    pub title: String,
    pub link: String,
    pub pub_date: DateTime<Utc>,
    pub description: Option<String>,
}
