# Roadmap: phantom

## Overview
Phantom is an anomaly detection engine designed to detect regime shifts in geospatial, weather, and financial data. This roadmap outlines the path from the current core math implementation to a fully integrated signal engine and execution loop.

---

## Phase 1: Analytical Hardening
**Goal:** Ensure numerical stability and mathematical robustness of the core engine.

- **Dependencies:** None
- **Requirements:** MATH-01, MATH-02, MATH-03, MATH-04
- **Success Criteria:**
  - `cargo build` and `cargo test --lib` pass cleanly without elision warnings.
  - ESN linear solver handles ill-conditioned matrices using SVD.
  - FTLE and ESN predictions on Lorenz attractor series match ground truth within 5% error.
  - Examples `argus_anomaly.rs` and `kalshi_edge.rs` are fully functional and runnable.

---

## Phase 2: Signal Integration
**Goal:** Establish reliable data bridges to external feeds and market APIs.

- **Dependencies:** Phase 1
- **Requirements:** SIG-01, SIG-02, SIG-03, SIG-04, SIG-05
- **Success Criteria:**
  - `src/feeds/` module contains working clients for Open-Meteo, OpenSky, and Price history.
  - Kalshi API client successfully handles RSA key-pair authentication.
  - External API responses are correctly parsed into `&[f64]` slices for detector consumption.

---

## Phase 3: Domain Detectors
**Goal:** Refine and validate specialized detectors for Weather, Price, and Geospatial domains.

- **Dependencies:** Phase 2
- **Requirements:** DET-01, DET-02, DET-03, DET-04
- **Success Criteria:**
  - `WeatherEdgeDetector` identifies opportunities with >8% edge from ensemble forecasts.
  - `PriceRegimeDetector` accurately flags `BreakoutSignal` transitions on historical data.
  - `ArgusDetector` identifies `Critical` anomalies on flight and seismic data.
  - `KalshiSignalEngine` ranks signals with clear `Enter / Watch / Skip` actions.

---

## Phase 4: Execution Loop & Monitoring
**Goal:** Automate signal distribution, persistence, and basic performance monitoring.

- **Dependencies:** Phase 3
- **Requirements:** EXE-01, EXE-02, EXE-03, EXE-04
- **Success Criteria:**
  - All emitted signals are persisted to a SQLite database with metadata.
  - `Critical` anomalies are automatically POSTed to the configured ARGUS endpoint.
  - Basic monitoring dashboard (CLI or log-based) tracks signal frequency and detection latency.

---

## Progress

| Phase | Status | Progress |
|-------|--------|----------|
| 1 - Analytical Hardening | ✅ Complete | 100% |
| 2 - Signal Integration | ✅ Complete | 100% |
| 3 - Domain Detectors | Pending | 0% |
| 4 - Execution Loop | Pending | 0% |

## Traceability Map

| Requirement | Phase | Status |
|-------------|-------|--------|
| MATH-01 | Phase 1 | ✅ Done |
| MATH-02 | Phase 1 | ✅ Done |
| MATH-03 | Phase 1 | ✅ Done |
| MATH-04 | Phase 1 | ✅ Done |
| SIG-01 | Phase 2 | ✅ Done |
| SIG-02 | Phase 2 | ✅ Done |
| SIG-03 | Phase 2 | ✅ Done |
| SIG-04 | Phase 2 | ✅ Done |
| SIG-05 | Phase 2 | ✅ Done |
| DET-01 | Phase 3 | Pending |
| DET-02 | Phase 3 | Pending |
| DET-03 | Phase 3 | Pending |
| DET-04 | Phase 3 | Pending |
| EXE-01 | Phase 4 | Pending |
| EXE-02 | Phase 4 | Pending |
| EXE-03 | Phase 4 | Pending |
| EXE-04 | Phase 4 | Pending |
