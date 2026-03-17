use axum::{
    extract::State,
    routing::{get, post, put},
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::daemon::{DaemonMetrics, DaemonStatus, BufferSizes, FeedDaemon};
use crate::daemon::feed_buffer::FeedBuffer;
use tokio::sync::RwLock;

pub mod analyze;
pub mod argus;
pub mod esn;
pub mod feeds;
pub mod metrics;
pub mod signals_db;
pub mod signals;
pub mod sse;

// ─── Shared daemon state for the status endpoint ────────────────────────────

/// Application state shared with Axum handlers via `State`.
#[derive(Clone)]
pub struct AppState {
    pub daemon_running: Arc<AtomicBool>,
    pub daemon_metrics: Arc<DaemonMetrics>,
    pub daemon_buffer: Arc<RwLock<FeedBuffer>>,
}

/// GET /api/daemon/status — returns daemon health snapshot.
async fn daemon_status(State(state): State<AppState>) -> Json<DaemonStatus> {
    let buf = state.daemon_buffer.read().await;
    let status = DaemonStatus {
        running: state.daemon_running.load(Ordering::Relaxed),
        last_weather_poll: *state.daemon_metrics.last_weather_poll.read().await,
        last_price_poll: *state.daemon_metrics.last_price_poll.read().await,
        last_seismic_poll: *state.daemon_metrics.last_seismic_poll.read().await,
        signals_generated: state.daemon_metrics.signals_generated.load(Ordering::Relaxed),
        weather_errors: state.daemon_metrics.weather_errors.load(Ordering::Relaxed),
        price_errors: state.daemon_metrics.price_errors.load(Ordering::Relaxed),
        seismic_errors: state.daemon_metrics.seismic_errors.load(Ordering::Relaxed),
        buffer_sizes: BufferSizes {
            weather: buf.len("weather"),
            price: buf.len("price"),
            seismic: buf.len("seismic"),
        },
    };
    Json(status)
}

// ─── Server entry points ────────────────────────────────────────────────────

/// Original entry point — starts server without daemon.
/// Kept for backward compatibility.
pub async fn start_server() {
    start_server_with_daemon(false).await;
}

/// Start the Axum server, optionally launching the background feed daemon.
pub async fn start_server_with_daemon(enable_daemon: bool) {
    // Capture the server start time for uptime tracking
    metrics::init_start_time();

    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "phantom=debug,tower_http=debug".into()),
        )
        .init();

    // Set up daemon (always create it so the status endpoint works)
    let daemon = FeedDaemon::new();
    let app_state = AppState {
        daemon_running: daemon.running_flag(),
        daemon_metrics: daemon.metrics(),
        daemon_buffer: daemon.buffer(),
    };

    if enable_daemon {
        daemon.start().await;
        tracing::info!("feed daemon started alongside API server");
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Daemon status route with AppState
    let daemon_router = Router::new()
        .route("/api/daemon/status", get(daemon_status))
        .with_state(app_state);

    let app = Router::new()
        // ── Core analysis endpoints ──────────────────────────────────────────
        .route("/api/health", get(feeds::health))
        .route("/api/analyze", post(analyze::analyze))
        .route("/api/embed", post(analyze::embed))
        .route("/api/ftle-field", post(analyze::ftle_field))
        .route("/api/esn-train", post(esn::train_esn))
        // ── Live feed endpoints ──────────────────────────────────────────────
        .route("/api/feeds/weather", get(feeds::get_weather))
        .route("/api/feeds/seismic", get(feeds::get_seismic))
        .route("/api/feeds/btc", get(feeds::get_btc))
        .route("/api/feeds/price", get(feeds::get_price))
        .route("/api/feeds/kalshi", get(feeds::get_kalshi_markets))
        .route("/api/feeds/opensky", get(feeds::get_opensky))
        // ── Signal persistence endpoints ─────────────────────────────────────
        .route("/api/signals/record", post(signals::record_signal))
        .route("/api/signals/analyze", post(signals::analyze_and_store))
        .route("/api/signals", get(signals::query_signals))
        // ── Signal outcome tracking (EXE-02) ────────────────────────────────
        .route("/api/signals/resolve", put(signals::resolve_signal))
        .route("/api/signals/summary", get(signals::signals_summary))
        // ── Signal SSE stream ──────────────────────────────────────────────────
        .route("/api/signals/stream", get(sse::signal_stream))
        // ── ARGUS anomaly detection (EXE-03) ──────────────────────────────────
        .route("/api/argus/analyze", post(argus::analyze_argus))
        .route("/api/esn/predict", post(esn::predict_esn))
        // ── Monitoring ──────────────────────────────────────────────────────────
        .route("/api/metrics", get(metrics::get_metrics))
        // ── Daemon status (stateful) ──────────────────────────────────────────
        .merge(daemon_router)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Phantom API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
