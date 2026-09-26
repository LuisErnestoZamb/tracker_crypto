pub mod arkham;
pub mod config;
pub mod db;
pub mod models;
pub mod schema;
pub mod tracker;
pub mod tronscan;

#[cfg(test)]
mod arkham_tests;

#[cfg(test)]
mod db_tests;

use anyhow::Result;
use config::Config;
use db::Database;
use std::path::Path;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::tracker::Tracker;

const SEED_WALLETS: &[&str] = &["TX"];

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let db = Database::new(&config).await?;

    let mut tracker = Tracker::new(config, db);

    if !tracker.exists_accounts().await? {
        for wallet in SEED_WALLETS {
            tracker.start_tracking(wallet);
            tracker.parser_account().await?;
            while tracker.should_continue {
                tracker.track_wallet().await?;
            }
        }
        info!("Done initial wallets");
    }

    let mut has_more = true;
    while has_more && file_exists("pull_wallets") {
        let wallets = tracker.get_wallet_list(250, 0).await?;

        if wallets.is_empty() {
            has_more = false;
            info!("No more wallets");
            continue;
        }

        for w in &wallets {
            tracker.start_tracking(&w.wallet);
            tracker.parser_account().await?;
            if !tracker.is_exchange().await? && !tracker.is_registered_wallet().await? {
                while tracker.should_continue {
                    tracker.track_wallet().await?;
                }
            }
        }
    }

    Ok(())
}

fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}
