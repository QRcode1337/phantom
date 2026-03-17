# AGENTS.md — Phantom

Anomaly detection engine for ARGUS geospatial, weather markets, and price regime changes.
Built in Rust. Produces ranked, actionable Kalshi trading signals via FTLE chaos mathematics
and Echo State Network prediction.

---

## What This Is

Phantom ingests time-series data from three domains — geospatial flight/seismic feeds (ARGUS),
weather model ensembles (Kalshi weather markets), and price series (BTC, prediction market
prices) — and runs Lyapunov exponent analysis to detect when a system transitions between
stable and chaotic regimes. Those transitions are the signal.

Core insight: **regime shifts are where the edge is.** When a weather model ensemble diverges,
Kalshi has mispriced the market. When a price series transitions stable→chaotic, momentum
breaks. Phantom quantifies this and outputs ranked `Enter / Watch / Skip` decisions.

---

## Crate Layout

```
phantom/
├── src/
│   ├── lib.rs                  # Public API surface
│   ├── main.rs                 # CLI demo / smoke test
│   ├── ftle/
│   │   ├── mod.rs              # chaos_score() + regime_changed() helpers
│   │   ├── ftle.rs             # Core FTLE / Lyapunov estimator (VP-tree, parallel)
│   │   ├── embedding.rs        # Delay embedding (phase space reconstruction)
│   │   └── echo_state.rs       # Echo State Network (reservoir computing predictor)
│   ├── detectors/
│   │   ├── mod.rs
│   │   ├── weather.rs          # WeatherEdgeDetector — Open-Meteo ensemble analysis
│   │   ├── price.rs            # PriceRegimeDetector — BTC / prediction market prices
│   │   └── argus.rs            # ArgusDetector — flight tracks, seismic series
│   └── signals/
│       ├── mod.rs
│       └── kalshi.rs           # KalshiSignalEngine — combines all detectors → signals
├── examples/
│   ├── argus_anomaly.rs        # [STUB — files missing, remove from Cargo.toml or create]
│   └── kalshi_edge.rs          # [STUB — files missing, remove from Cargo.toml or create]
├── Cargo.toml
└── AGENTS.md
```

---

## Key Abstractions

### `ftle::chaos_score(series: &[f64], dt: f64) -> Result<f64>`
The workhorse. Takes a univariate time series, delay-embeds it into phase space
(default: dim=3, tau=1), then estimates the largest Lyapunov exponent via nearest-neighbor
divergence (VP-tree, Theiler window, parallel slope fitting). Returns a normalized score
`0.0–1.0`:

| Score | Interpretation |
|---|---|
| `< 0.3` | Stable / trending — predictable, trend-following works |
| `0.3–0.7` | Transitioning — edge of chaos, watch closely |
| `> 0.7` | Chaotic — unpredictable, reduce size or skip |

### `ftle::regime_changed(history, recent, dt, threshold) -> Result<bool>`
Computes `chaos_score` on two windows independently. Returns `true` if the absolute
difference exceeds `threshold` (default `0.2`). A `true` → `chaos_score` direction
tells you the *type* of regime shift (breakout vs. reversal).

### `detectors::WeatherEdgeDetector`
Wraps `chaos_score` + probability estimation over an ensemble temperature/precip series.
Model spread (std dev) → `EdgeConfidence`. Edge = `|p - 0.5|`. Only surfaces opportunities
above `min_edge = 0.08` (8%). Key output: `WeatherEdge` → fed into `KalshiSignalEngine`.

### `detectors::PriceRegimeDetector`
Splits a price series into historical + recent windows, computes chaos on both, classifies
into `TradingSignal` enum:
- `Trending` → stable, low chaos
- `Transitioning` → approaching boundary
- `Chaotic` → high unpredictability
- `BreakoutSignal` → stable→chaotic transition
- `ReversalSignal` → chaotic→stable transition

Optionally runs ESN prediction (`use_esn_prediction = true`) to forecast next price.

### `detectors::ArgusDetector`
Same regime analysis applied to flight altitude series (ADS-B) and seismic magnitude series
(USGS). Outputs `ArgusAnomaly` with `AnomalySeverity` (`Low / Medium / High / Critical`).
`Critical` = chaos > 0.6 AND regime changed.

### `signals::KalshiSignalEngine`
Orchestrates all three detectors. Applies two gates before emitting `Enter`:
1. `edge >= enter_threshold` (default 10%)
2. `chaos_score <= max_chaos_for_entry` (default 0.65) — won't enter into chaos

Output is a ranked `KalshiSignal` with `action: Enter | Watch | Skip`.

### `ftle::EchoStateNetwork`
Reservoir computing predictor (fixed random reservoir, trained output weights via ridge
regression). Used by `PriceRegimeDetector` to predict next-step price. Supports save/load
(JSON), autonomous closed-loop generation, and configurable spectral radius / leak rate.
**Echo state property**: spectral radius < 1.0 enforced at construction.

---

## Configuration

Key tuning parameters (all have sane defaults):

