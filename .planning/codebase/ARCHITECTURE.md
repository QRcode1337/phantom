# Architecture

**Analysis Date:** 2025-03-05

## Pattern Overview

**Overall:** Layered Pipeline / Analysis Engine

**Key Characteristics:**
- **Mathematical Core:** Centralized chaos theory and nonlinear dynamics math in `src/ftle/`.
- **Domain Specialization:** Pluggable detectors in `src/detectors/` that adapt core math to specific data types (weather, price, geospatial).
- **Actionable Output:** Signal aggregation in `src/signals/` that converts raw anomaly scores into market-ready recommendations.

## Layers

**Chaos Math (Core):**
- Purpose: Provides the mathematical foundation for anomaly detection using Finite-Time Lyapunov Exponents (FTLE).
- Location: `src/ftle/`
- Contains: `DelayEmbedding` for phase space reconstruction, `estimate_lyapunov` for chaos measurement, and `EchoStateNetwork` for time-series prediction.
- Depends on: `ndarray`, `nalgebra`, `rayon`
- Used by: All detectors in `src/detectors/`

**Domain Detectors:**
- Purpose: Implement domain-specific logic and thresholds for various data sources.
- Location: `src/detectors/`
- Contains: `WeatherEdgeDetector`, `PriceRegimeDetector`, `ArgusDetector`.
- Depends on: `src/ftle/`
- Used by: `src/signals/`, `src/lib.rs`

**Signal Engine:**
- Purpose: Aggregates multiple detectors and applies business/trading logic to produce actionable signals.
- Location: `src/signals/`
- Contains: `KalshiSignalEngine`.
- Depends on: `src/detectors/`
- Used by: `src/main.rs`, External consumers via `src/lib.rs`

## Data Flow

**Anomaly Detection Flow:**

1. **Input:** Raw time-series data (e.g., `Vec<f64>` of temperatures or prices) is passed to a detector.
2. **Embedding:** The detector calls `src/ftle/embedding.rs` to reconstruct the system's phase space.
3. **Calculation:** `src/ftle/ftle.rs` calculates the Lyapunov exponent (λ) to determine system stability/chaos.
4. **Prediction (Optional):** `src/ftle/echo_state.rs` may be used to predict the next value in the series.
5. **Domain Logic:** The detector (e.g., `src/detectors/price.rs`) evaluates the chaos score against historical norms to identify regime shifts.
6. **Action:** The signal engine (e.g., `src/signals/kalshi.rs`) maps the detector's findings to specific actions (Enter, Watch, Skip).

**State Management:**
- The system is largely stateless and functional. Detectors and Engines hold configuration but do not maintain internal state between analysis calls (except for optional caching in `aimds-detection` which is an external dependency).

## Key Abstractions

**Chaos Score:**
- Purpose: Normalized 0.0–1.0 representation of system chaos derived from λ.
- Examples: `src/ftle/mod.rs` (the `chaos_score` function).
- Pattern: Utility function.

**Detector:**
- Purpose: Processes domain data into high-level events.
- Examples: `src/detectors/weather.rs` (`WeatherEdgeDetector`), `src/detectors/price.rs` (`PriceRegimeDetector`).
- Pattern: Strategy pattern (though not explicitly trait-based yet).

**Signal Engine:**
- Purpose: High-level aggregator for market-specific logic.
- Examples: `src/signals/kalshi.rs` (`KalshiSignalEngine`).
- Pattern: Facade pattern.

## Entry Points

**CLI Demo:**
- Location: `src/main.rs`
- Triggers: Manual execution via `cargo run`.
- Responsibilities: Initializes `KalshiSignalEngine`, generates synthetic data, and prints a signal report.

**Library API:**
- Location: `src/lib.rs`
- Triggers: Import by external crates or examples.
- Responsibilities: Re-exports key detectors and engines for public use.

## Error Handling

**Strategy:** Result-based using `anyhow`.

**Patterns:**
- Extensive use of `anyhow::Result` for bubbling up mathematical or data-processing errors.
- Default implementations for detectors to ensure safe initialization.

## Cross-Cutting Concerns

**Logging:** Uses `tracing` and `tracing-subscriber` for diagnostic output.
**Serialization:** Uses `serde` for all data structures (Signals, Edges, Anomalies) to support API responses.
**Parallelism:** Uses `rayon` within `src/ftle/` and `ndarray` for high-performance mathematical operations.

---

*Architecture analysis: 2025-03-05*
