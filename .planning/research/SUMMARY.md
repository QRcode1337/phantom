# Research Summary: Phantom

**Domain:** Anomaly detection engine for ARGUS geospatial, weather markets, and price regime changes using Chaos Math (FTLE) and Reservoir Computing (Echo State Networks).
**Researched:** 2025-05-24
**Overall confidence:** HIGH

## Executive Summary

The 'phantom' project occupies a specialized niche at the intersection of dynamical systems theory (Chaos Math) and modern machine learning (Reservoir Computing). It leverages Finite-Time Lyapunov Exponents (FTLE) to measure the stability and predictability of time-series data, specifically focusing on weather patterns and financial markets. 

By detecting transitions between "stable" (predictable) and "chaotic" (unpredictable) regimes, the system identifies trading edges in prediction markets like Kalshi. The core technical advantage lies in using Echo State Networks (ESNs)—a type of recurrent neural network—to provide high-speed, computationally efficient forecasts that are well-suited for the "edge of chaos" where traditional linear models fail.

## Key Findings

**Stack:** Rust-based implementation using `ndarray` for tensor operations, `nalgebra` for linear systems, and `tokio` for high-frequency signal processing.
**Architecture:** A modular signal-detector-action pipeline where raw data is embedded into phase space, analyzed for divergence (FTLE), and used to train local reservoirs (ESN) for short-term prediction.
**Critical pitfall:** Numerical instability in Lyapunov calculation and the sensitivity of the Echo State Property to spectral radius scaling.

## Implications for Roadmap

Based on research, suggested phase structure:

1. **Phase 1: Core Analytical Engine** - Ensure mathematical robustness of FTLE and ESN.
   - Addresses: Taken's embedding, VP-tree nearest neighbors, Ridge regression training.
   - Avoids: Degenerate time variance, exploding reservoir states.

2. **Phase 2: Signal Integration** - Build reliable bridges to external data.
   - Addresses: Open-Meteo ensemble fetching, Kalshi API (v2) integration, NWS feeds.
   - Research Flag: Kalshi's RSA key-pair authentication and inverted orderbook logic.

3. **Phase 3: Domain-Specific Detectors** - Tailor signals for Weather and Price.
   - Addresses: Ensemble model divergence (GFS vs ECMWF), Regime shift breakout signals.

4. **Phase 4: Automated Execution & Monitoring** - Full loop from detection to trade.

**Phase ordering rationale:**
- Mathematical primitives (FTLE/ESN) must be stable before building detectors that rely on their outputs.
- Weather/Price detectors require high-quality signals (Open-Meteo/Kalshi) to be valid.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Rust ecosystem for math (`ndarray`) is mature and high-performance. |
| Features | HIGH | Project already implements core FTLE and ESN logic. |
| Architecture | MEDIUM | Component boundaries (Signal vs Detector) are clear but integration with trading execution is yet to be fully defined. |
| Pitfalls | HIGH | Common chaos math pitfalls (embedding lag, Theiler window) are already addressed in current code. |

## Gaps to Address

- **Kalshi Execution:** While data fetching is understood, the trade execution path (RSA signing) needs verification in a production environment.
- **Latency Sensitivity:** How sensitive is the FTLE "breakout" signal to real-time market data delays?
- **Backtesting:** Need a robust way to validate "regime change" signals against historical Kalshi market outcomes.
