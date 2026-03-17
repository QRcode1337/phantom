//! Price regime change detector
//!
//! Uses FTLE Lyapunov exponent to detect when a market transitions
//! from trending/stable to chaotic — the regime change is the signal.
//!
//! Works on any price series: BTC, options premiums, prediction market prices.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use ndarray::Array2;
use crate::ftle::{chaos_score, echo_state::{EchoStateNetwork, EchoStateConfig}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRegimeSignal {
    pub asset: String,
    pub current_chaos: f64,
    pub historical_chaos: f64,
    pub regime_changed: bool,
    pub signal: TradingSignal,
    pub predicted_next: Option<f64>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingSignal {
    /// System stable + low chaos → trend-following works
    Trending,
    /// Edge of chaos → prepare for breakout, tighten stops
    Transitioning,
    /// High chaos → mean-reversion, reduce size, watch for reversal
    Chaotic,
    /// Regime just changed stable→chaotic → BREAKOUT signal
    BreakoutSignal,
    /// Regime just changed chaotic→stable → REVERSAL signal
    ReversalSignal,
}

pub struct PriceRegimeDetector {
    pub dt: f64,
    pub regime_threshold: f64,
    pub history_window: usize,
    pub recent_window: usize,
    pub use_esn_prediction: bool,
}

impl Default for PriceRegimeDetector {
    fn default() -> Self {
        Self {
            dt: 1.0,
            regime_threshold: 0.2,
            history_window: 200,
            recent_window: 50,
            use_esn_prediction: true,
        }
    }
}

impl PriceRegimeDetector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze a price series and return a trading signal.
    pub fn analyze(&self, asset: &str, prices: &[f64]) -> Result<PriceRegimeSignal> {
        let min_len = self.history_window + self.recent_window;
        if prices.len() < min_len {
            return Ok(PriceRegimeSignal {
                asset: asset.to_string(),
                current_chaos: 0.0,
                historical_chaos: 0.0,
                regime_changed: false,
                signal: TradingSignal::Trending,
                predicted_next: None,
                description: format!("Insufficient data: need {} points, got {}", min_len, prices.len()),
            });
        }

        let split = prices.len().saturating_sub(self.recent_window);
        let history = &prices[..split];
        let recent = &prices[split..];

        let hist_chaos = chaos_score(history, self.dt)?;
        let recent_chaos = chaos_score(recent, self.dt)?;
        let changed = (recent_chaos - hist_chaos).abs() > self.regime_threshold;

        // Determine signal
        let signal = if changed {
            if recent_chaos > hist_chaos {
                TradingSignal::BreakoutSignal  // was stable, now chaotic → breakout
            } else {
                TradingSignal::ReversalSignal  // was chaotic, now stable → reversal
            }
        } else if recent_chaos > 0.65 {
            TradingSignal::Chaotic
        } else if recent_chaos > 0.35 {
            TradingSignal::Transitioning
        } else {
            TradingSignal::Trending
        };

        // Optional ESN next-price prediction
        let predicted_next = if self.use_esn_prediction && prices.len() >= 100 {
            self.esn_predict(prices).ok()
        } else {
            None
        };

        let description = match &signal {
            TradingSignal::BreakoutSignal => format!(
                "🚨 {} BREAKOUT: regime shifted stable→chaotic (λ: {:.3}→{:.3}). Momentum play, size down.",
                asset, hist_chaos * 2.0, recent_chaos * 2.0
            ),
            TradingSignal::ReversalSignal => format!(
                "🔄 {} REVERSAL: chaos→stable (λ: {:.3}→{:.3}). Mean reversion setup.",
                asset, hist_chaos * 2.0, recent_chaos * 2.0
            ),
            TradingSignal::Chaotic => format!(
                "⚡ {} CHAOTIC (λ={:.3}): high unpredictability. Reduce size, widen stops.",
                asset, recent_chaos * 2.0
            ),
            TradingSignal::Transitioning => format!(
                "⚠️  {} TRANSITIONING (λ={:.3}): approaching regime boundary. Watch closely.",
                asset, recent_chaos * 2.0
            ),
            TradingSignal::Trending => format!(
                "✅ {} STABLE (λ={:.3}): trend-following conditions nominal.",
                asset, recent_chaos * 2.0
            ),
        };

        Ok(PriceRegimeSignal {
            asset: asset.to_string(),
            current_chaos: recent_chaos,
            historical_chaos: hist_chaos,
            regime_changed: changed,
            signal,
            predicted_next,
            description,
        })
    }

    fn esn_predict(&self, prices: &[f64]) -> Result<f64> {
        let n = prices.len().min(100);
        let config = EchoStateConfig {
            reservoir_size: 50,
            spectral_radius: 0.9,
            input_scaling: 0.1,
            ..EchoStateConfig::default()
        };
        let mut esn = EchoStateNetwork::new(config, 1, 1)?;

        let train_n = n - 1;
        let inputs = Array2::from_shape_fn((train_n, 1), |(i, _)| prices[i]);
        let targets = Array2::from_shape_fn((train_n, 1), |(i, _)| prices[i + 1]);
        esn.train(inputs.view(), targets.view(), 10)?;

        let last = Array2::from_shape_fn((1, 1), |_| *prices.last().unwrap());
        let pred = esn.predict_step(last.row(0))?;
        Ok(pred[0])
    }
}
