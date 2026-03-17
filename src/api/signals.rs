//! HTTP handlers for signal persistence endpoints.
//!
//! Routes (registered in api/mod.rs):
//!   POST /api/signals/record    — manually persist a signal
//!   GET  /api/signals           — query stored signals (with optional filters)
//!   POST /api/signals/analyze   — run the full KalshiSignalEngine pipeline on
//!                                 the latest live feeds and persist results

use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::signals::KalshiSignalEngine;
use crate::signals::kalshi::{KalshiSignal, SignalAction};
use crate::feeds;
use super::signals_db::{OutcomeResult, SignalOutcome, SignalRecord, SignalStore};

// ─── POST /api/signals/record ─────────────────────────────────────────────────

/// Request body accepted by `POST /api/signals/record`.
/// All fields mirror `SignalRecord` except `timestamp` which defaults to now.
#[derive(Debug, Deserialize)]
pub struct RecordRequest {
    pub signal_type: String,
    pub action: String,
    pub edge: f64,
    pub chaos_score: f64,
    pub market_type: String,
    pub direction: String,
    pub confidence: String,
    pub reason: String,
    /// Optional override; defaults to `Utc::now()` when absent.
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct RecordResponse {
    pub stored: bool,
    pub total_count: usize,
    pub record: SignalRecord,
}

pub async fn record_signal(Json(payload): Json<RecordRequest>) -> impl IntoResponse {
    let record = SignalRecord {
        timestamp: payload.timestamp.unwrap_or_else(Utc::now),
        signal_type: payload.signal_type,
        action: payload.action,
        edge: payload.edge,
        chaos_score: payload.chaos_score,
        market_type: payload.market_type,
        direction: payload.direction,
        confidence: payload.confidence,
        reason: payload.reason,
        outcome: None,
    };

    let store = SignalStore::new();

    if let Err(e) = store.store_signal(&record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to persist signal: {}", e) })),
        )
            .into_response();
    }

    let total_count = store.count().unwrap_or(0);

    (
        StatusCode::CREATED,
        Json(RecordResponse {
            stored: true,
            total_count,
            record,
        }),
    )
        .into_response()
}

// ─── GET /api/signals ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SignalsQuery {
    /// ISO-8601 UTC timestamp; only records at or after this time are returned.
    pub since: Option<DateTime<Utc>>,
    /// Filter by action string, e.g. "ENTER", "WATCH", "SKIP".
    pub action: Option<String>,
    /// Maximum number of records to return (most-recent first). Defaults to 200.
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SignalsResponse {
    pub records: Vec<SignalRecord>,
    pub count: usize,
    pub total_stored: usize,
}

pub async fn query_signals(Query(q): Query<SignalsQuery>) -> impl IntoResponse {
    let store = SignalStore::new();

    let all = match q.since {
        Some(since) => store.load_signals_since(since),
        None => store.load_signals(),
    };

    let all = match all {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read signals: {}", e) })),
            )
                .into_response()
        }
    };

    let total_stored = store.count().unwrap_or(all.len());

    // Apply action filter
    let mut filtered: Vec<SignalRecord> = match &q.action {
        Some(action_filter) => {
            let upper = action_filter.to_uppercase();
            all.into_iter()
                .filter(|r| r.action.to_uppercase() == upper)
                .collect()
        }
        None => all,
    };

    // Most-recent first
    filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Apply limit (default 200)
    let limit = q.limit.unwrap_or(200);
    filtered.truncate(limit);

    let count = filtered.len();

    (
        StatusCode::OK,
        Json(SignalsResponse {
            records: filtered,
            count,
            total_stored,
        }),
    )
        .into_response()
}

// ─── POST /api/signals/analyze ────────────────────────────────────────────────
//
// Runs the full KalshiSignalEngine on the latest live feed data, persists
// any Enter or Watch signals, and returns them.  Skip signals are returned
// but not persisted (they carry no actionable information for backtesting).

