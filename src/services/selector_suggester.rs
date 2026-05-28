use async_trait::async_trait;
use serde::Serialize;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct SelectorSuggestion {
    pub message: String,
    pub item_selector: Option<String>,
    pub title_selector: Option<String>,
    pub link_selector: Option<String>,
    pub date_selector: Option<String>,
    pub content_selector: Option<String>,
}

#[async_trait]
pub trait SelectorSuggester: Send + Sync {
    async fn suggest(&self, html: &str, url: &str) -> Result<SelectorSuggestion, AppError>;
}

#[derive(Debug, Default)]
pub struct NoopSelectorSuggester;

#[async_trait]
impl SelectorSuggester for NoopSelectorSuggester {
    async fn suggest(&self, _html: &str, _url: &str) -> AppResult<SelectorSuggestion> {
        Ok(SelectorSuggestion {
            message: "AI selector suggestion is not enabled in this MVP.".to_string(),
            item_selector: None,
            title_selector: None,
            link_selector: None,
            date_selector: None,
            content_selector: None,
        })
    }
}
