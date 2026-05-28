use askama::Template;
use axum::response::Html;

use crate::error::AppResult;

#[derive(Template)]
#[template(path = "picker.html")]
struct PickerTemplate;

pub async fn picker() -> AppResult<Html<String>> {
    Ok(Html(PickerTemplate.render()?))
}
