# External Integrations

**Analysis Date:** 2024-03-20

## APIs & External Services

**Weather Data:**
- **Open-Meteo** - Planned source for ensemble weather forecasts (GFS, ECMWF).
  - SDK/Client: `reqwest` (direct HTTP)
  - Purpose: Fetching hourly temperature and precipitation ensembles for market edge analysis.
  - Status: Referenced in `src/detectors/weather.rs` but not yet implemented.

**Geospatial Data (ARGUS):**
- **OpenSky Network / ADS-B** - Planned source for flight trajectory data.
  - SDK/Client: `reqwest` (direct HTTP)
  - Purpose: Detecting anomalous flight patterns (altitudes/paths) using FTLE.
  - Status: Referenced in `src/detectors/argus.rs`.
- **USGS Seismic API** - Planned source for earthquake event data.
  - SDK/Client: `reqwest` (direct HTTP)
  - Purpose: Identifying anomalous seismic clustering patterns.
  - Status: Referenced in `src/detectors/argus.rs`.

**Prediction Markets:**
- **Kalshi API** - Planned source for market prices and order book data.
  - SDK/Client: `reqwest` (direct HTTP)
  - Purpose: Fetching real-time pricing for temperature, precipitation, and price regime markets.
  - Status: Market logic exists in `src/signals/kalshi.rs`, but API connectivity is simulated.

## Data Storage

**Databases:**
- **Not detected**
  - Current state relies on in-memory processing of time-series data.

**File Storage:**
- **Local filesystem only**
  - Used for configuration and potentially future local caching of time-series datasets.

**Caching:**
- **None**
  - All analysis is currently performed on-demand from provided slices.

## Authentication & Identity

**Auth Provider:**
- **Custom / API Keys**
  - Implementation: Future requirement for Kalshi and potentially private weather/geospatial feeds.
  - Env vars: `KALSHI_API_KEY`, `KALSHI_API_SECRET` (planned).

## Monitoring & Observability

**Error Tracking:**
- **None**

**Logs:**
- **Tracing**
  - Implementation: Using `tracing` and `tracing-subscriber` in `Cargo.toml`.

## CI/CD & Deployment

**Hosting:**
- **Local / Bare Metal** (Current)

**CI Pipeline:**
- **None**

## Environment Configuration

**Required env vars:**
- None currently required for simulation.

**Secrets location:**
- Not applicable (simulation phase).

## Webhooks & Callbacks

**Incoming:**
- **None**

**Outgoing:**
- **None**

## Connectivity Status: Simulation vs. Reality

| Feature | Current (Simulation) | Planned (Connectivity) |
|---------|-----------------------|-------------------------|
| **Weather Feed** | Synthetic `f64` slices in `src/main.rs` | `reqwest` → Open-Meteo API |
| **Kalshi Markets** | Manual price input in `src/signals/kalshi.rs` | `reqwest` → Kalshi API |
| **Flight Tracks** | Synthetic data in `examples/argus_anomaly.rs` | `reqwest` → OpenSky API |
| **Seismic Data** | Not yet exercised | `reqwest` → USGS API |
| **Execution** | `print_report` to stdout | Automated order submission via Kalshi API |

---

*Integration audit: 2024-03-20*
