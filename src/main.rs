use serde::Deserialize;

static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

#[derive(Debug, Deserialize)]
struct ItemMapping {
    examine: String,
    id: u32,
    members: bool,
    lowalch: Option<u32>,
    limit: Option<u32>,
    value: u32,
    highalch: Option<u32>,
    icon: String,
    name: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;

    let body = client.get("https://prices.runescape.wiki/api/v1/osrs/mapping")
        .send()
        .await?
        .json::<Vec<ItemMapping>>()
        .await?;

    println!("body = {:?}", body.get(0));
    Ok(())
}
