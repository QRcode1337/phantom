//! Open-Meteo weather API client
//!
//! Fetches temperature and precipitation forecasts from Open-Meteo's free API.
//! Ensemble endpoint provides multi-model spread (GFS vs ECMWF) — the
//! disagreement between models is the weather detector's trading edge.

use anyhow::{bail, Result};
use serde::Deserialize;

use super::http_client;

const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
const ENSEMBLE_URL: &str = "https://ensemble-api.open-meteo.com/v1/ensemble";

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    hourly: Option<HourlyData>,
}

#[derive(Debug, Deserialize)]
struct HourlyData {
    temperature_2m: Option<Vec<Option<f64>>>,
    precipitation: Option<Vec<Option<f64>>>,
}

/// Ensemble forecast with per-model data for spread calculation.
#[derive(Debug, Clone)]
pub struct EnsembleForecast {
    pub lat: f64,
    pub lon: f64,
    pub temperature_2m: Vec<f64>,
    pub precipitation: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct EnsembleResponse {
    latitude: Option<f64>,
    longitude: Option<f64>,
    hourly: Option<HourlyData>,
}

/// Fetch hourly temperature forecast (°C) for a location.
pub async fn fetch_temperature(lat: f64, lon: f64, hours: usize) -> Result<Vec<f64>> {
    let client = http_client();
    let url = format!(
        "{}?latitude={}&longitude={}&hourly=temperature_2m&forecast_hours={}&temperature_unit=fahrenheit",
        FORECAST_URL, lat, lon, hours
    );

    let resp: ForecastResponse = client.get(&url).send().await?.json().await?;

    let temps = resp
        .hourly
        .and_then(|h| h.temperature_2m)
        .ok_or_else(|| anyhow::anyhow!("no temperature data in response"))?;

    let series: Vec<f64> = temps.into_iter().filter_map(|v| v).collect();
    if series.is_empty() {
        bail!("empty temperature series from Open-Meteo");
    }

    Ok(series)
}

/// Fetch hourly precipitation forecast (mm) for a location.
pub async fn fetch_precipitation(lat: f64, lon: f64, hours: usize) -> Result<Vec<f64>> {
    let client = http_client();
    let url = format!(
        "{}?latitude={}&longitude={}&hourly=precipitation&forecast_hours={}",
        FORECAST_URL, lat, lon, hours
    );

    let resp: ForecastResponse = client.get(&url).send().await?.json().await?;

    let precip = resp
        .hourly
        .and_then(|h| h.precipitation)
        .ok_or_else(|| anyhow::anyhow!("no precipitation data in response"))?;

    let series: Vec<f64> = precip.into_iter().filter_map(|v| v).collect();
    if series.is_empty() {
        bail!("empty precipitation series from Open-Meteo");
    }

    Ok(series)
}

/// Fetch ensemble forecast from multiple models (GFS + ECMWF).
/// The spread across ensemble members indicates forecast uncertainty —
/// high spread = models disagree = potential mispricing in weather markets.
pub async fn fetch_ensemble(lat: f64, lon: f64) -> Result<EnsembleForecast> {
    let client = http_client();
    let url = format!(
        "{}?latitude={}&longitude={}&models=gfs_seamless,ecmwf_ifs025&hourly=temperature_2m,precipitation&temperature_unit=fahrenheit",
        ENSEMBLE_URL, lat, lon
    );

    let resp: EnsembleResponse = client.get(&url).send().await?.json().await?;

    let hourly = resp
        .hourly
        .ok_or_else(|| anyhow::anyhow!("no hourly data in ensemble response"))?;

    let temperature_2m: Vec<f64> = hourly
        .temperature_2m
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v)
        .collect();

    let precipitation: Vec<f64> = hourly
        .precipitation
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v)
        .collect();

    Ok(EnsembleForecast {
        lat: resp.latitude.unwrap_or(lat),
        lon: resp.longitude.unwrap_or(lon),
        temperature_2m,
        precipitation,
    })
}