#[derive(Debug, Deserialize)]
pub struct AnalyzeSignalsRequest {
    /// NYC lat/lon for weather feed; defaults to 40.7128 / -74.0060.
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    /// Hours of temperature history to fetch (default 168 = 7 days).
    pub weather_hours: Option<usize>,
    /// Weather market threshold in °F (default 62.0).
    pub weather_target: Option<f64>,
    /// Whether market is "above" target (default true).
    pub weather_above: Option<bool>,
    /// BTC price history days (default 30).
    pub btc_days: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeSignalsResponse {
    pub signals: Vec<PersistableSignal>,
    pub persisted_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PersistableSignal {
    pub record: SignalRecord,
    pub persisted: bool,
}

pub async fn analyze_and_store(
    Json(payload): Json<AnalyzeSignalsRequest>,
) -> impl IntoResponse {
    let engine = KalshiSignalEngine::new();
    let store = SignalStore::new();
    let now = Utc::now();

    let lat = payload.lat.unwrap_or(40.7128);
    let lon = payload.lon.unwrap_or(-74.0060);
    let hours = payload.weather_hours.unwrap_or(168);
    let target = payload.weather_target.unwrap_or(62.0);
    let above = payload.weather_above.unwrap_or(true);
    let btc_days = payload.btc_days.unwrap_or(30);

    let mut results: Vec<PersistableSignal> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // ── Weather signals ───────────────────────────────────────────────────────
    match feeds::open_meteo::fetch_temperature(lat, lon, hours).await {
        Ok(temps) => {
            // Primary weather edge signal
            match engine.weather_signal(&temps, target, above) {
                Ok(sig) => {
                    let record = kalshi_signal_to_record(&sig, "weather/temperature", now);
                    let should_persist = sig.action != SignalAction::Skip;
                    if should_persist {
                        if let Err(e) = store.store_signal(&record) {
                            errors.push(format!("persist weather signal: {}", e));
                        }
                    }
                    results.push(PersistableSignal {
                        persisted: should_persist,
                        record,
                    });
                }
                Err(e) => errors.push(format!("weather signal: {}", e)),
            }

            // Regime shift signal (optional)
            match engine.regime_shift_signal(&temps) {
                Ok(Some(sig)) => {
                    let record = kalshi_signal_to_record(&sig, "weather/regime-shift", now);
                    let should_persist = sig.action != SignalAction::Skip;
                    if should_persist {
                        if let Err(e) = store.store_signal(&record) {
                            errors.push(format!("persist regime-shift signal: {}", e));
                        }
                    }
                    results.push(PersistableSignal {
                        persisted: should_persist,
                        record,
                    });
                }
                Ok(None) => {}
                Err(e) => errors.push(format!("regime shift signal: {}", e)),
            }
        }
        Err(e) => errors.push(format!("weather fetch: {}", e)),
    }

    // ── BTC price signal ──────────────────────────────────────────────────────
    match feeds::prices::fetch_btc_price_history(btc_days).await {
        Ok(prices) => {
            match engine.price_regime_signal("BTC-USD", &prices) {
                Ok(sig) => {
                    let record = kalshi_signal_to_record(&sig, "price/BTC-USD", now);
                    let should_persist = sig.action != SignalAction::Skip;
                    if should_persist {
                        if let Err(e) = store.store_signal(&record) {
                            errors.push(format!("persist BTC signal: {}", e));
                        }
                    }
                    results.push(PersistableSignal {
                        persisted: should_persist,
                        record,
                    });
                }
                Err(e) => errors.push(format!("BTC price signal: {}", e)),
            }
        }
        Err(e) => errors.push(format!("BTC fetch: {}", e)),
    }

    let persisted_count = results.iter().filter(|r| r.persisted).count();
    let status = if results.is_empty() && !errors.is_empty() {
        StatusCode::BAD_GATEWAY
    } else {
        StatusCode::OK
    };

    (
        status,
        Json(AnalyzeSignalsResponse {
            signals: results,
            persisted_count,
            errors,
        }),
    )
        .into_response()
}

// ─── Conversion helper ────────────────────────────────────────────────────────

fn kalshi_signal_to_record(
    sig: &KalshiSignal,
    signal_type: &str,
    timestamp: DateTime<Utc>,
) -> SignalRecord {
    SignalRecord {
        timestamp,
        signal_type: signal_type.to_string(),
        action: format!("{:?}", sig.action).to_uppercase(),
        edge: sig.edge,
        chaos_score: sig.chaos_score,
        market_type: sig.market_type.clone(),
        direction: sig.direction.clone(),
        confidence: sig.confidence.clone(),
        reason: sig.reason.clone(),
        outcome: None,
    }
}

// ─── PUT /api/signals/resolve ────────────────────────────────────────────────

/// Request body accepted by `PUT /api/signals/resolve`.
#[derive(Debug, Deserialize)]
pub struct ResolveRequest {
    /// ISO-8601 UTC timestamp identifying the signal to resolve.
    pub timestamp: DateTime<Utc>,
    /// One of "Win", "Loss", "Scratch", "Expired".
    pub result: String,
    /// Realised profit/loss (optional).
    pub pnl: Option<f64>,
    /// Free-form notes (optional).
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResolveResponse {
    pub resolved: bool,
    pub timestamp: DateTime<Utc>,
    pub result: String,
}

pub async fn resolve_signal(Json(payload): Json<ResolveRequest>) -> impl IntoResponse {
    let result = match payload.result.to_lowercase().as_str() {
        "win" => OutcomeResult::Win,
        "loss" => OutcomeResult::Loss,
        "scratch" => OutcomeResult::Scratch,
        "expired" => OutcomeResult::Expired,
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!(
                        "Invalid result '{}'. Expected one of: Win, Loss, Scratch, Expired",
                        other
                    )
                })),
            )
                .into_response();
        }
    };

    let outcome = SignalOutcome {
        resolved_at: Utc::now(),
        result: result.clone(),
        pnl: payload.pnl,
        notes: payload.notes,
    };

    let store = SignalStore::new();

    match store.resolve_signal(payload.timestamp, outcome) {
        Ok(true) => (
            StatusCode::OK,
            Json(ResolveResponse {
                resolved: true,
                timestamp: payload.timestamp,
                result: format!("{:?}", result),
            }),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "No signal found with the given timestamp"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to resolve signal: {}", e)
            })),
        )
            .into_response(),
    }
}

