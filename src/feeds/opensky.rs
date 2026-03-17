//! OpenSky / ADS-B flight data client
//!
//! Fetches live aircraft state vectors for geospatial anomaly detection.
//! Primary: OpenSky Network API (rate-limited).
//! Fallback: adsb.lol (free, no auth, no rate limit).

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::http_client;

const OPENSKY_URL: &str = "https://opensky-network.org/api/states/all";
const ADSB_LOL_URL: &str = "https://api.adsb.lol/v2/mil";

/// Aircraft state from OpenSky/ADS-B feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftState {
    pub icao24: String,
    pub callsign: Option<String>,
    pub lat: f64,
    pub lon: f64,
    pub baro_altitude: Option<f64>,
    pub geo_altitude: Option<f64>,
    pub velocity: Option<f64>,
    pub track: Option<f64>,
    pub vertical_rate: Option<f64>,
    pub on_ground: bool,
}

#[derive(Debug, Deserialize)]
struct OpenSkyResponse {
    states: Option<Vec<Vec<serde_json::Value>>>,
}

#[derive(Debug, Deserialize)]
struct AdsbLolResponse {
    ac: Option<Vec<AdsbLolAircraft>>,
}

#[derive(Debug, Deserialize)]
struct AdsbLolAircraft {
    hex: Option<String>,
    flight: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    alt_baro: Option<serde_json::Value>,
    alt_geom: Option<f64>,
    gs: Option<f64>,
    track: Option<f64>,
    baro_rate: Option<f64>,
    geom_rate: Option<f64>,
}

/// Fetch aircraft states, optionally filtered by bounding box (lamin, lomin, lamax, lomax).
/// Tries OpenSky first, falls back to adsb.lol on failure.
pub async fn fetch_states(bbox: Option<(f64, f64, f64, f64)>) -> Result<Vec<AircraftState>> {
    match fetch_opensky(bbox).await {
        Ok(states) if !states.is_empty() => Ok(states),
        _ => fetch_adsb_lol().await,
    }
}

async fn fetch_opensky(bbox: Option<(f64, f64, f64, f64)>) -> Result<Vec<AircraftState>> {
    let client = http_client();
    let url = match bbox {
        Some((lamin, lomin, lamax, lomax)) => format!(
            "{}?lamin={}&lomin={}&lamax={}&lomax={}",
            OPENSKY_URL, lamin, lomin, lamax, lomax
        ),
        None => OPENSKY_URL.to_string(),
    };

    let resp: OpenSkyResponse = client.get(&url).send().await?.json().await?;

    let states = resp.states.unwrap_or_default();
    let mut aircraft = Vec::with_capacity(states.len());

    for state in &states {
        if state.len() < 17 {
            continue;
        }

        let lat = match state[6].as_f64() {
            Some(v) => v,
            None => continue,
        };
        let lon = match state[5].as_f64() {
            Some(v) => v,
            None => continue,
        };

        aircraft.push(AircraftState {
            icao24: state[0].as_str().unwrap_or("unknown").to_string(),
            callsign: state[1].as_str().map(|s| s.trim().to_string()),
            lat,
            lon,
            baro_altitude: state[7].as_f64(),
            geo_altitude: state[13].as_f64(),
            velocity: state[9].as_f64(),
            track: state[10].as_f64(),
            vertical_rate: state[11].as_f64(),
            on_ground: state[8].as_bool().unwrap_or(false),
        });
    }

    Ok(aircraft)
}

async fn fetch_adsb_lol() -> Result<Vec<AircraftState>> {
    let client = http_client();
    let resp: AdsbLolResponse = client.get(ADSB_LOL_URL).send().await?.json().await?;

    let ac_list = resp.ac.unwrap_or_default();
    let mut aircraft = Vec::with_capacity(ac_list.len());

    for ac in &ac_list {
        let (lat, lon) = match (ac.lat, ac.lon) {
            (Some(la), Some(lo)) => (la, lo),
            _ => continue,
        };

        // alt_baro can be a number or "ground"
        let baro_altitude = ac.alt_baro.as_ref().and_then(|v| {
            v.as_f64().map(|ft| ft * 0.3048) // feet → meters
        });
        let on_ground = ac
            .alt_baro
            .as_ref()
            .and_then(|v| v.as_str())
            .map(|s| s == "ground")
            .unwrap_or(false);

        aircraft.push(AircraftState {
            icao24: ac.hex.clone().unwrap_or_else(|| "unknown".to_string()),
            callsign: ac.flight.as_ref().map(|s| s.trim().to_string()),
            lat,
            lon,
            baro_altitude,
            geo_altitude: ac.alt_geom.map(|ft| ft * 0.3048),
            velocity: ac.gs.map(|kts| kts * 0.514444), // knots → m/s
            track: ac.track,
            vertical_rate: ac
                .geom_rate
                .or(ac.baro_rate)
                .map(|fpm| fpm * 0.00508), // ft/min → m/s
            on_ground,
        });
    }

    Ok(aircraft)
}

/// Extract altitude series for a specific aircraft from a list of states.
/// For time-series analysis, call `fetch_states` repeatedly and accumulate.
pub fn extract_altitudes(icao24: &str, states: &[AircraftState]) -> Vec<f64> {
    states
        .iter()
        .filter(|s| s.icao24 == icao24)
        .filter_map(|s| s.baro_altitude.or(s.geo_altitude))
        .collect()
}
