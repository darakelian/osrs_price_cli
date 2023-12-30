use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config::ClientConfig;

/// Struct containing id and name of objects to look up
#[derive(Debug, Deserialize)]
pub struct ItemMapping {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PriceResult {
    pub high: Option<u32>,
    pub low: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct PriceResults {
    pub data: HashMap<u32, PriceResult>,
}

/// OSRS Prices API Client
pub struct Client {
    client: reqwest::Client,
    config: ClientConfig,
}

impl Client {
    const APP_USER_AGENT: &'static str =
        concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    const MAPPING_URL: &'static str = "https://prices.runescape.wiki/api/v1/osrs/mapping";
    const PRICES_LATEST_URL: &'static str = "https://prices.runescape.wiki/api/v1/osrs/latest";

    /// Try to build Client from ClientConfig
    pub fn try_from_config(config: ClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(Self::APP_USER_AGENT)
            .build()
            .context("Could not build reqwest::Client")?;

        Ok(Self { client, config })
    }

    fn prices_cache(&self) -> PathBuf {
        self.config.cache_dir.join("prices.json")
    }

    fn mappings_cache(&self) -> PathBuf {
        self.config.cache_dir.join("mappings.json")
    }

    /// Checks if the mappings should be refreshed
    fn should_refresh_mappings(&self) -> bool {
        !self.mappings_cache().exists()
    }

    fn should_refresh_prices(&self) -> bool {
        let cache = self.prices_cache();
        if !cache.exists() {
            return true;
        }

        // Check modified time, if > TTL ago, refresh
        let metadata = cache.metadata().expect("Prices cache should exist");
        let mtime = metadata
            .modified()
            .expect("Unable to access mtime for prices cache");

        let duration = SystemTime::now()
            .duration_since(mtime)
            .expect("Prices cache mtime should not be in the future");

        duration > self.config.price_cache_ttl
    }

    pub async fn get_mappings(&self, force_refresh: bool) -> Result<Vec<ItemMapping>> {
        let cache = self.mappings_cache();

        if force_refresh || self.should_refresh_mappings() {
            // Refresh mapping file if needed
            let body = self
                .client
                .get(Self::MAPPING_URL)
                .send()
                .await
                .context("Failed to send request")?
                .text()
                .await
                .context("Failed to receive response")?;

            if let Some(parent) = cache.parent() {
                fs::create_dir_all(parent).context("Unable to create directories")?;
            }

            fs::write(cache, &body).context("Unable to save mapping data")?;

            serde_json::from_str(&body).context("Unable to parse response JSON")
        } else {
            let reader =
                BufReader::new(File::open(cache).context("Unable to open mappings cache file")?);

            serde_json::from_reader(reader).context("Unable to parse mappings cache file")
        }
    }

    pub async fn get_prices(&self, force_refresh: bool) -> Result<PriceResults> {
        let cache = self.prices_cache();

        if force_refresh || self.should_refresh_prices() {
            // Refresh mapping file if needed
            let body = self
                .client
                .get(Self::PRICES_LATEST_URL)
                .send()
                .await
                .context("Failed to send request")?
                .text()
                .await
                .context("Failed to receive response")?;

            if let Some(parent) = cache.parent() {
                fs::create_dir_all(parent).context("Unable to create directories")?;
            }

            fs::write(cache, &body).context("Unable to save prices data")?;

            serde_json::from_str(&body).context("Unable to parse response JSON")
        } else {
            let reader =
                BufReader::new(File::open(cache).context("Unable to open prices cache file")?);

            serde_json::from_reader(reader).context("Unable to parse prices cache file")
        }
    }
}
