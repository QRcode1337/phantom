# Phantom — Agent Handoff Document

## What Is Phantom?

A Rust-based anomaly detection engine that uses **chaos math** (Lyapunov exponents, FTLE, Echo State Networks) to detect regime shifts in time-series data. Targets three domains: weather markets, price regime changes, and geospatial anomalies — all for trading on **Kalshi** (prediction markets).

## Current State

| Phase | Status |
|-------|--------|
| 1 — Analytical Hardening | Done. Math core stable, SVD solver, Lorenz validation. |
| 2 — Signal Integration | Done. 5 live feed clients in `src/feeds/`. |
| 3 — Domain Detectors | Done (Prototypes functional). |
| 4 — Execution Loop | Not started |
| **GUI — Analysis Workbench** | **API Complete, Frontend Initialized** |

## Codebase Map

```
phantom/
├── Cargo.toml              — Rust deps (ndarray, nalgebra, rsa, reqwest, axum pending)
├── src/
│   ├── lib.rs              — Re-exports: detectors, signals, feeds
│   ├── main.rs             — CLI: --live (real feeds) or demo (synthetic)
│   ├── ftle/               — CHAOS MATH CORE
│   │   ├── mod.rs          — chaos_score(), regime_changed(), lorenz_system()
│   │   ├── ftle.rs         — estimate_lyapunov(), VP-tree NN, FTLE field, delay_embed()
│   │   ├── embedding.rs    — DelayEmbedding, autocorrelation tau estimation
│   │   └── echo_state.rs   — EchoStateNetwork (reservoir computing, ridge regression, SVD)
│   ├── detectors/          — DOMAIN DETECTORS
│   │   ├── weather.rs      — WeatherEdgeDetector (ensemble spread → edge)
│   │   ├── price.rs        — PriceRegimeDetector (chaos thresholds → Breakout/Reversal)
│   │   └── argus.rs        — ArgusDetector (flight tracks, seismic)
│   ├── signals/
│   │   └── kalshi.rs       — KalshiSignalEngine (Enter/Watch/Skip decisions)
│   └── feeds/              — LIVE DATA CLIENTS (Phase 2)
│       ├── open_meteo.rs   — fetch_temperature(), fetch_precipitation(), fetch_ensemble()
│       ├── opensky.rs      — fetch_states() with adsb.lol fallback
│       ├── usgs.rs         — fetch_earthquakes(), fetch_magnitudes()
│       ├── prices.rs       — fetch_btc_price_history(), fetch_exchange_price()
│       └── kalshi.rs       — KalshiClient (RSA-PSS auth, markets, orderbook)
├── examples/
│   ├── argus_anomaly.rs
│   └── kalshi_edge.rs
├── tests/
│   └── feeds_integration.rs  — 10 live API integration tests (#[ignore])
└── docs/superpowers/specs/
    └── 2026-03-16-analysis-workbench-design.md  — FULL GUI SPEC
```

## Key Concepts

- **Chaos Score**: 0.0–1.0 normalized Lyapunov exponent. >0.7 = chaotic, 0.3–0.7 = transitioning, <0.3 = stable.
- **Regime Change**: When chaos_score shifts significantly between historical and recent windows.
- **FTLE Field**: Sliding-window Lyapunov exponents across a time series — regime boundaries show as ridges.
- **ESN**: Echo State Network — reservoir computing for next-step prediction. Trained via ridge regression. Divergence from actuals indicates chaos.
- **Delay Embedding**: Takens' theorem — reconstruct phase space from a 1D series using `[x(t), x(t+tau), x(t+2*tau), ...]`.

## What Needs To Be Built: Analysis Workbench GUI

**Full spec:** `docs/superpowers/specs/2026-03-16-analysis-workbench-design.md`

### Summary

Next.js frontend + Rust axum API backend. 6 interactive panels:

1. **Chaos Score Timeline** — Line chart with stable/transitioning/chaotic bands
2. **Phase Space Embedding** — 3D scatter of delay-embedded attractor
3. **FTLE Field Heatmap** — Sliding-window chaos across time
4. **ESN Prediction vs Actual** — Teacher-forced overlay showing divergence
5. **Parameter Tuner** — Sliders for embedding (dim, tau), FTLE (k_fit, theiler), ESN (reservoir_size, spectral_radius, leak_rate, connectivity, ridge_param), regime threshold
6. **Feed Status & Data Loader** — Live feed health + fetch/paste/upload data

### Tech Stack

- **Backend:** Rust + axum, wraps existing `src/ftle/` and `src/feeds/` functions as JSON endpoints
- **Frontend:** Next.js + React + react-plotly.js + Tailwind CSS
- **Charting:** Plotly.js (handles 3D scatter, heatmaps, line charts)

### API Endpoints (Rust side, `src/api/`)

| Endpoint | Method | Does |
|----------|--------|------|
| `/api/analyze` | POST | Full chaos analysis: embed → Lyapunov → chaos_score + regime |
| `/api/embed` | POST | Delay embedding → phase space vectors |
| `/api/ftle-field` | POST | Embed → sliding-window FTLE field |
| `/api/esn-train` | POST | Train ESN, return teacher-forced predictions + MSE |
| `/api/feeds/weather` | GET | Proxy Open-Meteo temperature |
| `/api/feeds/seismic` | GET | Proxy USGS magnitudes |
| `/api/feeds/btc` | GET | Proxy CoinGecko BTC prices |
| `/api/feeds/opensky` | GET | Proxy ADS-B aircraft states |
| `/api/health` | GET | Feed availability status |

### New Dependencies Needed

**Rust:** `axum`, `tower-http` (CORS)
**Frontend:** `next`, `react`, `react-plotly.js`, `plotly.js`, `tailwindcss`

### Critical Implementation Notes

- All computation endpoints accept raw `f64[]` series and handle embedding internally
- `estimate_lyapunov()` returns `Result<LyapunovResult>` — the endpoint derives chaos_score from lambda
- `calculate_ftle_field()` takes embedded `Vec<Vec<f64>>`, not raw series — embed first
- `EchoStateNetwork::new()` returns `Result<Self>` and validates spectral_radius < 1.0
- `train()` returns `Result<f64>` where the f64 is training MSE
- Minimum series for ESN: 22 points (washout=10)
- Minimum series for embedding: `dimension * tau + 1` points
- Feed proxies cache 60s, frontend debounces 500ms

## Credentials (DO NOT COMMIT)

- Kalshi RSA key: `~/.config/kalshi/private.pem`
- Kalshi key ID: `~/.config/kalshi/credentials.json`
- All other feeds (Open-Meteo, OpenSky, USGS, CoinGecko) are free / no auth

## How To Run

```bash
# Demo mode (synthetic data, no network)
cargo run

# Live mode (fetches real feeds)
cargo run -- --live

# Tests
cargo test --lib                                    # 14 unit tests
cargo test --test feeds_integration -- --ignored    # 10 live API tests
```
