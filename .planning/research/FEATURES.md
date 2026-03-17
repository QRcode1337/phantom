# Feature Landscape: Phantom

**Domain:** Chaos Math & Prediction Markets
**Researched:** 2025-05-24

## Table Stakes

Features users expect in an analytical engine for dynamical systems and markets.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **FTLE Calculation** | Core metric for local divergence and predictability. | Medium | Requires efficient nearest-neighbor search (VP-tree). |
| **ESN Training** | Fast training for short-term prediction. | Medium | Uses Ridge regression on augmented reservoir states. |
| **Data Fetchers** | Access to Kalshi and Open-Meteo feeds. | Low | Async HTTP fetching with `reqwest`. |
| **Phase Space Embedding** | Required for univariate time-series (Taken's theorem). | Low | Crucial for 1D price data. |

## Differentiators

Features that set 'phantom' apart from generic ML models.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Regime Shift Detection** | Detects transitions (stable → chaotic) which signal market breakouts. | High | Compares historical vs. recent Lyapunov exponents. |
| **Ensemble Model Divergence** | Uses model disagreement (GFS vs ECMWF) as a trading edge for Kalshi. | Medium | Weather-specific edge. |
| **Autonomous ESN Prediction** | Provides next-step forecasts based on learned reservoir dynamics. | High | Requires careful hyperparameter tuning (spectral radius). |
| **VP-Tree Acceleration** | High-speed neighbor search for large trajectories. | Medium | Optimized with `rayon` for multi-core performance. |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Long-term Forecasting** | Chaos math is inherently limited by the predictability horizon (1/λ). | Focus on short-term regime changes and volatility spikes. |
| **Traditional Deep Learning** | Backpropagation is too slow for real-time edge-of-chaos detection. | Stick to Reservoir Computing (ESN) for efficiency. |
| **Global Prediction** | Attempting to predict the entire market state. | Focus on specific niches (Kalshi weather/price) where edges are clearer. |

## Feature Dependencies

```
Data Fetchers → Phase Space Embedding → FTLE / ESN → Regime Detection → Trading Signal
```

## MVP Recommendation

Prioritize:
1. **Regime Shift Detection (Price):** Build the most general detector first to prove the "breakout" signal.
2. **Weather Ensemble Divergence:** High-value niche with clear external data sources.
3. **Kalshi API Integration:** Essential for closing the loop from analysis to action.

Defer: **Autonomous Prediction (Multi-step):** Start with next-step forecasting before attempting multi-step trajectories.

## Sources

- [Lyapunov exponent in financial time series (Research)](https://example.com/research-paper)
- [Echo State Networks for Chaotic Series Prediction](http://www.scholarpedia.org/article/Echo_state_network)
- [Kalshi Market Rules & API](https://kalshi.com)
