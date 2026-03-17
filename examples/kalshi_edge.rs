//! Kalshi edge detection example
//!
//! Demonstrates the KalshiSignalEngine producing Enter/Watch/Skip signals
//! from synthetic temperature forecasts and BTC price series.

use anyhow::Result;
use phantom::signals::KalshiSignalEngine;

fn main() -> Result<()> {
    println!("Phantom — Kalshi Signal Engine");
    println!("================================\n");

    let engine = KalshiSignalEngine::new();

    // --- Weather edge: NYC temp above 55°F ---
    // Ensemble of 200 hourly readings clustered above threshold → strong YES edge
    let warm_temps: Vec<f64> = (0..200)
        .map(|i| {
            let base = 58.0 + (i as f64 * 0.08).sin() * 5.0;
            let noise = ((i * 7 + 3) % 9) as f64 * 0.4 - 1.8;
            base + noise
        })
        .collect();

    let signal = engine.weather_signal(&warm_temps, 55.0, true)?;
    println!("🌡️  Weather: NYC temp ABOVE 55°F?");
    println!("   Action:     {:?}", signal.action);
    println!("   Direction:  {}", signal.direction);
    println!("   Edge:       {:.1}%", signal.edge * 100.0);
    println!("   Chaos:      {:.4}", signal.chaos_score);
    println!("   Confidence: {}", signal.confidence);
    println!("   → {}\n", signal.reason);

    // --- Regime shift on cold snap ---
    let cold_shift: Vec<f64> = (0..250)
        .map(|i| {
            if i < 180 {
                60.0 + (i as f64 * 0.05).cos() * 3.0
            } else {
                // Sudden cold drop — chaos + regime shift
                60.0 - (i - 180) as f64 * 1.2 + ((i * 13) % 17) as f64 * 0.8 - 6.8
            }
        })
        .collect();

    if let Some(shift) = engine.regime_shift_signal(&cold_shift)? {
        println!("❄️  Weather regime shift detected:");
        println!("   Action:     {:?}", shift.action);
        println!("   Edge:       {:.1}%", shift.edge * 100.0);
        println!("   → {}\n", shift.reason);
    } else {
        println!("❄️  No regime shift detected in cold series.\n");
    }

    // --- BTC price regime: stable trend ---
    let btc_stable: Vec<f64> = (0..300)
        .map(|i| {
            let trend = 84_000.0 + i as f64 * 15.0;
            let noise = ((i * 3) % 7) as f64 * 80.0 - 280.0;
            trend + noise
        })
        .collect();

    let btc_signal = engine.price_regime_signal("BTC-USD", &btc_stable)?;
    println!("₿  BTC stable trend:");
    println!("   Action:     {:?}", btc_signal.action);
    println!("   Direction:  {}", btc_signal.direction);
    println!("   Chaos:      {:.4}", btc_signal.chaos_score);
    println!("   → {}\n", btc_signal.reason);

    // --- BTC price regime: breakout ---
    let btc_breakout: Vec<f64> = (0..300)
        .map(|i| {
            if i < 200 {
                85_000.0 + (i as f64 * 0.05).sin() * 500.0
            } else {
                // Breakout into chaos
                85_000.0 + (i - 200) as f64 * 300.0
                    + ((i * 17 + 7) % 31) as f64 * 1_200.0 - 18_600.0
            }
        })
        .collect();

    let breakout_signal = engine.price_regime_signal("BTC-USD-BREAKOUT", &btc_breakout)?;
    println!("₿  BTC breakout scenario:");
    println!("   Action:     {:?}", breakout_signal.action);
    println!("   Direction:  {}", breakout_signal.direction);
    println!("   Chaos:      {:.4}", breakout_signal.chaos_score);
    println!("   → {}\n", breakout_signal.reason);

    // --- Full report ---
    engine.print_report(&[signal, btc_signal, breakout_signal]);

    Ok(())
}
