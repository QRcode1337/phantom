//! ARGUS geospatial anomaly detector
//!
//! Detects anomalous patterns in:
//! - Flight trajectory deviations (OpenSky/ADS-B feeds)
//! - Seismic event clustering (USGS feed)
//! - Correlated multi-feed events (flight + weather + seismic)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::ftle::{chaos_score, regime_changed};

/// A geospatial data point (lat, lon, altitude, timestamp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub ts: i64, // unix timestamp ms
}

/// Result of ARGUS anomaly analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgusAnomaly {
    pub feed: String,
    pub severity: AnomalySeverity,
    pub chaos_score: f64,
    pub regime_changed: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AnomalySeverity {
    pub fn from_score(score: f64, regime_changed: bool) -> Self {
        match (score, regime_changed) {
            (s, true) if s > 0.6 => Self::Critical,
            (s, _) if s > 0.7 => Self::High,
            (s, true) if s > 0.3 => Self::Medium,
            (s, _) if s > 0.4 => Self::Medium,
            _ => Self::Low,
        }
    }
}

pub struct ArgusDetector {
    /// Minimum series length to run analysis
    pub min_series_len: usize,
    /// Sampling interval in seconds
    pub dt: f64,
    /// Regime change threshold (0.0–1.0)
    pub regime_threshold: f64,
    /// History window size for regime comparison
    pub history_window: usize,
}

impl Default for ArgusDetector {
    fn default() -> Self {
        Self {
            min_series_len: 50,
            dt: 1.0,        // 1-second ADS-B update interval
            regime_threshold: 0.25,
            history_window: 100,
        }
    }
}

impl ArgusDetector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze a flight track altitude series for chaotic deviations.
    /// Feed in altitude readings over time from a single aircraft.
    pub fn analyze_flight_track(&self, feed_id: &str, altitudes: &[f64]) -> Result<ArgusAnomaly> {
        if altitudes.len() < self.min_series_len {
            return Ok(ArgusAnomaly {
                feed: feed_id.to_string(),
                severity: AnomalySeverity::Low,
                chaos_score: 0.0,
                regime_changed: false,
                description: format!("Insufficient data ({} points)", altitudes.len()),
            });
        }

        let score = chaos_score(altitudes, self.dt)?;

        // Split series into history vs recent for regime detection
        let split = altitudes.len().saturating_sub(self.history_window);
        let changed = if split > 20 {
            regime_changed(&altitudes[..split], &altitudes[split..], self.dt, self.regime_threshold)?
        } else {
            false
        };

        let severity = AnomalySeverity::from_score(score, changed);
        let description = match &severity {
            AnomalySeverity::Critical => format!("CRITICAL: Flight {} showing chaotic trajectory + regime shift (λ={:.3})", feed_id, score * 2.0),
            AnomalySeverity::High => format!("HIGH: Flight {} trajectory is strongly chaotic (λ={:.3})", feed_id, score * 2.0),
            AnomalySeverity::Medium => format!("MEDIUM: Flight {} showing elevated chaos (λ={:.3})", feed_id, score * 2.0),
            AnomalySeverity::Low => format!("LOW: Flight {} nominal (λ={:.3})", feed_id, score * 2.0),
        };

        Ok(ArgusAnomaly { feed: feed_id.to_string(), severity, chaos_score: score, regime_changed: changed, description })
    }

    /// Analyze seismic event magnitudes for anomalous clustering patterns.
    pub fn analyze_seismic_series(&self, magnitudes: &[f64]) -> Result<ArgusAnomaly> {
        if magnitudes.len() < self.min_series_len {
            return Ok(ArgusAnomaly {
                feed: "usgs-seismic".to_string(),
                severity: AnomalySeverity::Low,
                chaos_score: 0.0,
                regime_changed: false,
                description: "Insufficient seismic data".to_string(),
            });
        }

        let score = chaos_score(magnitudes, self.dt)?;
        let split = magnitudes.len().saturating_sub(self.history_window);
        let changed = if split > 20 {
            regime_changed(&magnitudes[..split], &magnitudes[split..], self.dt, self.regime_threshold)?
        } else {
            false
        };

        let severity = AnomalySeverity::from_score(score, changed);
        Ok(ArgusAnomaly {
            feed: "usgs-seismic".to_string(),
            severity,
            chaos_score: score,
            regime_changed: changed,
            description: format!("Seismic pattern chaos={:.3} regime_shift={}", score, changed),
        })
    }
}
