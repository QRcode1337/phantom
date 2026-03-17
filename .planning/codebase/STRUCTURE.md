# Codebase Structure

**Analysis Date:** 2025-03-05

## Directory Layout

```
phantom/
├── examples/               # Usage examples (Kalshi, Argus)
├── src/                    # Primary source code
│   ├── detectors/          # Domain-specific anomaly detectors
│   │   ├── argus.rs        # Geospatial/seismic anomaly detection
│   │   ├── mod.rs          # Detector module exports
│   │   ├── price.rs        # Price regime shift detection
│   │   └── weather.rs      # Weather market edge detection
│   ├── ftle/               # Chaos math (Finite-Time Lyapunov Exponents)
│   │   ├── echo_state.rs   # Echo State Networks for prediction
│   │   ├── embedding.rs    # Phase space reconstruction (Delay Embedding)
│   │   ├── ftle.rs         # Core Lyapunov exponent math
│   │   └── mod.rs          # High-level FTLE utilities
│   ├── signals/            # Aggregation and market signal engines
│   │   ├── kalshi.rs       # Kalshi market specific signals
│   │   └── mod.rs          # Signal module exports
│   ├── lib.rs              # Library entry point & public API
│   └── main.rs             # CLI demo entry point
├── AGENTS.md               # Agent-specific instructions
└── Cargo.toml              # Build and dependency configuration
```

## Directory Purposes

**`src/detectors/`:**
- Purpose: Specializes in analyzing specific data types to find "edges" or "anomalies."
- Contains: Rust source files for each domain (Argus, Price, Weather).
- Key files: `price.rs`, `weather.rs`, `argus.rs`.

**`src/ftle/`:**
- Purpose: The mathematical "engine room" of the project. Implements nonlinear dynamics algorithms.
- Contains: Low-level math modules for embedding, FTLE estimation, and reservoir computing (ESN).
- Key files: `ftle.rs`, `embedding.rs`, `echo_state.rs`.

**`src/signals/`:**
- Purpose: Translates detected anomalies into actionable business/market signals.
- Contains: Engines that consume multiple detectors.
- Key files: `kalshi.rs`.

## Key File Locations

**Entry Points:**
- `src/main.rs`: CLI demonstration of the engine.
- `src/lib.rs`: Defines the public API for the library.

**Configuration:**
- `Cargo.toml`: Project metadata and dependencies.
- `src/ftle/mod.rs`: Houses global configuration structs for math (e.g., `EmbeddingConfig`).

**Core Logic:**
- `src/ftle/ftle.rs`: The actual implementation of the Lyapunov exponent estimation.
- `src/ftle/embedding.rs`: Phase space reconstruction from time series data.

**Testing:**
- (Not explicitly found in separate directory, tests likely co-located in source files or `examples/`).

## Naming Conventions

**Files:**
- snake_case: `weather_edge.rs`, `kalshi_signal.rs`.

**Directories:**
- snake_case: `detectors/`, `ftle/`, `signals/`.

**Functions/Variables:**
- snake_case: `chaos_score()`, `analyze_temperature()`.

**Types/Structs:**
- PascalCase: `WeatherEdgeDetector`, `KalshiSignalEngine`.

## Where to Add New Code

**New Anomaly Source:**
- Primary code: Create a new file in `src/detectors/` (e.g., `src/detectors/stock_options.rs`).
- Tests: Include a `#[cfg(test)]` module at the bottom of the new file.

**New Market Aggregator:**
- Implementation: Create a new file in `src/signals/` (e.g., `src/signals/polymarket.rs`).
- Integration: Update `src/lib.rs` to export the new engine.

**New Mathematical Tool:**
- Shared helpers: `src/ftle/` if it's related to chaos math, or `src/utils/` (create if needed) for general utilities.

## Special Directories

**`examples/`:**
- Purpose: Contains standalone Rust programs demonstrating how to use the library for specific tasks (e.g., `examples/kalshi_edge.rs`).
- Generated: No.
- Committed: Yes.

---

*Structure analysis: 2025-03-05*
