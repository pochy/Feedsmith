use std::time::Duration;

use crate::{
    error::{AppError, AppResult},
    services::url_guard,
};
use futures_util::StreamExt;
use reqwest::{header, redirect::Policy, Client, StatusCode};

#[derive(Debug)]
pub struct Fetcher {
    client: Client,
    timeout: Duration,
    max_body_bytes: usize,
    max_redirects: usize,
    user_agent: String,
}

impl Fetcher {
    pub fn new(
        timeout: Duration,
        max_body_bytes: usize,
        max_redirects: usize,
        user_agent: String,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::builder()
                .redirect(Policy::none())
                .timeout(timeout)
                .build()?,
            timeout,
            max_body_bytes,
            max_redirects,
            user_agent,
        })
    }

    pub async fn fetch_html(&self, raw_url: &str) -> AppResult<String> {
        let mut url = url_guard::validate_url(raw_url)?;
        for redirect_count in 0..=self.max_redirects {
            url_guard::validate_resolved_url(&url).await?;
            let response = self
                .client
                .get(url.clone())
                .timeout(self.timeout)
                .header(header::USER_AGENT, self.user_agent.as_str())
                .send()
                .await
                .map_err(|e| AppError::Fetch(e.to_string()))?;

            if is_redirect(response.status()) {
                if redirect_count == self.max_redirects {
                    return Err(AppError::Fetch("too many redirects".to_string()));
                }
                let location = response
                    .headers()
                    .get(header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| AppError::Fetch("redirect without Location".to_string()))?;
                url = url
                    .join(location)
                    .map_err(|_| AppError::Fetch("invalid redirect URL".to_string()))?;
                url_guard::validate_parsed_url(&url)?;
                continue;
            }

            if !response.status().is_success() {
                return Err(AppError::Fetch(format!(
                    "unexpected status {}",
                    response.status()
                )));
            }
            validate_content_type(response.headers())?;
            return self.read_limited_body(response).await;
        }
        Err(AppError::Fetch("too many redirects".to_string()))
    }

    async fn read_limited_body(&self, response: reqwest::Response) -> AppResult<String> {
        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AppError::Fetch(e.to_string()))?;
            if bytes.len() + chunk.len() > self.max_body_bytes {
                return Err(AppError::Fetch("response body too large".to_string()));
            }
            bytes.extend_from_slice(&chunk);
        }
        String::from_utf8(bytes).map_err(|_| AppError::Fetch("response is not UTF-8".to_string()))
    }
}

fn is_redirect(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    )
}

fn validate_content_type(headers: &header::HeaderMap) -> AppResult<()> {
    let Some(value) = headers.get(header::CONTENT_TYPE) else {
        return Ok(());
    };
    let value = value
        .to_str()
        .map_err(|_| AppError::Fetch("invalid Content-Type".to_string()))?;
    let mime = value
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match mime.as_str() {
        "text/html" | "application/xhtml+xml" | "application/xml" | "text/xml" => Ok(()),
        _ => Err(AppError::Fetch(format!("unsupported Content-Type: {mime}"))),
    }
}
