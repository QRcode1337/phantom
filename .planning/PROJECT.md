# Project: phantom

## Core Value
Anomaly detection engine for ARGUS geospatial, weather markets, and price regime changes using Chaos Math (FTLE) and Reservoir Computing (Echo State Networks).

## Vision
To provide a high-performance, Rust-based engine that identifies trading edges and critical anomalies by detecting transitions between stable and chaotic regimes in complex time-series data.

## Scope
- Chaos math primitives (FTLE, embedding)
- Reservoir computing (Echo State Networks)
- Domain-specific detectors (Weather, Price, Geospatial/Seismic)
- Signal orchestration for trading (Kalshi)
- Integration with external data feeds (Open-Meteo, OpenSky, etc.)

## Constraints
- Language: Rust
- Performance: High-frequency signal processing, parallelized math
- Reliability: Numerical stability in Lyapunov calculations
- Dependencies: Pinned versions for `aimds` and `midstreamer` crates
