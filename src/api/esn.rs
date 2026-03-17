use axum::{Json, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use crate::ftle::EchoStateNetwork;
use crate::ftle::EchoStateConfig;
use ndarray::{Array2, ArrayView1};

#[derive(Debug, Deserialize)]
pub struct EsnTrainRequest {
    pub series: Vec<f64>,
    pub reservoir_size: Option<usize>,
    pub spectral_radius: Option<f64>,
    pub leak_rate: Option<f64>,
    pub ridge_param: Option<f64>,
    pub connectivity: Option<f64>,
    pub input_scaling: Option<f64>,
    pub seed: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct EsnTrainResponse {
    pub predictions: Vec<f64>,
    pub actuals: Vec<f64>,
    pub mse: f64,
    pub training_samples: usize,
    pub reservoir_size: usize,
    pub dimension: usize,
}

pub async fn train_esn(Json(payload): Json<EsnTrainRequest>) -> impl IntoResponse {
    let n = payload.series.len();
    if n < 22 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Series too short for ESN training (min 22 points)" }))).into_response();
    }

    let mut config = EchoStateConfig::default();
    if let Some(v) = payload.reservoir_size { config.reservoir_size = v; }
    if let Some(v) = payload.spectral_radius { 
        if v >= 1.0 {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Spectral radius must be < 1.0" }))).into_response();
        }
        config.spectral_radius = v; 
    }
    if let Some(v) = payload.leak_rate { config.leak_rate = v; }
    if let Some(v) = payload.ridge_param { config.ridge_param = v; }
    if let Some(v) = payload.connectivity { config.connectivity = v; }
    if let Some(v) = payload.input_scaling { config.input_scaling = v; }
    config.seed = payload.seed;

    let mut esn = match EchoStateNetwork::new(config, 1, 1) {
        Ok(e) => e,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    let train_n = n - 1;
    let inputs = Array2::from_shape_fn((train_n, 1), |(i, _)| payload.series[i]);
    let targets = Array2::from_shape_fn((train_n, 1), |(i, _)| payload.series[i+1]);

    let washout = 10;
    let mse = match esn.train(inputs.view(), targets.view(), washout) {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    // Generate teacher-forced predictions
    let mut predictions = Vec::with_capacity(train_n - washout);
    let mut actuals = Vec::with_capacity(train_n - washout);
    
    esn.reset_state();
    for i in 0..train_n {
        let input_val = [payload.series[i]];
        let input = ArrayView1::from(&input_val);
        let pred = match esn.predict_step(input) {
            Ok(p) => p,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
        };
        
        if i >= washout {
            predictions.push(pred[0]);
            actuals.push(payload.series[i+1]);
        }
    }

    (StatusCode::OK, Json(EsnTrainResponse {
        predictions,
        actuals,
        mse,
        training_samples: train_n - washout,
        reservoir_size: esn.get_config().reservoir_size,
        dimension: 1,
    })).into_response()
}

// ─── POST /api/esn/predict ───────────────────────────────────────────────────
//
// Trains on the provided series, then runs free-running (autonomous) prediction
// for `horizon` steps into the future.

#[derive(Debug, Deserialize)]
pub struct EsnPredictRequest {
    /// Historical time series to train on.
    pub series: Vec<f64>,
    /// Number of future steps to predict (default 10, max 200).
    pub horizon: Option<usize>,
    // ESN hyperparameters (same as train)
    pub reservoir_size: Option<usize>,
    pub spectral_radius: Option<f64>,
    pub leak_rate: Option<f64>,
    pub ridge_param: Option<f64>,
    pub connectivity: Option<f64>,
    pub input_scaling: Option<f64>,
    pub seed: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct EsnPredictResponse {
    /// Free-running predictions for each future step.
    pub forecast: Vec<f64>,
    /// Training MSE (teacher-forced).
    pub training_mse: f64,
    pub horizon: usize,
    pub training_samples: usize,
    pub reservoir_size: usize,
}

pub async fn predict_esn(Json(payload): Json<EsnPredictRequest>) -> impl IntoResponse {
    let n = payload.series.len();
    if n < 22 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Series too short for ESN training (min 22 points)"
        }))).into_response();
    }

    let horizon = payload.horizon.unwrap_or(10).min(200);
    if horizon == 0 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "horizon must be >= 1"
        }))).into_response();
    }

    let mut config = EchoStateConfig::default();
    if let Some(v) = payload.reservoir_size { config.reservoir_size = v; }
    if let Some(v) = payload.spectral_radius {
        if v >= 1.0 {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Spectral radius must be < 1.0"
            }))).into_response();
        }
        config.spectral_radius = v;
    }
    if let Some(v) = payload.leak_rate { config.leak_rate = v; }
    if let Some(v) = payload.ridge_param { config.ridge_param = v; }
    if let Some(v) = payload.connectivity { config.connectivity = v; }
    if let Some(v) = payload.input_scaling { config.input_scaling = v; }
    config.seed = payload.seed;

    let mut esn = match EchoStateNetwork::new(config, 1, 1) {
        Ok(e) => e,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        }))).into_response(),
    };

    // Train
    let train_n = n - 1;
    let inputs = Array2::from_shape_fn((train_n, 1), |(i, _)| payload.series[i]);
    let targets = Array2::from_shape_fn((train_n, 1), |(i, _)| payload.series[i + 1]);
    let washout = 10;

    let training_mse = match esn.train(inputs.view(), targets.view(), washout) {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        }))).into_response(),
    };

    // Warm up the reservoir by feeding the entire training series
    esn.reset_state();
    let mut last_pred = *payload.series.last().unwrap();
    for i in 0..train_n {
        let input_val = [payload.series[i]];
        let input = ArrayView1::from(&input_val);
        match esn.predict_step(input) {
            Ok(p) => { last_pred = p[0]; }
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": e.to_string()
            }))).into_response(),
        }
    }

    // Free-running prediction: feed each prediction back as input
    let mut forecast = Vec::with_capacity(horizon);
    let mut current = last_pred;
    for _ in 0..horizon {
        let input_val = [current];
        let input = ArrayView1::from(&input_val);
        match esn.predict_step(input) {
            Ok(p) => {
                current = p[0];
                forecast.push(current);
            }
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": e.to_string()
            }))).into_response(),
        }
    }

    (StatusCode::OK, Json(EsnPredictResponse {
        forecast,
        training_mse,
        horizon,
        training_samples: train_n - washout,
        reservoir_size: esn.get_config().reservoir_size,
    })).into_response()
}
