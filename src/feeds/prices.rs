//! Price history fetchers
//!
//! Aggregates price data from multiple sources:
//! - Kalshi prediction market contract prices (via authenticated client)
//! - Exchange prices (BTC, etc.) from CoinGecko free API

use anyhow::{bail, Result};

use super::http_client;
use super::kalshi::KalshiClient;

const COINGECKO_BASE: &str = "https://api.coingecko.com/api/v3";

/// Fetch Kalshi market price history as a `Vec<f64>` of YES prices.
pub async fn fetch_kalshi_market_history(
    ticker: &str,
    client: &KalshiClient,
) -> Result<Vec<f64>> {
    client.get_market_history(ticker).await
}

/// Fetch BTC/USD price history from CoinGecko.
pub async fn fetch_btc_price_history(days: usize) -> Result<Vec<f64>> {
    fetch_exchange_price("bitcoin", days).await
}

/// Fetch price history for any CoinGecko-supported asset.
/// Returns price series sorted chronologically.
pub async fn fetch_exchange_price(coin_id: &str, days: usize) -> Result<Vec<f64>> {
    let client = http_client();
    let url = format!(
        "{}/coins/{}/market_chart?vs_currency=usd&days={}",
        COINGECKO_BASE, coin_id, days
    );

    let resp: serde_json::Value = client
        .get(&url)
        .header("User-Agent", "phantom/0.1.0")
        .header("Accept", "application/json")
        .send()
        .await?
        .json()
        .await?;

    let prices = resp
        .get("prices")
        .and_then(|p| p.as_array())
        .ok_or_else(|| anyhow::anyhow!("no 'prices' array in CoinGecko response"))?;

    let series: Vec<f64> = prices
        .iter()
        .filter_map(|pair| {
            pair.as_array()
                .and_then(|arr| arr.get(1))
                .and_then(|v| v.as_f64())
        })
        .collect();

    if series.is_empty() {
        bail!("empty price series from CoinGecko for '{}'", coin_id);
    }

    Ok(series)
}
