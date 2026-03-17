//! HTTP handler for ARGUS anomaly analysis with optional webhook notification.
//!
//! Route (registered in api/mod.rs):
//!   POST /api/argus/analyze — run seismic + flight anomaly detection,
//!                              optionally POST critical anomalies to a webhook URL.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::detectors::argus::{ArgusAnomaly, ArgusDetector};
use crate::feeds;

// ─── Request / Response types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ArgusAnalyzeRequest {
    /// USGS feed period. Defaults to "all_day".
    /// Options: "all_hour", "all_day", "all_week", "all_month",
    ///          "significant_day", "significant_week", "significant_month"
    pub seismic_period: Option<String>,
    /// If set, critical anomalies are POSTed to this URL.
    /// Must start with "http://" or "https://".
    pub webhook_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ArgusAnomalyDto {
    pub feed: String,
    pub severity: String,
    pub chaos_score: f64,
    pub regime_changed: bool,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct ArgusAnalyzeResponse {
    pub anomalies: Vec<ArgusAnomalyDto>,
    pub critical_count: usize,
    pub webhook_sent: bool,
    pub total_seismic_events: usize,
    pub total_flights_analyzed: usize,
    pub errors: Vec<String>,
}

// ─── POST /api/argus/analyze ─────────────────────────────────────────────────

pub async fn analyze_argus(
    Json(payload): Json<ArgusAnalyzeRequest>,
) -> impl IntoResponse {
    let period = payload
        .seismic_period
        .as_deref()
        .unwrap_or("all_day");

    // Validate webhook URL if provided
    if let Some(ref url) = payload.webhook_url {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "webhook_url must start with http:// or https://"
                })),
            )
                .into_response();
        }
    }

    let detector = ArgusDetector::new();
    let mut anomalies: Vec<ArgusAnomalyDto> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut total_seismic_events: usize = 0;
    let mut total_flights_analyzed: usize = 0;

    // ── Seismic analysis ─────────────────────────────────────────────────────
    match feeds::usgs::fetch_magnitudes(period).await {
        Ok(magnitudes) => {
            total_seismic_events = magnitudes.len();
            match detector.analyze_seismic_series(&magnitudes) {
                Ok(anomaly) => anomalies.push(anomaly_to_dto(&anomaly)),
                Err(e) => errors.push(format!("seismic analysis: {}", e)),
            }
        }
        Err(e) => errors.push(format!("seismic fetch: {}", e)),
    }

    // ── Flight analysis ──────────────────────────────────────────────────────
    match feeds::opensky::fetch_states(None).await {
        Ok(states) => {
            // Collect unique ICAO24 identifiers for airborne aircraft
            let mut seen = std::collections::HashSet::new();
            let airborne: Vec<&str> = states
                .iter()
                .filter(|s| !s.on_ground && s.baro_altitude.or(s.geo_altitude).is_some())
                .filter(|s| seen.insert(s.icao24.as_str()))
                .map(|s| s.icao24.as_str())
                .take(5) // sample up to 5 aircraft
                .collect();

            for icao in &airborne {
                let altitudes = feeds::opensky::extract_altitudes(icao, &states);
                if altitudes.is_empty() {
                    continue;
                }

                let feed_id = format!("opensky-flight-{}", icao);
                match detector.analyze_flight_track(&feed_id, &altitudes) {
                    Ok(anomaly) => {
                        total_flights_analyzed += 1;
                        anomalies.push(anomaly_to_dto(&anomaly));
                    }
                    Err(e) => errors.push(format!("flight {} analysis: {}", icao, e)),
                }
            }
        }
        Err(e) => errors.push(format!("flight fetch: {}", e)),
    }

    // ── Webhook dispatch for critical anomalies ──────────────────────────────
    let critical_count = anomalies
        .iter()
        .filter(|a| a.severity == "Critical")
        .count();

    let mut webhook_sent = false;

    if critical_count > 0 {
        if let Some(ref url) = payload.webhook_url {
            let critical_anomalies: Vec<&ArgusAnomalyDto> = anomalies
                .iter()
                .filter(|a| a.severity == "Critical")
                .collect();

            let webhook_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            match webhook_client
                .post(url)
                .json(&serde_json::json!({
                    "source": "phantom-argus",
                    "critical_count": critical_count,
                    "anomalies": critical_anomalies,
                }))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    webhook_sent = true;
                }
                Ok(resp) => {
                    errors.push(format!(
                        "webhook returned non-success status: {}",
                        resp.status()
                    ));
                }
                Err(e) => {
                    errors.push(format!("webhook POST failed: {}", e));
                }
            }
        }
    }

    // ── Response ─────────────────────────────────────────────────────────────
    let status = if anomalies.is_empty() && !errors.is_empty() {
        StatusCode::BAD_GATEWAY
    } else {
        StatusCode::OK
    };

    (
        status,
        Json(ArgusAnalyzeResponse {
            anomalies,
            critical_count,
            webhook_sent,
            total_seismic_events,
            total_flights_analyzed,
            errors,
        }),
    )
        .into_response()
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn anomaly_to_dto(a: &ArgusAnomaly) -> ArgusAnomalyDto {
    ArgusAnomalyDto {
        feed: a.feed.clone(),
        severity: format!("{:?}", a.severity),
        chaos_score: a.chaos_score,
        regime_changed: a.regime_changed,
        description: a.description.clone(),
    }
}
