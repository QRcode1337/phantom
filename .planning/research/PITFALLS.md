# Domain Pitfalls: Phantom

**Domain:** Chaos Math & Prediction Markets
**Researched:** 2025-05-24

## Critical Pitfalls

Mistakes that cause rewrites or major issues.

### Pitfall 1: Spectral Radius Violations
**What goes wrong:** Reservoir states explode or become static.
**Why it happens:** Failing to ensure the Echo State Property (Spectral Radius ρ < 1.0) during weight initialization.
**Consequences:** ESN becomes unstable or loses its "fading memory" capability.
**Prevention:** Use the **Power Method** to estimate the spectral radius and scale the reservoir weights accordingly during initialization.
**Detection:** Monitor `state.std()` for abnormal growth.

### Pitfall 2: Embedding Parameter Mismatch
**What goes wrong:** Chaotic structure is obscured by noise or "stretched" too thin.
**Why it happens:** Choosing the wrong embedding dimension (`m`) or delay (`τ`) for a given time-series.
**Consequences:** Lyapunov estimates become meaningless or noisy.
**Prevention:** Use the **False Nearest Neighbors (FNN)** algorithm for `m` and **Mutual Information (MI)** for `τ`.
**Detection:** High variance in FTLE across sliding windows of the same regime.

## Moderate Pitfalls

### Pitfall 1: Theiler Window Neglect
**What goes wrong:** Underestimating divergence by including temporal neighbors.
**Why it happens:** Nearest-neighbor search finds points that are close in time (on the same trajectory segment) rather than close in state space.
**Prevention:** Implement a `theiler_window` exclusion rule in the VP-Tree search.

### Pitfall 2: Kalshi Bid-Only Orderbooks
**What goes wrong:** Miscalculating the spread or executable price.
**Why it happens:** Kalshi API often only returns one side of the book (bids for "Yes").
**Prevention:** Invert the "No" bids to derive the "Yes" asks (Ask = 100 - Bid_No).

## Minor Pitfalls

### Pitfall 1: Log-Return Scaling
**What goes wrong:** Tiny values in returns leading to numerical precision issues.
**Prevention:** Scale returns (e.g., multiply by 100 or 1000) before embedding if using fixed-precision, though `f64` usually handles this.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Core Math | Numerical instability in slope fitting. | Use RANSAC or robust linear regression for λ estimation. |
| Kalshi Integration | RSA Key-pair management. | Store keys securely (Keychain/Vault) and use a dedicated signing module. |
| Weather Detector | Ensemble model staleness. | Verify `reference_time` in Open-Meteo responses to avoid stale forecasts. |

## Sources

- [Common pitfalls in Lyapunov estimation](https://example.com/lyapunov-pitfalls)
- [Echo State Property Analysis](https://example.com/esn-stability)
- [Kalshi Developer Forums / Community Support](https://kalshi.com/developers)
