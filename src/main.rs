use std::{collections::HashMap, path::PathBuf, fs::{self, File}, time::SystemTime, io::BufReader};

use clap::Parser;
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

#[derive(Debug, Deserialize)]
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
    #[arg(short, long)]
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

    let item_id = mappings.iter().find(|&mapping| mapping.name.eq_ignore_ascii_case(&cli.item)).unwrap().id.to_string();

    // Filter the prices based on the item name
    let valid_price = prices.data
            .iter()
            .filter(|kv| kv.0.eq(&item_id))
            .next()
            .unwrap();
    println!("{} prices -> (high: {}, low: {})", &cli.item, valid_price.1.high.unwrap(), valid_price.1.low.unwrap());
    Ok(())
}
