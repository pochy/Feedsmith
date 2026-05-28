use std::{env, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub fetch_timeout: Duration,
    pub fetch_max_body_bytes: usize,
    pub fetch_max_redirects: usize,
    pub user_agent: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("APP_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://feedsmith.db".to_string()),
            fetch_timeout: Duration::from_secs(
                env::var("FETCH_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
            ),
            fetch_max_body_bytes: env::var("FETCH_MAX_BODY_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2 * 1024 * 1024),
            fetch_max_redirects: env::var("FETCH_MAX_REDIRECTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            user_agent: env::var("USER_AGENT").unwrap_or_else(|_| "Feedsmith/0.1".to_string()),
        }
    }
}
