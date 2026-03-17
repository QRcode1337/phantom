# Codebase Concerns

**Analysis Date:** 2024-03-13

## Tech Debt

**Broken Examples:**
- Issue: `Cargo.toml` defines `[[example]]` blocks for `argus_anomaly` and `kalshi_edge`, but the `examples/` directory is empty. This prevents users from running demonstration code and breaks `cargo run --example`.
- Files: `Cargo.toml`, `examples/`
- Impact: Poor developer experience and lack of functional documentation/demos.
- Fix approach: Implement the missing example files in `examples/argus_anomaly.rs` and `examples/kalshi_edge.rs` or remove the blocks from `Cargo.toml`.

**Unused Dependencies:**
- Issue: `reqwest` is included in `[dependencies]` but is not imported or used anywhere in `src/`.
- Files: `Cargo.toml`
- Impact: Increased binary size and longer compilation times.
- Fix approach: Remove `reqwest` from `Cargo.toml` or implement the planned data-fetching logic (Open-Meteo/NWS feeds).

## Scalability

**Sequential FTLE Field Computation:**
- Issue: `calculate_ftle_field` uses a sequential `for` loop to compute the FTLE for every sliding window. Given the $O(N \cdot W \cdot D^2)$ complexity, this becomes a major bottleneck for long trajectories or high-dimensional data.
- Files: `src/ftle/ftle.rs`
- Impact: Slow performance on large datasets; fails to utilize multi-core architecture.
- Fix approach: Use `rayon` to parallelize the windowed computation, similar to how `estimate_lyapunov` uses `par_iter()`.

**In-line ESN Training:**
- Issue: `PriceRegimeDetector::esn_predict` initializes and trains a new Echo State Network (ESN) on every call to `analyze`.
- Files: `src/detectors/price.rs`
- Impact: High latency for real-time price analysis. Training a reservoir is expensive and shouldn't happen per-tick.
- Fix approach: Decouple ESN training from prediction. Maintain a persistent ESN state or use incremental learning for the readout weights.

## Error Handling & Reliability

**Silent Failure in Signal Processing:**
- Issue: `calculate_ftle_field` catches errors from `calculate_ftle_segment` and pushes `f64::NAN` to the result vector instead of returning an error or logging the failure.
- Files: `src/ftle/ftle.rs`
- Impact: Downstream detectors receive `NaN` values which can cause unexpected behavior or "garbage" signals without clear indication of the failure root cause.
- Fix approach: Return `Result<Vec<f64>>` and bubble up errors, or provide a structured output that includes error status for specific windows.

**Numerical Instability in Gram-Schmidt:**
- Issue: `gram_schmidt_orthonormalize` uses a hardcoded `1e-12` threshold for zero-vector detection.
- Files: `src/ftle/ftle.rs`
- Impact: On datasets with very small scales (e.g., normalized price returns), valid vectors might be treated as zero, causing the orthonormalization to fail or return incorrect results.
- Fix approach: Make the epsilon configurable in `FtleParams` or use a scale-relative threshold.

## Fragile Areas

**Input Validation Gaps:**
- Issue: Many public methods in `ArgusDetector`, `WeatherEdgeDetector`, and `PriceRegimeDetector` lack validation for `NaN`, `Inf`, or extreme outliers in the input `&[f64]` slices.
- Files: `src/detectors/argus.rs`, `src/detectors/price.rs`, `src/detectors/weather.rs`
- Why fragile: Chaos math (Lyapunov exponents) is extremely sensitive to `NaN` and outliers, which can "poison" the entire analysis.
- Safe modification: Add a validation step using `v.iter().all(|x| x.is_finite())` before processing.

## Missing Critical Features

**Data Feed Integration:**
- Issue: The system is designed to work with "feeds" (flight tracks, weather, prices), but there is currently no code to actually fetch this data from external APIs (like Open-Meteo or OpenSky).
- Blocks: End-to-end automation of the anomaly detection pipeline.
- Priority: High (required for the tool to be useful beyond synthetic data).

---

*Concerns audit: 2024-03-13*
