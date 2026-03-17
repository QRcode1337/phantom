//! Monitoring metrics endpoint for the PHANTOM signal engine.
//!
//! Route (registered in api/mod.rs):
//!   GET /api/metrics — aggregated signal stats, performance averages,
//!                      feed health, and server uptime

use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::{Duration, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

use super::signals_db::SignalStore;

// ─── Server start time ──────────────────────────────────────────────────────

lazy_static::lazy_static! {
    static ref SERVER_START: Instant = Instant::now();
}

/// Force the lazy_static to initialise.  Call this once from `start_server`
/// so uptime is measured from actual boot rather than first request.
pub fn init_start_time() {
    lazy_static::initialize(&SERVER_START);
}

// ─── Response types ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub signals: SignalMetrics,
    pub performance: PerformanceMetrics,
    pub feeds: HashMap<String, String>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct SignalMetrics {
    pub total: usize,
    pub by_action: HashMap<String, usize>,
    pub by_type: HashMap<String, usize>,
    pub last_24h: usize,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub avg_edge_enter: f64,
    pub avg_chaos_enter: f64,
    pub avg_edge_watch: f64,
}

// ─── Feed health check ─────────────────────────────────────────────────────

/// Ping a URL with a 3-second timeout.  Returns `"ok"` on any 2xx response
/// and `"error"` otherwise (including timeouts and connection failures).
async fn check_feed(url: &str) -> String {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => "ok".to_string(),
        _ => "error".to_string(),
    }
}

// ─── GET /api/metrics ───────────────────────────────────────────────────────

pub async fn get_metrics() -> impl IntoResponse {
    let store = SignalStore::new();

    // Load all signals
    let all_signals = match store.load_signals() {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to read signals: {}", e)
                })),
            )
                .into_response()
        }
    };

    let total = all_signals.len();

    // ── Group by action ─────────────────────────────────────────────────────
    let mut by_action: HashMap<String, usize> = HashMap::new();
    for sig in &all_signals {
        *by_action.entry(sig.action.clone()).or_insert(0) += 1;
    }

    // ── Group by signal_type ────────────────────────────────────────────────
    let mut by_type: HashMap<String, usize> = HashMap::new();
    for sig in &all_signals {
        *by_type.entry(sig.signal_type.clone()).or_insert(0) += 1;
    }

    // ── Last 24 hours ───────────────────────────────────────────────────────
    let cutoff = Utc::now() - Duration::hours(24);
    let last_24h = all_signals
        .iter()
        .filter(|s| s.timestamp >= cutoff)
        .count();

    // ── Performance averages ────────────────────────────────────────────────
    let (mut sum_edge_enter, mut sum_chaos_enter, mut count_enter) = (0.0_f64, 0.0_f64, 0_usize);
    let (mut sum_edge_watch, mut count_watch) = (0.0_f64, 0_usize);

    for sig in &all_signals {
        match sig.action.as_str() {
            "ENTER" => {
                sum_edge_enter += sig.edge;
                sum_chaos_enter += sig.chaos_score;
                count_enter += 1;
            }
            "WATCH" => {
                sum_edge_watch += sig.edge;
                count_watch += 1;
            }
            _ => {}
        }
    }

    let avg_edge_enter = if count_enter > 0 {
        sum_edge_enter / count_enter as f64
    } else {
        0.0
    };
    let avg_chaos_enter = if count_enter > 0 {
        sum_chaos_enter / count_enter as f64
    } else {
        0.0
    };
    let avg_edge_watch = if count_watch > 0 {
        sum_edge_watch / count_watch as f64
    } else {
        0.0
    };

    // ── Feed health (concurrent pings) ──────────────────────────────────────
    let (weather, seismic, btc, opensky) = tokio::join!(
        check_feed("https://api.open-meteo.com/v1/forecast?latitude=0&longitude=0&hourly=temperature_2m&forecast_hours=1"),
        check_feed("https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_hour.geojson"),
        check_feed("https://api.coingecko.com/api/v3/ping"),
        check_feed("https://opensky-network.org/api/states/all?lamin=0&lomin=0&lamax=1&lomax=1"),
    );

    let mut feeds = HashMap::new();
    feeds.insert("weather".to_string(), weather);
    feeds.insert("seismic".to_string(), seismic);
    feeds.insert("btc".to_string(), btc);
    feeds.insert("opensky".to_string(), opensky);

    // ── Uptime ──────────────────────────────────────────────────────────────
    let uptime_seconds = SERVER_START.elapsed().as_secs();

    (
        StatusCode::OK,
        Json(MetricsResponse {
            signals: SignalMetrics {
                total,
                by_action,
                by_type,
                last_24h,
            },
            performance: PerformanceMetrics {
                avg_edge_enter,
                avg_chaos_enter,
                avg_edge_watch,
            },
            feeds,
            uptime_seconds,
        }),
    )
        .into_response()
}
