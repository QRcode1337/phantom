//! External data feed clients for Phantom detectors.
//!
//! Each module fetches real-world data and returns `Vec<f64>` series
//! ready for consumption by detectors in `src/detectors/`.

pub mod open_meteo;
pub mod opensky;
pub mod usgs;
pub mod prices;
pub mod kalshi;

pub use kalshi::KalshiClient;

use reqwest::Client;

/// Build a shared HTTP client with 10s timeout.
pub fn http_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client")
}
