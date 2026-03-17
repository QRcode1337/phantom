//! FTLE / Lyapunov chaos math modules
//! Extracted from temporal-attractor-studio v0.1.0 (MIT)

pub mod embedding;
pub mod ftle;
pub mod echo_state;

pub use ftle::{estimate_lyapunov, FtleParams, LyapunovResult};
pub use echo_state::{EchoStateNetwork, EchoStateConfig};
pub use embedding::{EmbeddingConfig, DelayEmbedding};

use anyhow::Result;

/// Compute chaos score 0.0–1.0 from a time series.
/// > 0.7 = chaotic, 0.3–0.7 = transitioning, < 0.3 = stable
pub fn chaos_score(series: &[f64], dt: f64) -> Result<f64> {
    let config = EmbeddingConfig::default();
    let embedder = DelayEmbedding::new(config.clone());
    let embedded = embedder.delay_embed(series, config.default_dimension, config.default_tau)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let params = FtleParams::default();
    let result = estimate_lyapunov(
        &embedded,
        dt,
        params.k_fit,
        params.theiler,
        params.max_pairs,
        params.min_init_sep,
    )?;

    let score = if result.lambda <= 0.0 { 0.0 } else { (result.lambda / 2.0).min(1.0) };
    Ok(score)
}

/// Detect regime change between historical and recent windows.
pub fn regime_changed(history: &[f64], recent: &[f64], dt: f64, threshold: f64) -> Result<bool> {
    let hist = chaos_score(history, dt)?;
    let rec = chaos_score(recent, dt)?;
    Ok((rec - hist).abs() > threshold)
}

/// Generate a Lorenz attractor time series for validation.
/// Standard parameters: sigma=10, rho=28, beta=8/3
pub fn lorenz_system(steps: usize, dt: f64) -> ndarray::Array2<f64> {
    let mut data = ndarray::Array2::zeros((steps, 3));
    let mut x = 1.0;
    let mut y = 1.0;
    let mut z = 1.0;

    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;

    for i in 0..steps {
        data[[i, 0]] = x;
        data[[i, 1]] = y;
        data[[i, 2]] = z;

        let dx = sigma * (y - x) * dt;
        let dy = (x * (rho - z) - y) * dt;
        let dz = (x * y - beta * z) * dt;

        x += dx;
        y += dy;
        z += dz;
    }

    data
}
