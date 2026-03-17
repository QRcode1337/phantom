# Phantom Analysis Workbench — Design Spec

## Problem

Phantom's chaos math engine (FTLE, ESN, delay embedding) produces rich analytical output — chaos scores, phase space trajectories, FTLE fields, regime transitions — but it's only accessible via CLI text output. There's no way to visualize attractors, interactively tune parameters, or see how chaos evolves across a time series. This limits the tool's usefulness for research, parameter tuning, and building intuition about the underlying dynamics.

## Solution

An interactive browser-based analysis workbench with 6 visualization panels. A Rust API server exposes Phantom's internals as JSON endpoints. A Next.js frontend renders interactive charts using Plotly.js. Users load data (live feeds or manual input), visualize the chaos analysis, and tune detector parameters with immediate visual feedback.

## Architecture

Two processes:

- **Rust API** (`cargo run -- --api`, port 8080) — axum server wrapping existing Phantom library functions. Thin endpoints, no new math.
- **Next.js Frontend** (`phantom-ui/`, port 3000) — React SPA with Plotly.js charts. Calls Rust API for all computation.

```
phantom (Rust binary)          phantom-ui (Next.js app)
┌─────────────────────┐       ┌──────────────────────────┐
│  axum JSON API      │◄─────►│  React + Plotly.js       │
│  POST /api/analyze   │       │  6 views / panels        │
│  POST /api/embed     │       │  Parameter controls      │
│  POST /api/ftle-field│       │  Feed status & data load │
│  POST /api/esn-train │       └──────────────────────────┘
│  GET  /api/feeds/*   │       localhost:3000
│  GET  /api/health    │
└─────────────────────┘
localhost:8080
```

## Rust API Endpoints (`src/api/`)

New module in Phantom. Each endpoint wraps existing library functions with JSON serialization.

### Computation Endpoints

#### `POST /api/analyze`

**Input:** `{ series: f64[], dt: f64, dimension?: usize, tau?: usize }`

**Implementation:** The endpoint performs the full analysis pipeline:
1. Delay-embed the raw series using `DelayEmbedding::new(EmbeddingConfig::default()).delay_embed(series, dimension, tau)` (defaults: dimension=3, tau=1 from `EmbeddingConfig::default()`).
2. Call `ftle::ftle::estimate_lyapunov(&embedded, dt, params.k_fit, params.theiler, params.max_pairs, params.min_init_sep)` to get the full `LyapunovResult`.
3. Derive `chaos_score` from `lambda`: `if lambda <= 0.0 { 0.0 } else { (lambda / 2.0).min(1.0) }` (same formula as `ftle::chaos_score()`).
4. Derive `regime` from `chaos_score`: `"stable"` if < 0.3, `"transitioning"` if 0.3–0.7, `"chaotic"` if > 0.7.

**Output:** `{ chaos_score: f64, lambda: f64, lyapunov_time: f64, doubling_time: f64, regime: "stable" | "transitioning" | "chaotic", points_used: usize, dimension: usize, pairs_found: usize }`

**Wraps:** `ftle::ftle::estimate_lyapunov()` + inline chaos_score derivation.

#### `POST /api/embed`

**Input:** `{ series: f64[], dimension?: usize, tau?: usize }`

**Implementation:** Instantiates `DelayEmbedding::new(EmbeddingConfig::default())` and calls `.delay_embed(series, dimension, tau)?`. If `dimension`/`tau` are omitted, uses `EmbeddingConfig::default()` values (dimension=3, tau=1). Note: uses the method on `DelayEmbedding` (from `embedding.rs`), not the standalone `delay_embed()` function in `ftle.rs`. Minimum series length: `dimension * tau + 1` (e.g., 4 for defaults). The `min_points_required` field on `EmbeddingConfig` (100) is advisory and not enforced by `delay_embed()` — the actual guard is `series.len() < m * tau + 1`. Returns HTTP 400 if series is too short.

**Output:** `{ vectors: f64[][], dimension: usize, tau: usize, num_vectors: usize }`

