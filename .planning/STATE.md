# State: phantom

## Project Reference
**Core Value:** Anomaly detection engine for ARGUS, weather markets, and price regime changes using Chaos Math (FTLE) and Echo State Networks.
**Current Focus:** Initial roadmap setup and core engine validation.

---

## Current Position
**Current Phase:** Phase 2: Signal Integration
**Current Status:** Phase 1 complete. All math primitives hardened, tested, and validated.

**Progress Bar:**
[====================] Phase 1: 100% ✅
[--------------------] Phase 2: 0%

---

## Performance Metrics
- **Unit Tests:** 14 passing (all modules)
- **Integration Tests:** 0 (Phase 2 priority)
- **Codebase Stability:** High — build clean, no errors
- **Build Status:** ✅ Clean (`cargo build` + `cargo test --lib` both pass)

---

## Accumulated Context
### Decisions
- Using Rust for high-frequency signal processing and parallel math.
- Leveraging `ndarray` for tensor operations and `nalgebra` for linear algebra.
- Focusing on "regime shifts" as the core trading edge.

### Decisions
- SVD solver already implemented in `echo_state.rs` — handles ill-conditioned matrices robustly.
- Lorenz validation test confirms ESN pipeline runs end-to-end (finite MSE, no NaN/panic).
- Examples use synthetic data; real data feeds are Phase 2 work.

### Blockers
- None.

### Todos (Phase 2)
- Implement `src/feeds/` module for external data ingestion.
- Open-Meteo ensemble API client (SIG-02).
- OpenSky/ADS-B polling client (SIG-03).
- Price history fetcher (SIG-04).
- Kalshi RSA key-pair auth + orderbook signal formatting (SIG-05).

---

## Session Continuity
**Last Phase:** Setup
**Next Phase:** Phase 1 (Execute remaining MATH requirements)
**Key Insight:** Numerical stability in the Lyapunov estimator is the most critical foundation for all downstream detectors.
