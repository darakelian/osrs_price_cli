mod client;
mod config;

use anyhow::Result;
use log::info;
use num_format::{SystemLocale, ToFormattedString};

use crate::{
    client::{Client, ItemMapping, PriceResult},
    config::Config,
};

fn get_matching_items<S: AsRef<str>>(
    name: S,
    mappings: &[ItemMapping],
) -> impl Iterator<Item = &ItemMapping> {
    let lower = name.as_ref().to_lowercase();

    mappings.iter().filter_map(move |mapping| {
        mapping
            .name
            .to_lowercase()
            .contains(&lower)
            .then_some(mapping)
    })
}

fn display_price(name: &String, price: &PriceResult) {
    let locale = SystemLocale::default().unwrap();
    println!(
        "{} -> high: {}, low: {}",
        name,
        price
            .high
            .map(|p| p.to_formatted_string(&locale))
            .unwrap_or("N/A".into()),
        price
            .low
            .map(|p| p.to_formatted_string(&locale))
            .unwrap_or("N/A".into()),
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = Config::from_cli();
    info!("Using config: {:?}", config);

    let client = Client::try_from_config(config.client)?;

    let mappings = client.get_mappings(config.refresh_mappings).await?;
    let prices = client.get_prices(config.refresh_prices).await?;

    // Get item_ids for all items containing input item
    for item in get_matching_items(&config.item, &mappings) {
        // Display the results
        if let Some(price) = prices.data.get(&item.id) {
            display_price(&item.name, &price);
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use crate::config::ClientConfig;

    use super::*;

    #[tokio::test]
    async fn test_name_matching() {
        let cache_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "test_data"].iter().collect();
        let config = ClientConfig {
            cache_dir,
            price_cache_ttl: Duration::from_secs(u64::MAX),
        };

        let client = Client::try_from_config(config).expect("Client should be created");

        let mappings = client
            .get_mappings(false)
            .await
            .expect("Client mappings should be loaded from test data");

        let single_name_ids = get_matching_items("Zulrah's scales", &mappings).collect::<Vec<_>>();
        assert_eq!(single_name_ids.len(), 1);

        let multi_name_ids = get_matching_items("twisted", &mappings).collect::<Vec<_>>();
        assert_eq!(multi_name_ids.len(), 23);
    }
}
