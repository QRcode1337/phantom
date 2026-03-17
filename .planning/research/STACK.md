# Technology Stack: Phantom

**Project:** Phantom
**Researched:** 2025-05-24

## Recommended Stack

### Core Framework
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust | 2021 | Performance & Reliability | Essential for high-frequency chaos math calculations and memory-safe async. |

### Linear Algebra & Tensor Ops
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `ndarray` | 0.15 | Matrix manipulation | Efficient multi-dimensional array operations; standard in Rust for scientific computing. |
| `nalgebra` | 0.32 | Linear Solvers | Robust LU decomposition and Ridge Regression solvers (used in ESN training). |

### Infrastructure & Communication
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `tokio` | 1.50 | Async Runtime | Concurrent fetching from Kalshi and Open-Meteo while performing background calculations. |
| `reqwest` | 0.12 | HTTP Client | Reliable client for RESTful API calls to prediction markets and weather services. |
| `serde` | 1.0 | Data Serialization | Seamless JSON handling for API responses and model persistence (saving/loading ESNs). |

### Parallelism
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `rayon` | 1.8 | Parallel processing | Used in FTLE calculation to parallelize slope fitting across millions of state vectors. |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Math | `ndarray` | `nalgebra` alone | `ndarray` is better for data-centric tensor ops; `nalgebra` is preferred for geometric/linear system solving. Using both is optimal. |
| Language | Rust | Python (numpy/pytorch) | Rust provides lower latency and better concurrency management for real-time regime detection. |

## Installation

```bash
# Core dependencies already in Cargo.toml
cargo build --release
```

## Sources

- [Official Rust ndarray documentation](https://docs.rs/ndarray)
- [Official nalgebra documentation](https://nalgebra.org)
- [Kalshi API v2 Documentation](https://trading-api.readme.io/)
- [Open-Meteo API Documentation](https://open-meteo.com/en/docs)
