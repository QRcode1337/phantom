//! Kalshi prediction market API client
//!
//! Authenticated client using RSA-PSS (SHA256) request signing.
//! Credential loading: env vars first, fallback to ~/.config/kalshi/credentials.json.

use anyhow::{bail, Context, Result};
use base64::Engine as _;
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::BlindedSigningKey;
use rsa::sha2::Sha256;
use rsa::signature::RandomizedSigner;
use rsa::signature::SignatureEncoding;
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};

use super::http_client;

const PROD_BASE_URL: &str = "https://api.elections.kalshi.com/trade-api/v2";
const DEMO_BASE_URL: &str = "https://demo-api.kalshi.co/trade-api/v2";

/// Kalshi API client with RSA-PSS authentication.
pub struct KalshiClient {
    api_key_id: String,
    signing_key: BlindedSigningKey<Sha256>,
    base_url: String,
    client: reqwest::Client,
}

/// Kalshi market info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketInfo {
    pub ticker: String,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub status: Option<String>,
    pub yes_bid: Option<f64>,
    pub yes_ask: Option<f64>,
    pub no_bid: Option<f64>,
    pub no_ask: Option<f64>,
    pub last_price: Option<f64>,
    pub volume: Option<u64>,
    pub open_interest: Option<u64>,
}

/// Kalshi orderbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    pub yes: Option<Vec<Vec<f64>>>,
    pub no: Option<Vec<Vec<f64>>>,
}

#[derive(Debug, Deserialize)]
struct CredentialsFile {
    api_key_id: String,
    private_key_path: String,
}


impl KalshiClient {
    /// Create a production Kalshi client.
    /// Tries env vars first, falls back to ~/.config/kalshi/credentials.json.
    pub fn new() -> Result<Self> {
        Self::with_base_url(PROD_BASE_URL)
    }

    /// Create a demo/sandbox Kalshi client.
    pub fn demo() -> Result<Self> {
        Self::with_base_url(DEMO_BASE_URL)
    }

    fn with_base_url(base_url: &str) -> Result<Self> {
        let (api_key_id, key_path) = load_credentials()?;
        let private_key = load_private_key(&key_path)?;
        let signing_key = BlindedSigningKey::<Sha256>::new(private_key);

        Ok(KalshiClient {
            api_key_id,
            signing_key,
            base_url: base_url.to_string(),
            client: http_client(),
        })
    }

    /// Make a signed GET request to the Kalshi API.
    async fn signed_get(&self, path: &str) -> Result<serde_json::Value> {
        let full_path = format!("/trade-api/v2{}", path);
        let timestamp_ms = chrono::Utc::now().timestamp_millis().to_string();

        // Message to sign: {timestamp}{METHOD}{path}
        let message = format!("{}GET{}", timestamp_ms, full_path);

        // RSA-PSS sign (scoped to drop ThreadRng before any .await)
        let sig_b64 = {
            let mut rng = rand::thread_rng();
            let signature = self.signing_key.sign_with_rng(&mut rng, message.as_bytes());
            base64::engine::general_purpose::STANDARD.encode(signature.to_bytes())
        };

        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .header("KALSHI-ACCESS-KEY", &self.api_key_id)
            .header("KALSHI-ACCESS-TIMESTAMP", &timestamp_ms)
            .header("KALSHI-ACCESS-SIGNATURE", &sig_b64)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("Kalshi API error {}: {}", status, body);
        }

