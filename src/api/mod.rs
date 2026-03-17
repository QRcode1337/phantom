use axum::{
    routing::{get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod analyze;
pub mod argus;
pub mod esn;
pub mod feeds;
pub mod metrics;
pub mod signals_db;
pub mod signals;

pub async fn start_server() {
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

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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
        .route("/api/feeds/opensky", get(feeds::get_opensky))
        // ── Signal persistence endpoints ─────────────────────────────────────
        .route("/api/signals/record", post(signals::record_signal))
        .route("/api/signals/analyze", post(signals::analyze_and_store))
        .route("/api/signals", get(signals::query_signals))
        // ── Signal outcome tracking (EXE-02) ────────────────────────────────
        .route("/api/signals/resolve", put(signals::resolve_signal))
        .route("/api/signals/summary", get(signals::signals_summary))
        // ── ARGUS anomaly detection (EXE-03) ──────────────────────────────────
        .route("/api/argus/analyze", post(argus::analyze_argus))
        .route("/api/esn/predict", post(esn::predict_esn))
        // ── Monitoring ──────────────────────────────────────────────────────────
        .route("/api/metrics", get(metrics::get_metrics))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Phantom API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
