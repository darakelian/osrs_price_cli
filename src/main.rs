use std::{collections::{HashMap, HashSet}, path::PathBuf, fs::{self, File}, time::SystemTime, io::BufReader};
use clap::Parser;
use num_format::{SystemLocale, ToFormattedString};
use reqwest::Client;
use serde::Deserialize;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

static MAPPING_URL: &str = "https://prices.runescape.wiki/api/v1/osrs/mapping";
static PRICES_LATEST_URL: &str = "https://prices.runescape.wiki/api/v1/osrs/latest";

/// Struct containing id and name of objects to look up
#[derive(Debug, Deserialize)]
struct ItemMapping {
    id: u32,
    name: String
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct PriceResult {
    high: Option<u32>,
    low: Option<u32>
}

#[derive(Debug, Deserialize)]
struct PriceResults {
    data: HashMap<String, PriceResult>
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    refresh_mappings: bool,
    #[arg(short, long)]
    force_prices: bool,
    #[arg(short, long, default_value = "./cache_dir")]
    cache_dir: PathBuf,
    #[arg(help = "Name of item (matches any item containing this string)")]
    item: String
}

/// Checks if the mappings should be refreshed
fn should_refresh_mapping(mapping_path: &PathBuf, refresh_mappings: bool) -> bool {
    if !mapping_path.exists() || refresh_mappings {
        return true;
    }
    false
}

fn should_refresh_prices(prices_path: &PathBuf, force_prices: bool) -> bool {
    if !prices_path.exists() || force_prices {
        return true;
    }
    // Check modified time, if > 5 minutes ago, refresh
    let metadata = prices_path
        .metadata()
        .expect("Unable to get metadata for prices.json");
    let mtime = metadata.modified().expect("Unable to access mtime for prices.json");
    let duration = SystemTime::now().duration_since(mtime).unwrap();
    if duration.as_secs() >= 5 * 60 {
        return true;
    }
    false
}

async fn get_mappings(client: &Client, cache_dir: &PathBuf, refresh_mappings: bool) -> Result<Vec<ItemMapping>, Box<dyn std::error::Error>> {
    let mapping_path: PathBuf = cache_dir.join("mappings.json");

    if should_refresh_mapping(&mapping_path, refresh_mappings) {
        // Refresh mapping file if needed
        let mapping_body = client.get(MAPPING_URL)
            .send()
            .await?
            .text()
            .await?;
        fs::create_dir_all(mapping_path.parent().unwrap()).expect("Unable to create directories");
        fs::write(mapping_path, &mapping_body).expect("Unable to save mapping data");
        let results = serde_json::from_str(&mapping_body)?;
        return Ok(results);
    }
    let reader = BufReader::new(File::open(mapping_path)?);
    let results = serde_json::from_reader(reader)?;
    Ok(results)
}

async fn get_prices(client: &Client, cache_dir: &PathBuf, force_prices: bool) -> Result<PriceResults, Box<dyn std::error::Error>> {
    let prices_file = cache_dir.join("prices.json");
    if should_refresh_prices(&prices_file, force_prices) {
        // Re-download the prices
        let prices_body = client.get(PRICES_LATEST_URL)
            .send()
            .await?
            .text()
            .await?;
        fs::create_dir_all(prices_file.parent().unwrap())?;
        fs::write(prices_file, &prices_body).expect("Unable to save prices data");
        let results: PriceResults = serde_json::from_str(&prices_body)?;
        return Ok(results);
    }
    let reader = BufReader::new(File::open(prices_file)?);
    let results = serde_json::from_reader(reader)?;
    Ok(results)
}

fn get_matching_item_ids(name: &String, mappings: &Vec<ItemMapping>) -> HashSet<u32> {
    let lower = name.to_lowercase();
    HashSet::from_iter(mappings
        .iter()
        .filter(|&mapping| mapping.name.to_lowercase().contains(&lower))
        .map(|mapping| mapping.id))
}

fn display_price(name: &String, price: &PriceResult) {
    let locale = SystemLocale::default().unwrap();
    println!("{} -> high: {}, low: {}", name, price.high.unwrap_or(0).to_formatted_string(&locale), price.low.unwrap_or(0).to_formatted_string(&locale));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let cli = Cli::parse();

    // Load mappings
    let mappings = get_mappings(&client, &cli.cache_dir, cli.refresh_mappings).await?;

    // Refresh prices if >5 minutes or user requests new prices
    let prices = get_prices(&client, &cli.cache_dir, cli.force_prices).await?;

    // Get item_ids for all items containing input item
    let item_ids: HashSet<u32> = get_matching_item_ids(&cli.item, &mappings);
    
    // Display the results
    for item_id in item_ids.iter() {
        let name = &mappings.iter().find(|&m| m.id == *item_id).unwrap().name;
        match prices.data.get(&item_id.to_string()) {
            Some(price) => display_price(&name, &price),
            None => continue
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[tokio::test]
    async fn test_name_matching() {
        let client = Client::new();
        
        let mut cache_dir = env::current_dir().unwrap();
        cache_dir.push("src");
        cache_dir.push("test_data");

        let mappings = get_mappings(&client, &cache_dir, false).await.unwrap();
        
        let single_name_ids = get_matching_item_ids(&String::from("Zulrah's scales"), &mappings);
        assert_eq!(single_name_ids.len(), 1);

        let multi_name_ids = get_matching_item_ids(&String::from("twisted"), &mappings);
        assert_eq!(multi_name_ids.len(), 23);
    }
}