**Wraps:** `ftle::embedding::DelayEmbedding::delay_embed()`

#### `POST /api/ftle-field`

**Input:** `{ series: f64[], window_size: usize, dt: f64, dimension?: usize, tau?: usize }`

**Implementation:** The endpoint first delay-embeds the raw series using `DelayEmbedding::delay_embed()` (same as `/api/embed`), then passes the embedded `Vec<Vec<f64>>` trajectory to `ftle::ftle::calculate_ftle_field(&embedded, window_size, dt)?`. Note: `field.len() = embedded.len() - window_size` — the output is shorter than the input series by `(dimension - 1) * tau + window_size`. The `positions` array is `0..field.len()`, where each value represents the **start index** of the sliding window in the embedded trajectory.

**Output:** `{ field: f64[], positions: usize[], window_size: usize, series_len: usize, embedded_len: usize }`

**Wraps:** `ftle::embedding::DelayEmbedding::delay_embed()` → `ftle::ftle::calculate_ftle_field()`

#### `POST /api/esn-train`

**Input:** `{ series: f64[], reservoir_size?: usize, spectral_radius?: f64, leak_rate?: f64, ridge_param?: f64, connectivity?: f64, input_scaling?: f64, seed?: u64 }`

**Validation:**
- `series.len() >= 22` (minimum: washout=10 requires n-1 > washout, so n >= 12, but 22 gives meaningful training). Returns HTTP 400 if too short.
- `spectral_radius < 1.0`. Returns HTTP 400 if >= 1.0. Note: `EchoStateNetwork::new()` also enforces this internally with `bail!()` as a secondary guard.
- `connectivity` in 0.0–1.0 (default 0.1 from `EchoStateConfig::default()`).
- `leak_rate` in 0.0–1.0 (default 1.0).

**Implementation:**
1. Construct input matrix: `Array2::from_shape_fn((n-1, 1), |(i, _)| series[i])`.
2. Construct target matrix: `Array2::from_shape_fn((n-1, 1), |(i, _)| series[i+1])` — one-step-ahead prediction.
3. Create `EchoStateConfig` from params. Omitted fields use `EchoStateConfig::default()`: `connectivity=0.1`, `input_scaling=1.0`.
4. Instantiate `let mut esn = EchoStateNetwork::new(config, 1, 1)?` (returns `Result<Self>`).
5. Train: `let mse = esn.train(inputs.view(), targets.view(), 10)?` — returns `Result<f64>` where the `f64` is the training MSE.
6. Generate **teacher-forced** predictions: for each step i in the test portion, call `esn.predict_step(ArrayView1::from(&[series[i]]))?` using the actual value as input (not the previous prediction). This shows how well the ESN tracks the series, with divergence indicating chaos.
7. Return predictions, corresponding actuals, and MSE.

**Output:** `{ predictions: f64[], actuals: f64[], mse: f64, training_samples: usize, reservoir_size: usize, dimension: usize }`

**Wraps:** `ftle::echo_state::EchoStateNetwork`

### Feed Proxy Endpoints

| Endpoint | Method | Params | Output | Wraps |
|----------|--------|--------|--------|-------|
| `/api/feeds/weather` | GET | `lat, lon, hours` | `{ series: f64[], source, points }` | `feeds::open_meteo::fetch_temperature()` |
| `/api/feeds/seismic` | GET | `period` | `{ series: f64[], source, points }` | `feeds::usgs::fetch_magnitudes()` |
| `/api/feeds/btc` | GET | `days` | `{ series: f64[], source, points }` | `feeds::prices::fetch_btc_price_history()` |
| `/api/feeds/opensky` | GET | `bbox?` (lamin,lomin,lamax,lomax) | `{ states: AircraftState[], count }` | `feeds::opensky::fetch_states()` |
| `/api/health` | GET | — | `{ status, feeds: { weather, seismic, btc, opensky } }` | Pings each feed |