// ─── GET /api/signals/summary ────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SignalsSummary {
    pub total_signals: usize,
    pub resolved: usize,
    pub unresolved: usize,
    pub wins: usize,
    pub losses: usize,
    pub scratches: usize,
    pub expired: usize,
    pub win_rate: Option<f64>,
    pub avg_edge_wins: Option<f64>,
    pub avg_edge_losses: Option<f64>,
    pub total_pnl: f64,
}

pub async fn signals_summary() -> impl IntoResponse {
    let store = SignalStore::new();

    let records = match store.load_signals() {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to read signals: {}", e)
                })),
            )
                .into_response();
        }
    };

    let total_signals = records.len();

    let mut wins = 0usize;
    let mut losses = 0usize;
    let mut scratches = 0usize;
    let mut expired = 0usize;
    let mut resolved = 0usize;
    let mut total_pnl = 0.0f64;
    let mut win_edges: Vec<f64> = Vec::new();
    let mut loss_edges: Vec<f64> = Vec::new();

    for record in &records {
        if let Some(ref outcome) = record.outcome {
            resolved += 1;
            if let Some(pnl) = outcome.pnl {
                total_pnl += pnl;
            }
            match outcome.result {
                OutcomeResult::Win => {
                    wins += 1;
                    win_edges.push(record.edge);
                }
                OutcomeResult::Loss => {
                    losses += 1;
                    loss_edges.push(record.edge);
                }
                OutcomeResult::Scratch => scratches += 1,
                OutcomeResult::Expired => expired += 1,
            }
        }
    }

    let decisive = wins + losses;
    let win_rate = if decisive > 0 {
        Some(wins as f64 / decisive as f64)
    } else {
        None
    };

    let avg_edge_wins = if win_edges.is_empty() {
        None
    } else {
        Some(win_edges.iter().sum::<f64>() / win_edges.len() as f64)
    };

    let avg_edge_losses = if loss_edges.is_empty() {
        None
    } else {
        Some(loss_edges.iter().sum::<f64>() / loss_edges.len() as f64)
    };

    (
        StatusCode::OK,
        Json(SignalsSummary {
            total_signals,
            resolved,
            unresolved: total_signals - resolved,
            wins,
            losses,
            scratches,
            expired,
            win_rate,
            avg_edge_wins,
            avg_edge_losses,
            total_pnl,
        }),
    )
        .into_response()
}
