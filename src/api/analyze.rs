use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use crate::ftle::{self, FtleParams, DelayEmbedding, EmbeddingConfig};

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub series: Vec<f64>,
    pub dt: f64,
    pub dimension: Option<usize>,
    pub tau: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub chaos_score: f64,
    pub lambda: f64,
    pub lyapunov_time: f64,
    pub doubling_time: f64,
    pub regime: String,
    pub points_used: usize,
    pub dimension: usize,
    pub pairs_found: usize,
}

pub async fn analyze(Json(payload): Json<AnalyzeRequest>) -> impl IntoResponse {
    let config = EmbeddingConfig::default();
    let dim = payload.dimension.unwrap_or(config.default_dimension);
    let tau = payload.tau.unwrap_or(config.default_tau);

    let min_required = (dim - 1) * tau + 1;
    if payload.series.len() < min_required {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": format!("Series too short: need {} points for dim={}, tau={}, got {}", min_required, dim, tau, payload.series.len()),
            "min_required": min_required,
            "got": payload.series.len(),
            "suggestion": "Reduce dimension or tau, or provide more data"
        }))).into_response();
    }

    let embedder = DelayEmbedding::new(config);
    let embedded = match embedder.delay_embed(&payload.series, dim, tau) {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let params = FtleParams::default();

    let theiler = params.theiler;
    if embedded.len() <= theiler * 2 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": format!("Not enough embedded vectors ({}) for Theiler window ({}). Need at least {} data points.", embedded.len(), theiler, (dim - 1) * tau + theiler * 2 + 1),
            "suggestion": "Increase series length or reduce dimension/tau"
        }))).into_response();
    }

    let result = match ftle::ftle::estimate_lyapunov(
        &embedded,
        payload.dt,
        params.k_fit,
        params.theiler,
        params.max_pairs,
        params.min_init_sep,
    ) {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let score = if result.lambda <= 0.0 { 0.0 } else { (result.lambda / 2.0).min(1.0) };
    let regime = if score < 0.3 { "stable" } else if score < 0.7 { "transitioning" } else { "chaotic" };

    (StatusCode::OK, Json(AnalyzeResponse {
        chaos_score: score,
        lambda: result.lambda,
        lyapunov_time: result.lyapunov_time,
        doubling_time: result.doubling_time,
        regime: regime.to_string(),
        points_used: result.points_used,
        dimension: result.dimension,
        pairs_found: result.pairs_found,
    })).into_response()
}

#[derive(Debug, Deserialize)]
pub struct EmbedRequest {
    pub series: Vec<f64>,
    pub dimension: Option<usize>,
    pub tau: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct EmbedResponse {
    pub vectors: Vec<Vec<f64>>,
    pub dimension: usize,
    pub tau: usize,
    pub num_vectors: usize,
}

pub async fn embed(Json(payload): Json<EmbedRequest>) -> impl IntoResponse {
    let config = EmbeddingConfig::default();
    let dim = payload.dimension.unwrap_or(config.default_dimension);
    let tau = payload.tau.unwrap_or(config.default_tau);

    let min_required = (dim - 1) * tau + 1;
    if payload.series.len() < min_required {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": format!("Series too short: need {} points for dim={}, tau={}, got {}", min_required, dim, tau, payload.series.len()),
            "min_required": min_required,
            "got": payload.series.len(),
            "suggestion": "Reduce dimension or tau, or provide more data"
        }))).into_response();
    }

    let embedder = DelayEmbedding::new(config);
    match embedder.delay_embed(&payload.series, dim, tau) {
        Ok(vectors) => {
            let num_vectors = vectors.len();
            (StatusCode::OK, Json(EmbedResponse {
                vectors,
                dimension: dim,
                tau,
                num_vectors,
            })).into_response()
        },
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct FtleFieldRequest {
    pub series: Vec<f64>,
    pub window_size: usize,
    pub dt: f64,
    pub dimension: Option<usize>,
    pub tau: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct FtleFieldResponse {
    pub field: Vec<f64>,
    pub positions: Vec<usize>,
    pub window_size: usize,
    pub series_len: usize,
    pub embedded_len: usize,
}

pub async fn ftle_field(Json(payload): Json<FtleFieldRequest>) -> impl IntoResponse {
    let config = EmbeddingConfig::default();
    let dim = payload.dimension.unwrap_or(config.default_dimension);
    let tau = payload.tau.unwrap_or(config.default_tau);

    let min_required = (dim - 1) * tau + 1;
    if payload.series.len() < min_required {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": format!("Series too short: need {} points for dim={}, tau={}, got {}", min_required, dim, tau, payload.series.len()),
            "min_required": min_required,
            "got": payload.series.len(),
            "suggestion": "Reduce dimension or tau, or provide more data"
        }))).into_response();
    }

    let embedder = DelayEmbedding::new(config);
    let embedded = match embedder.delay_embed(&payload.series, dim, tau) {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let embedded_len = embedded.len();
    match ftle::ftle::calculate_ftle_field(&embedded, payload.window_size, payload.dt) {
        Ok(field) => {
            let positions: Vec<usize> = (0..field.len()).collect();
            (StatusCode::OK, Json(FtleFieldResponse {
                field,
                positions,
                window_size: payload.window_size,
                series_len: payload.series.len(),
                embedded_len,
            })).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