**Seismic `period` validation:** Server validates that `period` is one of: `"all_hour"`, `"all_day"`, `"all_week"`, `"all_month"`, `"significant_day"`, `"significant_week"`, `"significant_month"`. Returns HTTP 400 for invalid values.

**OpenSky response schema:** Returns the full `Vec<AircraftState>` serialized as JSON. Each `AircraftState` has fields: `icao24: string, callsign: string?, lat: f64, lon: f64, baro_altitude: f64?, geo_altitude: f64?, velocity: f64?, track: f64?, vertical_rate: f64?, on_ground: bool`. The frontend can extract altitude series client-side for detector analysis if needed.

### Response Format

All endpoints return JSON. Errors return `{ "error": "message" }` with appropriate HTTP status codes (400 for validation errors, 502 for upstream feed failures, 500 for internal errors).

### Caching & Rate Limits

- Feed proxy endpoints cache responses for 60 seconds (in-memory TTL cache). Repeated requests within the TTL return cached data.
- Frontend debounces "Apply" button: minimum 500ms between analysis requests.
- CoinGecko (10-30 req/min) and OpenSky (100/day unauth) rate limits are handled by the cache. If a 429 response is received from upstream, it surfaces as a feed status error with a "Rate limited — try again in X seconds" message.

## Frontend Views (`phantom-ui/`)

### Layout

```
┌─────────────────────────────────────────────────────────┐
│  [Feed Status Bar]  Weather ● Seismic ● BTC ● Flights  │
│  [Data: NYC Temperature 200pts]        [Load] [Paste]   │
├────────────┬────────────────────────────────────────────┤
│            │  ┌─────────────────┬─────────────────┐     │
│  Parameter │  │ Chaos Timeline  │ Phase Space 3D  │     │
│  Tuner     │  ├─────────────────┼─────────────────┤     │
│            │  │ FTLE Heatmap    │ ESN Pred vs Act │     │
│  [sliders] │  └─────────────────┴─────────────────┘     │
│            │                                            │
│  [Apply]   │                                            │
└────────────┴────────────────────────────────────────────┘
```

Left sidebar: parameter tuner. Top bar: feed status + data loader. Main area: 2x2 chart grid.

### View 1: Chaos Score Timeline

- Plotly line chart. X = time index, Y = chaos score (0.0–1.0).
- Three horizontal band fills: green (<0.3 stable), yellow (0.3–0.7 transitioning), red (>0.7 chaotic).
- Updates on data load or parameter change.

### View 2: Phase Space Embedding

- Plotly 3D scatter plot of delay-embedded state vectors.
- Controls: dimension toggle (2D/3D), tau slider.
- Points color-coded by time index (gradient) to show trajectory evolution.
- Interactive rotation, zoom, pan.

### View 3: FTLE Field Heatmap

- Plotly heatmap. X = window position in time series, color = FTLE magnitude.
- Regime boundaries appear as bright ridges.
- Adjustable window_size slider.

### View 4: ESN Prediction vs Actual

- Dual-line Plotly chart. Blue = actual series, orange = ESN prediction.
- Shaded region showing divergence magnitude.
- Stats panel: MSE, Lyapunov time, reservoir size, training samples.

### View 5: Parameter Tuner (Sidebar)

Grouped sliders with labels and current values:

**Embedding:**
- dimension: 2–10 (default 3)
- tau: 1–20 (default 1)

**FTLE:**
- k_fit: 2–30 (default 12)
- theiler: 5–100 (default 20)
- max_pairs: 100–10000 (default 4000)

**ESN:**
- reservoir_size: 10–500 (default 100)
- spectral_radius: 0.1–0.99 (default 0.95). Note: `EchoStateNetwork::new()` enforces `spectral_radius < 1.0` with a hard error. The slider max of 0.99 prevents this, but the API also validates server-side and returns HTTP 400 if >= 1.0.
- leak_rate: 0.01–1.0 (default 1.0)
- connectivity: 0.01–1.0 (default 0.1). Controls reservoir sparsity — lower values = sparser, faster computation.
- input_scaling: 0.01–5.0 (default 1.0). Scales input weights before reservoir injection.
- ridge_param: 1e-12–1e-2, log scale (default 1e-8)

