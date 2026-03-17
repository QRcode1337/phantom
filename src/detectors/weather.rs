//! Weather market edge detector
//!
//! Uses FTLE chaos analysis + Open-Meteo ensemble model divergence
//! to find pricing inefficiencies in Kalshi weather markets.
//!
//! Core insight: when GFS and ECMWF models DISAGREE, the Kalshi market
//! is likely mispriced. Model spread = your edge.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::ftle::{chaos_score, regime_changed};

/// Open-Meteo ensemble response (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleForecast {
    pub latitude: f64,
    pub longitude: f64,
    /// Hourly temperature readings per model
    pub temperature_2m: Vec<f64>,
    /// Hourly precipitation per model
    pub precipitation: Vec<f64>,
}

/// A detected weather market edge opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEdge {
    pub market_type: WeatherMarketType,
    /// Estimated probability our model gives (0.0–1.0)
    pub probability: f64,
    /// Model spread (standard deviation across ensemble members)
    pub model_spread: f64,
    /// Chaos score of the temperature/precip series
    pub chaos_score: f64,
    /// Edge = how far our probability is from 0.5 (pure chance)
    pub edge: f64,
    /// Confidence based on model agreement
    pub confidence: EdgeConfidence,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherMarketType {
    TemperatureAbove { target: f64 },
    TemperatureBelow { target: f64 },
    PrecipitationAbove { threshold_mm: f64 },
    RegimeShift,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeConfidence {
    High,   // model_spread < 1.0°C — models agree, trust the probability
    Medium, // model_spread 1.0–3.0°C
    Low,    // model_spread > 3.0°C — too uncertain to trade
}

impl EdgeConfidence {
    pub fn from_spread(spread: f64) -> Self {
        if spread < 1.0 {
            Self::High
        } else if spread < 3.0 {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

pub struct WeatherEdgeDetector {
    pub dt: f64,
    pub regime_threshold: f64,
    /// Minimum edge to bother reporting (0.0–0.5)
    pub min_edge: f64,
}

impl Default for WeatherEdgeDetector {
    fn default() -> Self {
        Self {
            dt: 1.0,           // hourly data
            regime_threshold: 0.2,
            min_edge: 0.08,    // only report if we have >8% edge
        }
    }
}

impl WeatherEdgeDetector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze temperature series and compute probability of being above target.
    pub fn analyze_temperature(
        &self,
        temps: &[f64],
        target: f64,
        above: bool,
    ) -> Result<WeatherEdge> {
        // Probability from ensemble: fraction of readings above/below target
        let count = if above {
            temps.iter().filter(|&&t| t > target).count()
        } else {
            temps.iter().filter(|&&t| t < target).count()
        };
        let probability = count as f64 / temps.len() as f64;

        // Model spread (std dev)
        let mean = temps.iter().sum::<f64>() / temps.len() as f64;
        let variance = temps.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / temps.len() as f64;
        let spread = variance.sqrt();

        // Chaos score on the temperature series
        let chaos = if temps.len() >= 50 {
            chaos_score(temps, self.dt).unwrap_or(0.0)
        } else {
            0.0
        };

        let edge = (probability - 0.5).abs();
        let confidence = EdgeConfidence::from_spread(spread);

        let market_type = if above {
            WeatherMarketType::TemperatureAbove { target }
        } else {
            WeatherMarketType::TemperatureBelow { target }
        };

        let description = format!(
            "Temp {} {:.1}°C: p={:.1}% spread={:.2}°C chaos={:.3} edge={:.1}% [{}]",
            if above { ">" } else { "<" },
            target,
            probability * 100.0,
            spread,
            chaos,
            edge * 100.0,
            match &confidence { EdgeConfidence::High => "HIGH CONF", EdgeConfidence::Medium => "MED CONF", EdgeConfidence::Low => "LOW CONF" }
        );

        Ok(WeatherEdge { market_type, probability, model_spread: spread, chaos_score: chaos, edge, confidence, description })
    }

    /// Detect weather regime shifts — e.g. transition from stable high pressure to chaotic storm pattern.
    /// These often precede large weather market moves.
    pub fn detect_regime_shift(&self, temps: &[f64]) -> Result<Option<WeatherEdge>> {
        if temps.len() < 100 {
            return Ok(None);
        }

        let split = temps.len() / 2;
        let changed = regime_changed(&temps[..split], &temps[split..], self.dt, self.regime_threshold)?;

        if !changed {
            return Ok(None);
        }

        let recent_score = chaos_score(&temps[split..], self.dt)?;
        let edge = (recent_score - 0.5).abs().min(0.4);

        Ok(Some(WeatherEdge {
            market_type: WeatherMarketType::RegimeShift,
            probability: if recent_score > 0.5 { 0.5 + edge } else { 0.5 - edge },
            model_spread: 0.0,
            chaos_score: recent_score,
            edge,
            confidence: if edge > 0.2 { EdgeConfidence::High } else { EdgeConfidence::Medium },
            description: format!(
                "REGIME SHIFT detected — weather pattern transitioning (chaos={:.3}). Watch for large temp/precip moves.",
                recent_score
            ),
        }))
    }

    /// Filter edges worth trading (above min_edge threshold and not Low confidence)
    pub fn tradeable_edges(&self, edges: Vec<WeatherEdge>) -> Vec<WeatherEdge> {
        edges.into_iter()
            .filter(|e| e.edge >= self.min_edge && e.confidence != EdgeConfidence::Low)
            .collect()
    }
}
