//! Kalshi market signal engine
//!
//! Combines FTLE regime detection + weather edge analysis to produce
//! ranked, actionable Kalshi market signals.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::detectors::weather::{WeatherEdgeDetector, WeatherEdge, EdgeConfidence};
use crate::detectors::price::{PriceRegimeDetector, PriceRegimeSignal, TradingSignal};

/// A ranked Kalshi trading opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KalshiSignal {
    pub market_type: String,
    pub direction: String,       // "YES" or "NO"
    pub edge: f64,               // 0.0–0.5
    pub confidence: String,      // "HIGH" / "MEDIUM" / "LOW"
    pub chaos_score: f64,        // FTLE chaos in the underlying series
    pub action: SignalAction,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalAction {
    /// Strong edge — trade it
    Enter,
    /// Edge exists but chaos is too high — watch only
    Watch,
    /// No edge — skip
    Skip,
}

pub struct KalshiSignalEngine {
    pub weather_detector: WeatherEdgeDetector,
    pub price_detector: PriceRegimeDetector,
    /// Minimum edge threshold to enter (not just watch)
    pub enter_threshold: f64,
    /// Max chaos score to allow an entry (too chaotic = watch only)
    pub max_chaos_for_entry: f64,
}

impl Default for KalshiSignalEngine {
    fn default() -> Self {
        Self {
            weather_detector: WeatherEdgeDetector::default(),
            price_detector: PriceRegimeDetector::default(),
            enter_threshold: 0.10,   // need at least 10% edge to enter
            max_chaos_for_entry: 0.65,
        }
    }
}

impl KalshiSignalEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate Kalshi signals from a weather temperature series.
    /// `temps` = hourly ensemble forecast temps (°F or °C, consistent)
    /// `target` = the market threshold
    /// `above` = true if the market is "temp ABOVE target"
    pub fn weather_signal(
        &self,
        temps: &[f64],
        target: f64,
        above: bool,
    ) -> Result<KalshiSignal> {
        let edge = self.weather_detector.analyze_temperature(temps, target, above)?;
        Ok(self.edge_to_signal("weather/temperature", &edge))
    }

    /// Generate a signal from a Kalshi prediction market price series.
    /// Detects regime changes that suggest mispricing.
    pub fn price_regime_signal(&self, market_ticker: &str, prices: &[f64]) -> Result<KalshiSignal> {
        let regime = self.price_detector.analyze(market_ticker, prices)?;
        Ok(self.regime_to_signal(market_ticker, &regime))
    }

    /// Check for a weather regime shift (often precedes large market moves).
    pub fn regime_shift_signal(&self, temps: &[f64]) -> Result<Option<KalshiSignal>> {
        match self.weather_detector.detect_regime_shift(temps)? {
            Some(edge) => Ok(Some(self.edge_to_signal("weather/regime-shift", &edge))),
            None => Ok(None),
        }
    }

    fn edge_to_signal(&self, market_type: &str, edge: &WeatherEdge) -> KalshiSignal {
        let direction = if edge.probability > 0.5 { "YES" } else { "NO" }.to_string();
        let confidence = match edge.confidence {
            EdgeConfidence::High => "HIGH",
            EdgeConfidence::Medium => "MEDIUM",
            EdgeConfidence::Low => "LOW",
        }.to_string();

        let action = if edge.confidence == EdgeConfidence::Low {
            SignalAction::Skip
        } else if edge.edge >= self.enter_threshold && edge.chaos_score <= self.max_chaos_for_entry {
            SignalAction::Enter
        } else if edge.edge > 0.0 {
            SignalAction::Watch
        } else {
            SignalAction::Skip
        };

        KalshiSignal {
            market_type: market_type.to_string(),
            direction,
            edge: edge.edge,
            confidence,
            chaos_score: edge.chaos_score,
            action,
            reason: edge.description.clone(),
        }
    }

    fn regime_to_signal(&self, ticker: &str, regime: &PriceRegimeSignal) -> KalshiSignal {
        let (direction, action) = match &regime.signal {
            TradingSignal::BreakoutSignal => ("YES", SignalAction::Watch),
            TradingSignal::ReversalSignal => ("NO", SignalAction::Watch),
            TradingSignal::Transitioning => ("YES", SignalAction::Watch),
            TradingSignal::Chaotic => ("NO", SignalAction::Skip),
            TradingSignal::Trending => ("YES", SignalAction::Enter),
        };

        let edge = (regime.current_chaos - regime.historical_chaos).abs().min(0.4);

        KalshiSignal {
            market_type: format!("price/{}", ticker),
            direction: direction.to_string(),
            edge,
            confidence: if regime.regime_changed { "HIGH".to_string() } else { "MEDIUM".to_string() },
            chaos_score: regime.current_chaos,
            action,
            reason: regime.description.clone(),
        }
    }

    /// Print a formatted signal report to stdout
    pub fn print_report(&self, signals: &[KalshiSignal]) {
        println!("\n╔══════════════════════════════════════════════╗");
        println!("║         PHANTOM — KALSHI SIGNAL REPORT       ║");
        println!("╚══════════════════════════════════════════════╝\n");

        let enter: Vec<_> = signals.iter().filter(|s| s.action == SignalAction::Enter).collect();
        let watch: Vec<_> = signals.iter().filter(|s| s.action == SignalAction::Watch).collect();

        if enter.is_empty() && watch.is_empty() {
            println!("No actionable signals.");
            return;
        }

        if !enter.is_empty() {
            println!("🟢 ENTER ({}):", enter.len());
            for s in &enter {
                println!("  {} {} | edge={:.1}% chaos={:.3} conf={} | {}",
                    s.direction, s.market_type,
                    s.edge * 100.0, s.chaos_score, s.confidence,
                    s.reason);
            }
        }

        if !watch.is_empty() {
            println!("\n👁  WATCH ({}):", watch.len());
            for s in &watch {
                println!("  {} {} | edge={:.1}% chaos={:.3} | {}",
                    s.direction, s.market_type,
                    s.edge * 100.0, s.chaos_score,
                    &s.reason[..s.reason.len().min(80)]);
            }
        }
        println!();
    }
}
