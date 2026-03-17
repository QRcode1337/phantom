//! Phantom — Anomaly detection engine
//!
//! Three targets:
//! - ARGUS geospatial (flight tracks, seismic)
//! - Weather markets (Kalshi temperature/precip)
//! - Price regime changes (BTC, prediction market prices)

pub mod ftle;
pub mod detectors;
pub mod signals;
pub mod feeds;
pub mod api;
pub mod daemon;

pub use detectors::{ArgusDetector, WeatherEdgeDetector, PriceRegimeDetector};
pub use signals::KalshiSignalEngine;
