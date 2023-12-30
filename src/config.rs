use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use directories::ProjectDirs;

/// App CLI args
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(help = "Name of item (matches any item containing this string)")]
    item: String,

    #[arg(short = 'c', long)]
    cache_dir: Option<PathBuf>,

    #[arg(short = 't', long, default_value_t = 300)]
    price_cache_ttl_secs: u64,

    #[arg(short = 'p', long, action)]
    refresh_prices: bool,

    #[arg(short = 'm', long, action)]
    refresh_mappings: bool,
}

/// Client config
#[derive(Debug)]
pub struct ClientConfig {
    /// Cache dir
    pub cache_dir: PathBuf,

    /// Price Cache TTL
    pub price_cache_ttl: Duration,
}

/// App config
#[derive(Debug)]
pub struct Config {
    /// Name of item to search for
    pub item: String,

    /// Refresh Mappings (even if cached)
    pub refresh_mappings: bool,

    /// Refresh Prices (even if cached)
    pub refresh_prices: bool,

    /// client config
    pub client: ClientConfig,
}

impl Config {
    const QUALIFIER: &'static str = "com";
    const ORGANIZATION: &'static str = "darakelian";
    const APPLICATION: &'static str = env!("CARGO_PKG_NAME");

    pub fn from_cli() -> Self {
        Cli::parse().into()
    }

    fn project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from(Self::QUALIFIER, Self::ORGANIZATION, Self::APPLICATION)
    }

    fn default_cache_dir() -> PathBuf {
        Self::project_dirs()
            .map(|p| p.cache_dir().into())
            .unwrap_or(PathBuf::from("cache_dir"))
    }
}

impl From<Cli> for Config {
    fn from(cli: Cli) -> Self {
        let cache_dir = cli.cache_dir.unwrap_or_else(|| Self::default_cache_dir());

        let client = ClientConfig {
            cache_dir,
            price_cache_ttl: Duration::from_secs(cli.price_cache_ttl_secs),
        };

        Self {
            item: cli.item,
            refresh_mappings: cli.refresh_mappings,
            refresh_prices: cli.refresh_prices,
            client,
        }
    }
}