        let json: serde_json::Value = resp.json().await?;
        Ok(json)
    }

    /// Get a single market by ticker.
    pub async fn get_market(&self, ticker: &str) -> Result<MarketInfo> {
        let resp = self.signed_get(&format!("/markets/{}", ticker)).await?;
        let market = resp
            .get("market")
            .ok_or_else(|| anyhow::anyhow!("no 'market' field in response"))?;
        parse_market_info(market)
    }

    /// List markets for a series ticker.
    pub async fn get_markets(&self, series_ticker: &str) -> Result<Vec<MarketInfo>> {
        let resp = self
            .signed_get(&format!(
                "/markets?series_ticker={}&status=open&limit=100",
                series_ticker
            ))
            .await?;

        let markets = resp
            .get("markets")
            .and_then(|m| m.as_array())
            .ok_or_else(|| anyhow::anyhow!("no 'markets' array in response"))?;

        markets.iter().map(parse_market_info).collect()
    }

    /// Get market price history as a Vec<f64> of YES prices (cents, 0-100).
    pub async fn get_market_history(&self, ticker: &str) -> Result<Vec<f64>> {
        let resp = self
            .signed_get(&format!("/markets/{}/history", ticker))
            .await?;

        let history = resp
            .get("history")
            .and_then(|h| h.as_array())
            .ok_or_else(|| anyhow::anyhow!("no 'history' array in response"))?;

        let prices: Vec<f64> = history
            .iter()
            .filter_map(|point| {
                point
                    .get("yes_price")
                    .and_then(|p| p.as_f64())
            })
            .collect();

        if prices.is_empty() {
            bail!("empty price history for '{}'", ticker);
        }

        Ok(prices)
    }

    /// Get the current orderbook for a market.
    pub async fn get_orderbook(&self, ticker: &str) -> Result<Orderbook> {
        let resp = self
            .signed_get(&format!("/markets/{}/orderbook", ticker))
            .await?;

        let orderbook = resp
            .get("orderbook")
            .ok_or_else(|| anyhow::anyhow!("no 'orderbook' field in response"))?;

        let ob: Orderbook = serde_json::from_value(orderbook.clone())?;
        Ok(ob)
    }
}

fn parse_market_info(market: &serde_json::Value) -> Result<MarketInfo> {
    Ok(MarketInfo {
        ticker: market
            .get("ticker")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string(),
        title: market.get("title").and_then(|t| t.as_str()).map(String::from),
        subtitle: market.get("subtitle").and_then(|t| t.as_str()).map(String::from),
        status: market.get("status").and_then(|t| t.as_str()).map(String::from),
        yes_bid: market.get("yes_bid").and_then(|v| v.as_f64()),
        yes_ask: market.get("yes_ask").and_then(|v| v.as_f64()),
        no_bid: market.get("no_bid").and_then(|v| v.as_f64()),
        no_ask: market.get("no_ask").and_then(|v| v.as_f64()),
        last_price: market.get("last_price").and_then(|v| v.as_f64()),
        volume: market.get("volume").and_then(|v| v.as_u64()),
        open_interest: market.get("open_interest").and_then(|v| v.as_u64()),
    })
}

/// Load credentials from env vars, falling back to ~/.config/kalshi/credentials.json.
fn load_credentials() -> Result<(String, String)> {
    // Try env vars first
    if let (Ok(key_id), Ok(key_path)) = (
        std::env::var("KALSHI_API_KEY_ID"),
        std::env::var("KALSHI_PRIVATE_KEY_PATH"),
    ) {
        return Ok((key_id, key_path));
    }

    // Fallback to credentials file
    let home = std::env::var("HOME").context("HOME not set")?;
    let creds_path = format!("{}/.config/kalshi/credentials.json", home);
    let contents = std::fs::read_to_string(&creds_path)
        .with_context(|| format!("failed to read {}", creds_path))?;
    let creds: CredentialsFile = serde_json::from_str(&contents)
        .with_context(|| "failed to parse credentials.json")?;

    // Expand ~ in private_key_path
    let key_path = if creds.private_key_path.starts_with('~') {
        creds.private_key_path.replacen('~', &home, 1)
    } else {
        creds.private_key_path
    };

    Ok((creds.api_key_id, key_path))
}

/// Load and parse an RSA private key from a PEM file.
fn load_private_key(path: &str) -> Result<RsaPrivateKey> {
    let pem = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read private key: {}", path))?;
    RsaPrivateKey::from_pkcs8_pem(&pem)
        .or_else(|_| {
            // Try PKCS1 format as fallback
            use rsa::pkcs1::DecodeRsaPrivateKey;
            RsaPrivateKey::from_pkcs1_pem(&pem)
        })
        .context("failed to parse RSA private key (tried PKCS8 and PKCS1)")
}
