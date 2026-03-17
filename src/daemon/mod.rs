//! Background feed polling daemon for Phantom.
//!
//! Spawns `tokio` tasks that periodically fetch external data, run the signal
//! engine, and persist actionable signals — all without blocking the Axum
//! HTTP server.
//!
//! # Architecture
//!
//! Three independent polling loops run concurrently:
//! - **Weather** (30 min default): Open-Meteo temps → weather & regime-shift signals
//! - **Price** (5 min default): CoinGecko BTC → price regime signals
//! - **Seismic** (10 min default): USGS magnitudes → ARGUS anomaly analysis
//!
//! Each loop accumulates data into a shared [`FeedBuffer`] so detectors always
//! operate on the full rolling window, not just the latest fetch.

pub mod feed_buffer;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::RwLock;
use tokio::time;

use crate::api::signals_db::{SignalRecord, SignalStore};
use crate::detectors::argus::AnomalySeverity;
use crate::feeds;
use crate::signals::kalshi::SignalAction;
use crate::signals::KalshiSignalEngine;
use crate::ArgusDetector;

use feed_buffer::FeedBuffer;

// ─── Feed names (buffer keys) ────────────────────────────────────────────────

const FEED_WEATHER: &str = "weather";
const FEED_PRICE: &str = "price";
const FEED_SEISMIC: &str = "seismic";

// ─── Shared daemon state ─────────────────────────────────────────────────────

/// Metrics tracked by the daemon, queryable via the status endpoint.
#[derive(Debug)]
pub struct DaemonMetrics {
    pub last_weather_poll: RwLock<Option<DateTime<Utc>>>,
    pub last_price_poll: RwLock<Option<DateTime<Utc>>>,
    pub last_seismic_poll: RwLock<Option<DateTime<Utc>>>,
    pub signals_generated: AtomicU64,
    pub weather_errors: AtomicU64,
    pub price_errors: AtomicU64,
    pub seismic_errors: AtomicU64,
}

impl DaemonMetrics {
    fn new() -> Self {
        Self {
            last_weather_poll: RwLock::new(None),
            last_price_poll: RwLock::new(None),
            last_seismic_poll: RwLock::new(None),
            signals_generated: AtomicU64::new(0),
            weather_errors: AtomicU64::new(0),
            price_errors: AtomicU64::new(0),
            seismic_errors: AtomicU64::new(0),
        }
    }
}

/// JSON-serializable snapshot of daemon state for the status endpoint.
#[derive(Debug, Serialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub last_weather_poll: Option<DateTime<Utc>>,
    pub last_price_poll: Option<DateTime<Utc>>,
    pub last_seismic_poll: Option<DateTime<Utc>>,
    pub signals_generated: u64,
    pub weather_errors: u64,
    pub price_errors: u64,
    pub seismic_errors: u64,
    pub buffer_sizes: BufferSizes,
}

#[derive(Debug, Serialize)]
pub struct BufferSizes {
    pub weather: usize,
    pub price: usize,
    pub seismic: usize,
}

// ─── FeedDaemon ──────────────────────────────────────────────────────────────

/// Background task manager that polls feeds on independent schedules.
pub struct FeedDaemon {
    /// Polling interval for weather feeds (default: 30 minutes).
    pub weather_interval: Duration,
    /// Polling interval for price feeds (default: 5 minutes).
    pub price_interval: Duration,
    /// Polling interval for seismic feeds (default: 10 minutes).
    pub seismic_interval: Duration,
    /// Whether the daemon is running.
    running: Arc<AtomicBool>,
    /// Shared rolling buffer for all feeds.
    buffer: Arc<RwLock<FeedBuffer>>,
    /// Observable metrics.
    metrics: Arc<DaemonMetrics>,
}

impl FeedDaemon {
    /// Create a new daemon with default intervals.
    pub fn new() -> Self {
        Self {
            weather_interval: Duration::from_secs(30 * 60),  // 30 minutes
            price_interval: Duration::from_secs(5 * 60),     // 5 minutes
            seismic_interval: Duration::from_secs(10 * 60),  // 10 minutes
            running: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(RwLock::new(FeedBuffer::default())),
            metrics: Arc::new(DaemonMetrics::new()),
        }
    }

    /// Returns a handle to the shared buffer (for external read access).
    pub fn buffer(&self) -> Arc<RwLock<FeedBuffer>> {
        Arc::clone(&self.buffer)
    }

    /// Returns a handle to the daemon metrics (for the status endpoint).
    pub fn metrics(&self) -> Arc<DaemonMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Returns a handle to the running flag (for the status endpoint).
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    /// Spawn three background polling tasks. Returns immediately.
    ///
    /// Each task runs in its own `tokio::spawn` and uses `tokio::time::interval`
    /// for scheduling. A failed fetch never kills the loop — the error is logged
    /// and the task retries on the next interval tick.
    pub async fn start(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            tracing::warn!("daemon already running — ignoring duplicate start");
            return;
        }

