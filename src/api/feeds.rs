use axum::{Json, response::IntoResponse, http::StatusCode, extract::Query};
use serde::{Deserialize, Serialize};
use crate::feeds;
use crate::feeds::kalshi::KalshiClient;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

#[derive(Debug, Serialize)]
pub struct FeedResponse {
    pub series: Vec<f64>,
    pub source: String,
    pub points: usize,
}

#[derive(Debug, Deserialize)]
pub struct WeatherQuery {
    pub lat: f64,
    pub lon: f64,
    pub hours: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SeismicQuery {
    pub period: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BtcQuery {
    pub days: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct OpenSkyQuery {
    pub lamin: Option<f64>,
    pub lomin: Option<f64>,
    pub lamax: Option<f64>,
    pub lomax: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct PriceQuery {
    pub asset: String,
    pub days: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct KalshiQuery {
    pub series: String,
}

/// Allowed CoinGecko coin IDs for the generic price endpoint.
const ALLOWED_ASSETS: &[&str] = &["bitcoin", "ethereum", "solana"];

// Simple in-memory cache
struct CacheEntry {
    data: serde_json::Value,
    expiry: Instant,
}

struct FeedCache {
    entries: HashMap<String, CacheEntry>,
}

lazy_static::lazy_static! {
    static ref CACHE: Arc<Mutex<FeedCache>> = Arc::new(Mutex::new(FeedCache { entries: HashMap::new() }));
}

async fn get_from_cache_or_fetch<F, Fut>(key: String, fetch: F) -> Result<serde_json::Value, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<serde_json::Value, String>>,
{
    {
        let cache = CACHE.lock().unwrap();
        if let Some(entry) = cache.entries.get(&key) {
            if entry.expiry > Instant::now() {
                return Ok(entry.data.clone());
            }
        }
    }

    let data = fetch().await?;
    
    {
        let mut cache = CACHE.lock().unwrap();
        cache.entries.insert(key, CacheEntry {
            data: data.clone(),
            expiry: Instant::now() + Duration::from_secs(60),
        });
    }

    Ok(data)
}

pub async fn health() -> impl IntoResponse {
    let kalshi_status = if KalshiClient::new().is_ok() {
        "available"
    } else {
        "unavailable (credentials not configured)"
    };

    (StatusCode::OK, Json(serde_json::json!({
        "status": "ok",
        "feeds": {
            "weather": "available",
            "seismic": "available",
            "btc": "available",
            "price": "available",
            "kalshi": kalshi_status,
            "opensky": "available"
        }
    })))
}

pub async fn get_weather(Query(q): Query<WeatherQuery>) -> impl IntoResponse {
    let key = format!("weather-{}-{}-{}", q.lat, q.lon, q.hours.unwrap_or(168));
    let result = get_from_cache_or_fetch(key, || async move {
        match feeds::open_meteo::fetch_temperature(q.lat, q.lon, q.hours.unwrap_or(168)).await {
            Ok(series) => {
                let resp = FeedResponse {
                    series,
                    source: "open-meteo".to_string(),
                    points: 0, // will be set below
                };
                serde_json::to_value(resp).map_err(|e| e.to_string())
            },
            Err(e) => Err(e.to_string()),
        }
    }).await;

    match result {
        Ok(mut val) => {
            let points_count = val.get("series")
                .and_then(|s| s.as_array())
                .map(|a| a.len());

            if let (Some(p), Some(points_val)) = (points_count, val.get_mut("points")) {
                *points_val = serde_json::Value::from(p);
            }
            (StatusCode::OK, Json(val)).into_response()
        },
        Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e }))).into_response(),
    }
}

pub async fn get_seismic(Query(q): Query<SeismicQuery>) -> impl IntoResponse {
    let period = q.period.unwrap_or_else(|| "all_day".to_string());
    let valid_periods = ["all_hour", "all_day", "all_week", "all_month", "significant_day", "significant_week", "significant_month"];
    if !valid_periods.contains(&period.as_str()) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("Invalid seismic period '{}'. Valid: {:?}", period, valid_periods) }))).into_response();
    }

    let key = format!("seismic-{}", period);
    let result = get_from_cache_or_fetch(key, || async move {
        match feeds::usgs::fetch_magnitudes(&period).await {
            Ok(series) => {
                let resp = FeedResponse {
                    series,
                    source: "usgs".to_string(),
                    points: 0,
                };
                serde_json::to_value(resp).map_err(|e| e.to_string())
            },
            Err(e) => Err(e.to_string()),
        }
    }).await;

    match result {
        Ok(mut val) => {
            let points_count = val.get("series")
                .and_then(|s| s.as_array())
                .map(|a| a.len());

            if let (Some(p), Some(points_val)) = (points_count, val.get_mut("points")) {
                *points_val = serde_json::Value::from(p);
            }
            (StatusCode::OK, Json(val)).into_response()
        },
        Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e }))).into_response(),
    }
}

