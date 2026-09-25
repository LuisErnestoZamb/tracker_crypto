use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub turso_url: String,
    pub turso_auth_token: Option<String>,
    pub tron_token: String,
    pub arkm_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            turso_url: env::var("TURSO_URL").context("TURSO_URL not set")?,
            turso_auth_token: env::var("TURSO_AUTH_TOKEN").ok(),
            tron_token: env::var("TRON_TOKEN").context("TRON_TOKEN not set")?,
            arkm_api_key: env::var("ARKM_API_KEY").ok(),
        })
    }
}