        tracing::info!(
            weather_secs = self.weather_interval.as_secs(),
            price_secs = self.price_interval.as_secs(),
            seismic_secs = self.seismic_interval.as_secs(),
            "starting feed daemon"
        );

        // ── Weather task ─────────────────────────────────────────────────────
        {
            let running = Arc::clone(&self.running);
            let buffer = Arc::clone(&self.buffer);
            let metrics = Arc::clone(&self.metrics);
            let interval_dur = self.weather_interval;

            tokio::spawn(async move {
                let mut interval = time::interval(interval_dur);
                // The first tick fires immediately
                interval.tick().await;

                while running.load(Ordering::Relaxed) {
                    tracing::info!("daemon: weather poll starting");
                    Self::poll_weather(&buffer, &metrics).await;
                    interval.tick().await;
                }

                tracing::info!("daemon: weather task stopped");
            });
        }

        // ── Price task ───────────────────────────────────────────────────────
        {
            let running = Arc::clone(&self.running);
            let buffer = Arc::clone(&self.buffer);
            let metrics = Arc::clone(&self.metrics);
            let interval_dur = self.price_interval;

            tokio::spawn(async move {
                let mut interval = time::interval(interval_dur);
                interval.tick().await;

                while running.load(Ordering::Relaxed) {
                    tracing::info!("daemon: price poll starting");
                    Self::poll_prices(&buffer, &metrics).await;
                    interval.tick().await;
                }

                tracing::info!("daemon: price task stopped");
            });
        }