| Param | Default | Where | Effect |
|---|---|---|---|
| `FtleParams.k_fit` | 12 | `ftle.rs` | Early steps to fit slope — higher = smoother λ |
| `FtleParams.theiler` | 20 | `ftle.rs` | Temporal exclusion window — prevents trivial neighbors |
| `FtleParams.max_pairs` | 4000 | `ftle.rs` | Cap on NN pairs sampled — trades speed vs. accuracy |
| `WeatherEdgeDetector.min_edge` | 0.08 | `weather.rs` | Min edge to report (8%) |
| `PriceRegimeDetector.history_window` | 200 | `price.rs` | Historical chaos baseline window |
| `PriceRegimeDetector.recent_window` | 50 | `price.rs` | Recent chaos comparison window |
| `KalshiSignalEngine.enter_threshold` | 0.10 | `kalshi.rs` | Min edge to `Enter` (not just `Watch`) |
| `KalshiSignalEngine.max_chaos_for_entry` | 0.65 | `kalshi.rs` | Max chaos allowed to `Enter` |
| `EchoStateConfig.spectral_radius` | 0.95 | `echo_state.rs` | Reservoir stability (must be < 1.0) |
| `EchoStateConfig.reservoir_size` | 100 | `echo_state.rs` | Reservoir node count |
| `ArgusDetector.regime_threshold` | 0.25 | `argus.rs` | Regime change sensitivity |

---

## Data Requirements

| Detector | Minimum Series Length | Notes |
|---|---|---|
| `chaos_score` | ~50 points | Needs enough for delay embedding + VP-tree |
| `WeatherEdgeDetector` | 50 for basic, 100 for regime shift | More ensemble members = better spread |
| `PriceRegimeDetector` | 250 (200 history + 50 recent) | Returns `Trending` with low-data warning if short |
| `ArgusDetector` | 50 | Returns `Low` severity with insufficient-data message |
| `EchoStateNetwork.train` | ≥ 10 (functional), 100+ (useful) | More data = better output weights |

---

## Known Issues / Tech Debt

### 1. Missing Example Files (build error)
`Cargo.toml` declares two examples that don't exist on disk:
```toml
[[example]]
name = "argus_anomaly"
path = "examples/argus_anomaly.rs"

[[example]]
name = "kalshi_edge"
path = "examples/kalshi_edge.rs"
```
**Fix:** Either create the example files or remove the declarations. `cargo test --lib` works;
`cargo test` / `cargo build` will fail until resolved.

### 2. Lifetime Warnings (non-blocking)
`echo_state.rs` has two elided lifetime warnings on `update_state` and `get_state`. Fix with:
```bash
cargo fix --lib -p phantom
```
These are warnings only — doesn't affect correctness.

### 3. ESN Linear Solver
`EchoStateNetwork::solve_linear_system` uses nalgebra LU decomposition. For ill-conditioned
matrices, consider switching to SVD (`a_na.svd(true, true).solve(&b_na, 1e-12)`).

### 4. No Live Feed Integration Yet
All three detectors accept `&[f64]` slices — they're pure analysis functions with no HTTP
fetch built in. Feeding them requires:
- **Weather**: Open-Meteo ensemble API → `temperature_2m` array
- **Price**: exchange/CLOB price history
- **ARGUS**: OpenSky/ADS-B → altitude series per ICAO hex

The reqwest dependency is present for future HTTP integration.

---

## Testing

```bash
# Run all unit tests (12 tests, all passing)
cargo test --lib

# Run with output
cargo test --lib -- --nocapture

# Run specific module
cargo test --lib ftle::ftle
cargo test --lib ftle::echo_state

# Smoke test via binary (synthetic data demo)
cargo run
```

**Current coverage:** 12 unit tests across `ftle`, `embedding`, `echo_state`.
Detectors and signal engine have no tests yet — add integration tests with synthetic
series before using in production.

---

## Integration Points

### → ARGUS
Feed altitude series from OpenSky/ADS-B polling into `ArgusDetector::analyze_flight_track`.
USGS seismic magnitude series into `analyze_seismic_series`. Flag `Critical` anomalies
back to ARGUS HUD via the existing ARGUS API route layer.

### → Kalshi Bot (`bots/kalshi/`)
`KalshiSignalEngine` is the bridge. Wire Open-Meteo ensemble forecasts + Kalshi price
history into the engine, pipe `Enter` signals to the bot's order execution layer.

### → Polymarket Bots (`bots/polymarket/`)
`PriceRegimeDetector` can analyze Polymarket price series (0–1 probability as price).
`BreakoutSignal` / `ReversalSignal` on a market's price history = potential edge
complementary to the x_signal_bot sentiment approach.

---

## Next Build Priorities

1. **Fix example stubs** — create `examples/argus_anomaly.rs` + `examples/kalshi_edge.rs`
   or remove from `Cargo.toml` so `cargo build` passes cleanly
2. **HTTP feed layer** — `src/feeds/` module: Open-Meteo client, OpenSky client, price
   history fetcher — makes detectors self-contained and runnable as a daemon
3. **Detector tests** — integration tests with synthetic chaotic series (Lorenz attractor
   samples are ideal ground truth for FTLE validation)
4. **Signal persistence** — write `Enter` signals to SQLite with timestamp + outcome
   tracking so the edge model can be backtested
5. **ARGUS wiring** — emit `Critical` anomalies to ARGUS backend via HTTP POST

---

## Dependencies of Note

| Crate | Purpose |
|---|---|
| `ndarray` + `rayon` | Parallel array math for FTLE slope calculation |
| `nalgebra` | LU decomposition in ESN ridge regression |
| `aimds-detection`, `aimds-core` | Pattern matching primitives (pinned exact versions) |
| `midstreamer-temporal-compare` | Temporal comparison utilities |
| `reqwest` | HTTP (future feed layer — not yet used in detectors) |
| `blake3`, `sha2` | Hashing (from aimds-detection) |
| `dashmap` | Concurrent hashmap (from aimds-detection) |

All versions pinned with `=` for aimds/midstreamer crates — do not loosen without testing.

---

*Last updated: 2026-03-12*