**Regime:**
- threshold: 0.05–0.5 (default 0.2)

"Apply" button triggers re-analysis with current parameters. All 4 charts update. Button is debounced (500ms minimum between requests).

### View 6: Feed Status & Data Loader (Top Bar)

- Feed health indicators: green/red dots for each feed (weather, seismic, BTC, flights). Determined by `GET /api/health`.
- Rate limit errors (429) display as amber dot with "Rate limited" tooltip.
- Three data input modes:
  - **Fetch Live:** Dropdown per feed with "Fetch" button.
  - **Paste:** Text area accepting comma/newline-separated numbers.
  - **Upload:** CSV file upload (first numeric column used as series).
- Active series metadata displayed: source name, point count.

## Data Flow

1. User selects data source (e.g., "Fetch Live → BTC 30d").
2. Frontend calls `GET /api/feeds/btc?days=30`.
3. Rust API checks in-memory cache (60s TTL). On miss, delegates to `feeds::prices::fetch_btc_price_history(30)`.
4. Returns `{ series: [...], source: "coingecko", points: N }`. (Point count varies by CoinGecko granularity: ~168 for 7d, ~720 for 30d at hourly intervals.)
5. Frontend stores series in React state, fires 4 parallel POST requests:
   - `/api/analyze` → chaos score + lambda + regime for timeline
   - `/api/embed` → embedded vectors for phase space plot
   - `/api/ftle-field` → FTLE values for heatmap
   - `/api/esn-train` → predictions for overlay chart
6. Charts render with Plotly.
7. User adjusts parameters in sidebar, clicks "Apply", step 5 repeats with new params (debounced).

## New Dependencies

### Rust

| Crate | Purpose |
|-------|---------|
| `axum` | HTTP server framework |
| `tower-http` | CORS middleware |

All other deps (serde, serde_json, tokio, ndarray, the feeds module) already exist.

### Frontend (`phantom-ui/package.json`)

| Package | Purpose |
|---------|---------|
| `next` | React framework |
| `react`, `react-dom` | UI |
| `react-plotly.js`, `plotly.js` | All charts (line, 3D scatter, heatmap) |
| `tailwindcss` | Styling |

## File Structure

```
src/api/
├── mod.rs        — axum router, CORS setup, server startup
├── analyze.rs    — /api/analyze, /api/embed, /api/ftle-field
├── esn.rs        — /api/esn-train
└── feeds.rs      — /api/feeds/*, /api/health, feed cache

phantom-ui/
├── package.json
├── next.config.js
├── tailwind.config.js
├── src/
│   ├── app/
│   │   ├── layout.tsx
│   │   └── page.tsx        — main workbench page
│   ├── components/
│   │   ├── ChaosTimeline.tsx
│   │   ├── PhaseSpace.tsx
│   │   ├── FtleHeatmap.tsx
│   │   ├── EsnPrediction.tsx
│   │   ├── ParameterTuner.tsx
│   │   ├── FeedStatus.tsx
│   │   └── DataLoader.tsx
│   └── lib/
│       └── api.ts          — fetch helpers for Rust API
```

## Error Handling

- API errors return `{ "error": "descriptive message" }` with HTTP 400 (validation), 502 (upstream feed failure), or 500 (internal).
- Frontend shows inline error banners per chart panel (not modal dialogs).
- Feed failures show red status dot + tooltip with error message.
- Rate limit (429) responses show amber status dot with retry guidance.
- Parameter validation: server-side validation for all computation params (spectral_radius < 1.0, dimension >= 2, tau >= 1, etc.). Sliders constrain client-side ranges as first defense.

## Out of Scope (v1)

- Trading controls / order placement
- Signal report view (Enter/Watch/Skip)
- WebSocket streaming / real-time updates
- Persisting analysis sessions
- Multi-series comparison (one active series at a time)
