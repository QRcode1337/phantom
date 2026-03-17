# Requirements: phantom (v1)

## v1: Core Signal & Detection Engine

### Analytical Hardening (MATH)
- **MATH-01**: Fix elided lifetime warnings in `echo_state.rs` via elision or explicit naming.
- **MATH-02**: Enhance `EchoStateNetwork` linear solver to handle ill-conditioned matrices (consider SVD instead of LU).
- **MATH-03**: Validate FTLE and ESN with synthetic chaotic ground truth (e.g., Lorenz attractor series).
- **MATH-04**: Fix example stubs (`argus_anomaly.rs`, `kalshi_edge.rs`) or remove from `Cargo.toml` to ensure clean `cargo build`.

### Signal Integration (SIG)
- **SIG-01**: Implement `src/feeds/` module for fetching external data.
- **SIG-02**: Open-Meteo ensemble API client for weather market feeds.
- **SIG-03**: OpenSky / ADS-B polling client for geospatial feeds.
- **SIG-04**: Price history fetcher for exchange/CLOB market data.
- **SIG-05**: Implement Kalshi RSA key-pair authentication and orderbook-friendly signal formatting.

### Domain Detectors (DET)
- **DET-01**: Refine `WeatherEdgeDetector` to handle ensemble spread more effectively for Kalshi markets.
- **DET-02**: Implement integration tests for `PriceRegimeDetector` with historical market data.
- **DET-03**: Finalize `ArgusDetector` with critical anomaly thresholding for flight and seismic feeds.
- **DET-04**: Support for `Enter / Watch / Skip` decision logic in `KalshiSignalEngine`.

### Execution Loop & Monitoring (EXE)
- **EXE-01**: Signal persistence layer (e.g., SQLite) to store emitted signals with timestamp and metadata.
- **EXE-02**: Signal outcome tracking (backtesting support) to evaluate edge over time.
- **EXE-03**: HTTP POST integration for emitting `Critical` anomalies to ARGUS HUD.
- **EXE-04**: Basic monitoring metrics (signal count, average chaos score, detection latency).

---

## v2: Future Enhancements (Deferred)
- **V2-01**: Real-time streaming integration with Apache Kafka or similar.
- **V2-02**: Dynamic parameter optimization for ESN (spectral radius, leak rate) using genetic algorithms.
- **V2-03**: Support for additional prediction markets (Polymarket).
- **V2-04**: Web-based dashboard for real-time visualization of chaos scores and regime shifts.