pub async fn get_btc(Query(q): Query<BtcQuery>) -> impl IntoResponse {
    let days = q.days.unwrap_or(30);
    let key = format!("btc-{}", days);
    let result = get_from_cache_or_fetch(key, || async move {
        match feeds::prices::fetch_btc_price_history(days).await {
            Ok(series) => {
                let resp = FeedResponse {
                    series,
                    source: "coingecko".to_string(),
                    points: 0,
                };
                serde_json::to_value(resp).map_err(|e| e.to_string())
            },
            Err(e) => Err(e.to_string()),
        }
    }).await;

    match result {
        Ok(mut val) => {
            let points_count = val.get("series")
                .and_then(|s| s.as_array())
                .map(|a| a.len());

            if let (Some(p), Some(points_val)) = (points_count, val.get_mut("points")) {
                *points_val = serde_json::Value::from(p);
            }
            (StatusCode::OK, Json(val)).into_response()
        },
        Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e }))).into_response(),
    }
}

pub async fn get_opensky(Query(q): Query<OpenSkyQuery>) -> impl IntoResponse {
    // Build bounding box from query params if all four are provided
    let bbox = match (q.lamin, q.lomin, q.lamax, q.lomax) {
        (Some(lamin), Some(lomin), Some(lamax), Some(lomax)) => Some((lamin, lomin, lamax, lomax)),
        _ => None,
    };

    let key = match &bbox {
        Some((a, b, c, d)) => format!("opensky-{}-{}-{}-{}", a, b, c, d),
        None => "opensky-all".to_string(),
    };

    let result = get_from_cache_or_fetch(key, || async move {
        match feeds::opensky::fetch_states(bbox).await {
            Ok(states) => {
                let count = states.len();
                serde_json::to_value(serde_json::json!({
                    "states": states,
                    "count": count
                })).map_err(|e| e.to_string())
            },
            Err(e) => Err(e.to_string()),
        }
    }).await;

    match result {
        Ok(val) => (StatusCode::OK, Json(val)).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e }))).into_response(),
    }
}

// ── GET /api/feeds/price ─────────────────────────────────────────────────────

pub async fn get_price(Query(q): Query<PriceQuery>) -> impl IntoResponse {
    let asset = q.asset.to_lowercase();
    if !ALLOWED_ASSETS.contains(&asset.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!(
                    "Unsupported asset '{}'. Allowed: {:?}",
                    asset, ALLOWED_ASSETS
                )
            })),
        )
            .into_response();
    }

    let days = q.days.unwrap_or(30);
    let key = format!("price-{}-{}", asset, days);
    let result = get_from_cache_or_fetch(key, || {
        let asset = asset.clone();
        async move {
            match feeds::prices::fetch_exchange_price(&asset, days).await {
                Ok(series) => {
                    let resp = FeedResponse {
                        series,
                        source: "coingecko".to_string(),
                        points: 0,
                    };
                    serde_json::to_value(resp).map_err(|e| e.to_string())
                }
                Err(e) => Err(e.to_string()),
            }
        }
    })
    .await;

    match result {
        Ok(mut val) => {
            let points_count = val
                .get("series")
                .and_then(|s| s.as_array())
                .map(|a| a.len());

            if let (Some(p), Some(points_val)) = (points_count, val.get_mut("points")) {
                *points_val = serde_json::Value::from(p);
            }
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => {
            (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e }))).into_response()
        }
    }
}

// ── GET /api/feeds/kalshi ────────────────────────────────────────────────────

pub async fn get_kalshi_markets(Query(q): Query<KalshiQuery>) -> impl IntoResponse {
    let client = match KalshiClient::new() {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Kalshi credentials are not configured. Set KALSHI_API_KEY_ID and KALSHI_PRIVATE_KEY_PATH environment variables, or create ~/.config/kalshi/credentials.json."
                })),
            )
                .into_response();
        }
    };

    let key = format!("kalshi-series-{}", q.series);
    let series = q.series.clone();
    let result = get_from_cache_or_fetch(key, || async move {
        match client.get_markets(&series).await {
            Ok(markets) => {
                let count = markets.len();
                serde_json::to_value(serde_json::json!({
                    "markets": markets,
                    "count": count,
                    "source": "kalshi"
                }))
                .map_err(|e| e.to_string())
            }
            Err(e) => Err(e.to_string()),
        }
    })
    .await;

    match result {
        Ok(val) => (StatusCode::OK, Json(val)).into_response(),
        Err(e) => {
            (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e }))).into_response()
        }
    }
}
