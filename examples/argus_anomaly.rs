//! ARGUS anomaly detection example
//!
//! Demonstrates the ArgusDetector on synthetic flight altitude and seismic data.
//! A stable flight suddenly enters chaotic altitude oscillations — the detector flags it.

use anyhow::Result;
use phantom::ArgusDetector;

fn main() -> Result<()> {
    println!("Phantom ARGUS Anomaly Detector");
    println!("================================\n");

    let detector = ArgusDetector::new();

    // --- Flight track example ---
    // Simulate a flight: 200 steps of stable cruise, then 100 steps of chaotic deviation
    let stable: Vec<f64> = (0..200)
        .map(|i| 35_000.0 + (i as f64 * 0.05).sin() * 50.0 + ((i * 3) % 7) as f64 * 10.0)
        .collect();

    let chaotic: Vec<f64> = (0..100)
        .map(|i| {
            let base = 35_000.0;
            let chaos = ((i * 17 + 5) % 31) as f64 * 400.0 - 6_000.0;
            let osc = (i as f64 * 1.3).sin() * 2_000.0;
            base + chaos + osc
        })
        .collect();

    let full_track: Vec<f64> = stable.iter().chain(chaotic.iter()).cloned().collect();

    let anomaly = detector.analyze_flight_track("UAL-293", &full_track)?;
    println!("✈️  Flight Analysis: UAL-293");
    println!("   Severity:  {:?}", anomaly.severity);
    println!("   Chaos:     {:.4}", anomaly.chaos_score);
    println!("   Regime Δ:  {}", anomaly.regime_changed);
    println!("   → {}\n", anomaly.description);

    // --- Seismic example ---
    // Simulate background microseismic activity followed by foreshock swarm
    let background: Vec<f64> = (0..200)
        .map(|i| 1.0 + ((i * 7) % 5) as f64 * 0.2)
        .collect();

    let swarm: Vec<f64> = (0..150)
        .map(|i| {
            let mag = 2.5 + (i as f64 * 0.4).sin() * 1.5;
            let spike = if i % 11 == 0 { 2.0 } else { 0.0 };
            mag + spike
        })
        .collect();

    let seismic: Vec<f64> = background.iter().chain(swarm.iter()).cloned().collect();

    let seismic_anomaly = detector.analyze_seismic_series(&seismic)?;
    println!("🌍 Seismic Analysis: USGS Feed");
    println!("   Severity:  {:?}", seismic_anomaly.severity);
    println!("   Chaos:     {:.4}", seismic_anomaly.chaos_score);
    println!("   Regime Δ:  {}", seismic_anomaly.regime_changed);
    println!("   → {}\n", seismic_anomaly.description);

    // --- Normal flight (baseline) ---
    let normal_flight: Vec<f64> = (0..300)
        .map(|i| 39_000.0 + (i as f64 * 0.02).sin() * 30.0)
        .collect();

    let normal = detector.analyze_flight_track("SWA-881", &normal_flight)?;
    println!("✈️  Flight Analysis: SWA-881 (baseline)");
    println!("   Severity:  {:?}", normal.severity);
    println!("   Chaos:     {:.4}", normal.chaos_score);
    println!("   → {}\n", normal.description);

    println!("Done. ARGUS anomaly detection complete.");
    Ok(())
}