        // ── Seismic task ─────────────────────────────────────────────────────
        {
            let running = Arc::clone(&self.running);
            let buffer = Arc::clone(&self.buffer);
            let metrics = Arc::clone(&self.metrics);
            let interval_dur = self.seismic_interval;

            tokio::spawn(async move {
                let mut interval = time::interval(interval_dur);
                interval.tick().await;

                while running.load(Ordering::Relaxed) {
                    tracing::info!("daemon: seismic poll starting");
                    Self::poll_seismic(&buffer, &metrics).await;
                    interval.tick().await;
                }

                tracing::info!("daemon: seismic task stopped");
            });
        }
    }

    /// Signal all background tasks to stop after their current iteration.
    pub fn stop(&self) {
        tracing::info!("stopping feed daemon");
        self.running.store(false, Ordering::SeqCst);
    }

    /// Build a snapshot of the daemon status for the HTTP endpoint.
    pub async fn status(&self) -> DaemonStatus {
        let buf = self.buffer.read().await;
        DaemonStatus {
            running: self.running.load(Ordering::Relaxed),
            last_weather_poll: *self.metrics.last_weather_poll.read().await,
            last_price_poll: *self.metrics.last_price_poll.read().await,
            last_seismic_poll: *self.metrics.last_seismic_poll.read().await,
            signals_generated: self.metrics.signals_generated.load(Ordering::Relaxed),
            weather_errors: self.metrics.weather_errors.load(Ordering::Relaxed),
            price_errors: self.metrics.price_errors.load(Ordering::Relaxed),
            seismic_errors: self.metrics.seismic_errors.load(Ordering::Relaxed),
            buffer_sizes: BufferSizes {
                weather: buf.len(FEED_WEATHER),
                price: buf.len(FEED_PRICE),
                seismic: buf.len(FEED_SEISMIC),
            },
        }
    }

    // ── Private polling functions ────────────────────────────────────────────

    async fn poll_weather(
        buffer: &Arc<RwLock<FeedBuffer>>,
        metrics: &Arc<DaemonMetrics>,
    ) {
        let now = Utc::now();

        // Fetch NYC temperature data (200 hours of forecast)
        match feeds::open_meteo::fetch_temperature(40.7128, -74.006, 200).await {
            Ok(temps) => {
                tracing::info!(count = temps.len(), "daemon: fetched weather temps");

                // Accumulate into rolling buffer
                {
                    let mut buf = buffer.write().await;
                    buf.push(FEED_WEATHER, &temps);
                }

                // Read the full rolling window for analysis
                let series = {
                    let buf = buffer.read().await;
                    buf.snapshot(FEED_WEATHER).unwrap_or_default()
                };

                if series.is_empty() {
                    return;
                }

                let engine = KalshiSignalEngine::new();
                let store = SignalStore::new();

                // Primary weather signal
                match engine.weather_signal(&series, 62.0, true) {
                    Ok(sig) => {
                        tracing::info!(action = ?sig.action, edge = sig.edge, "daemon: weather signal");
                        if sig.action == SignalAction::Enter || sig.action == SignalAction::Watch {
                            let record = kalshi_signal_to_record(&sig, "weather/temperature", now);
                            if let Err(e) = store.store_signal(&record) {
                                tracing::warn!(error = %e, "daemon: failed to persist weather signal");
                            } else {
                                metrics.signals_generated.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "daemon: weather_signal analysis failed"),
                }

                // Regime shift signal
                match engine.regime_shift_signal(&series) {
                    Ok(Some(sig)) => {
                        tracing::info!(action = ?sig.action, "daemon: weather regime-shift signal");
                        if sig.action == SignalAction::Enter || sig.action == SignalAction::Watch {
                            let record = kalshi_signal_to_record(&sig, "weather/regime-shift", now);
                            if let Err(e) = store.store_signal(&record) {
                                tracing::warn!(error = %e, "daemon: failed to persist regime-shift signal");
                            } else {
                                metrics.signals_generated.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                    Ok(None) => {
                        tracing::debug!("daemon: no weather regime shift detected");
                    }
                    Err(e) => tracing::warn!(error = %e, "daemon: regime_shift_signal failed"),
                }

                *metrics.last_weather_poll.write().await = Some(now);
            }
            Err(e) => {
                tracing::warn!(error = %e, "daemon: weather fetch failed — will retry next interval");
                metrics.weather_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    async fn poll_prices(
        buffer: &Arc<RwLock<FeedBuffer>>,
        metrics: &Arc<DaemonMetrics>,
    ) {
        let now = Utc::now();

        // Fetch 30 days of BTC price history from CoinGecko
        match feeds::prices::fetch_btc_price_history(30).await {
            Ok(prices) => {
                tracing::info!(count = prices.len(), "daemon: fetched BTC prices");

                // Accumulate into rolling buffer
                {
                    let mut buf = buffer.write().await;
                    buf.push(FEED_PRICE, &prices);
                }

                // Read full rolling window
                let series = {
                    let buf = buffer.read().await;
                    buf.snapshot(FEED_PRICE).unwrap_or_default()
                };

                if series.is_empty() {
                    return;
                }

                let engine = KalshiSignalEngine::new();
                let store = SignalStore::new();

                match engine.price_regime_signal("BTC-USD", &series) {
                    Ok(sig) => {
                        tracing::info!(action = ?sig.action, edge = sig.edge, "daemon: price signal");
                        if sig.action == SignalAction::Enter || sig.action == SignalAction::Watch {
                            let record = kalshi_signal_to_record(&sig, "price/BTC-USD", now);
                            if let Err(e) = store.store_signal(&record) {
                                tracing::warn!(error = %e, "daemon: failed to persist price signal");
                            } else {
                                metrics.signals_generated.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "daemon: price_regime_signal failed"),
                }

                *metrics.last_price_poll.write().await = Some(now);
            }
            Err(e) => {
                tracing::warn!(error = %e, "daemon: BTC price fetch failed — will retry next interval");
                metrics.price_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    async fn poll_seismic(
        buffer: &Arc<RwLock<FeedBuffer>>,
        metrics: &Arc<DaemonMetrics>,
    ) {
        let now = Utc::now();

        // Fetch all earthquakes in the past day from USGS
        match feeds::usgs::fetch_magnitudes("all_day").await {
            Ok(mags) => {
                tracing::info!(count = mags.len(), "daemon: fetched seismic magnitudes");

                // Accumulate into rolling buffer
                {
                    let mut buf = buffer.write().await;
                    buf.push(FEED_SEISMIC, &mags);
                }

                // Read full rolling window
                let series = {
                    let buf = buffer.read().await;
                    buf.snapshot(FEED_SEISMIC).unwrap_or_default()
                };

                if series.len() < 50 {
                    tracing::debug!(
                        len = series.len(),
                        "daemon: not enough seismic data for analysis (need >= 50)"
                    );
                    *metrics.last_seismic_poll.write().await = Some(now);
                    return;
                }

                let detector = ArgusDetector::new();
                match detector.analyze_seismic_series(&series) {
                    Ok(anomaly) => {
                        tracing::info!(
                            severity = ?anomaly.severity,
                            chaos = anomaly.chaos_score,
                            regime_changed = anomaly.regime_changed,
                            "daemon: seismic analysis"
                        );

                        if anomaly.severity == AnomalySeverity::Critical {
                            tracing::warn!(
                                description = %anomaly.description,
                                chaos = anomaly.chaos_score,
                                "daemon: CRITICAL seismic anomaly detected"
                            );
                        } else if anomaly.severity == AnomalySeverity::High {
                            tracing::warn!(
                                description = %anomaly.description,
                                chaos = anomaly.chaos_score,
                                "daemon: HIGH seismic anomaly detected"
                            );
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "daemon: seismic analysis failed"),
                }

                *metrics.last_seismic_poll.write().await = Some(now);
            }
            Err(e) => {
                tracing::warn!(error = %e, "daemon: seismic fetch failed — will retry next interval");
                metrics.seismic_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl Default for FeedDaemon {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Conversion helper (mirrors api::signals::kalshi_signal_to_record) ──────

fn kalshi_signal_to_record(
    sig: &crate::signals::kalshi::KalshiSignal,
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
