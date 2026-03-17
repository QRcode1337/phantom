//! USGS earthquake feed client
//!
//! Fetches seismic event data from the USGS GeoJSON feed.
//! Extracts magnitude series for chaos analysis via ArgusDetector.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::http_client;

const USGS_BASE: &str = "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary";

/// A seismic event parsed from USGS GeoJSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeismicEvent {
    pub magnitude: f64,
    pub lat: f64,
    pub lon: f64,
    pub depth: f64,
    pub place: String,
    pub time: i64, // unix timestamp ms
}

#[derive(Debug, Deserialize)]
struct UsgsResponse {
    features: Vec<UsgsFeature>,
}

#[derive(Debug, Deserialize)]
struct UsgsFeature {
    properties: UsgsProperties,
    geometry: UsgsGeometry,
}

#[derive(Debug, Deserialize)]
struct UsgsProperties {
    mag: Option<f64>,
    place: Option<String>,
    time: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct UsgsGeometry {
    coordinates: Vec<f64>, // [lon, lat, depth]
}

/// Fetch earthquake events from USGS.
///
/// `period` options: "all_hour", "all_day", "all_week", "all_month",
///                   "significant_day", "significant_week", "significant_month"
pub async fn fetch_earthquakes(period: &str) -> Result<Vec<SeismicEvent>> {
    let client = http_client();
    let url = format!("{}/{}.geojson", USGS_BASE, period);

    let resp: UsgsResponse = client.get(&url).send().await?.json().await?;

    let mut events: Vec<SeismicEvent> = resp
        .features
        .into_iter()
        .filter_map(|f| {
            let mag = f.properties.mag?;
            let coords = &f.geometry.coordinates;
            if coords.len() < 3 {
                return None;
            }

            Some(SeismicEvent {
                magnitude: mag,
                lon: coords[0],
                lat: coords[1],
                depth: coords[2],
                place: f.properties.place.unwrap_or_default(),
                time: f.properties.time.unwrap_or(0),
            })
        })
        .collect();

    // Sort by time ascending for time-series analysis
    events.sort_by_key(|e| e.time);

    Ok(events)
}

/// Fetch just the magnitude series, sorted by time.
/// Ready for `ArgusDetector::analyze_seismic_series()`.
pub async fn fetch_magnitudes(period: &str) -> Result<Vec<f64>> {
    let events = fetch_earthquakes(period).await?;
    if events.is_empty() {
        bail!("no seismic events returned for period '{}'", period);
    }
    Ok(events.into_iter().map(|e| e.magnitude).collect())
}
