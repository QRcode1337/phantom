# Technology Stack

**Analysis Date:** 2024-03-22

## Languages

**Primary:**
- Rust 1.75+ (Edition 2021) - Core logic, detectors, and signal engine.

**Secondary:**
- None detected.

## Runtime

**Environment:**
- Tokio 1.50.0 (Asynchronous runtime)

**Package Manager:**
- Cargo (Rust)
- Lockfile: `Cargo.lock`

## Frameworks

**Core:**
- `aimds-detection` (=0.1.0) - Anomaly detection patterns.
- `aimds-core` (=0.1.0) - Core detection primitives.
- `midstreamer-temporal-compare` (=0.1.0) - Temporal data comparison.
- `midstreamer-scheduler` (=0.1.0) - Task scheduling (planned for recurring feed fetches).

**Testing:**
- `criterion` (=0.5.1) - Micro-benchmarking.
- `proptest` (=1.10.0) - Property-based testing.

**Build/Dev:**
- `cargo` - Primary build tool.

## Key Dependencies

**Critical:**
- `ndarray` (0.15 with `rayon`) - Numerical multidimensional arrays for chaos math.
- `nalgebra` (0.32) - Linear algebra for spatial/chaos calculations.
- `rayon` (1.8) - Parallel data processing.
- `serde` (=1.0.228) - Data serialization/deserialization.
- `serde_json` (=1.0.149) - JSON processing for API feeds.
- `reqwest` (0.12 with `json`) - HTTP client for fetching Open-Meteo/NWS/Kalshi feeds.

**Infrastructure:**
- `tokio` (=1.50.0) - Full async stack.
- `tracing` (=0.1.44) - Log instrumentation.
- `tracing-subscriber` (0.3) - Log collection and formatting.
- `anyhow` (=1.0.102) - Error handling.
- `thiserror` (=1.0.69) - Domain-specific error types.

## Configuration

**Environment:**
- No environment variables currently used for configuration.

**Build:**
- `Cargo.toml`: Central project configuration.
- `profile.release`: Configured for LTO (Link Time Optimization) and high optimization (`opt-level = 3`).

## Platform Requirements

**Development:**
- Rust toolchain (stable).

**Production:**
- Linux/Unix target recommended for high-performance signal processing.

---

*Stack analysis: 2024-03-22*
