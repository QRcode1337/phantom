//! Integration tests for the PHANTOM anomaly detection engine.
//!
//! Covers the FTLE chaos math layer, all three detectors (WeatherEdge,
//! PriceRegime, Argus), and the Kalshi signal engine.
//!
//! Run with:
//!   cargo test --test integration
//!
//! These tests are purely offline — no network calls are made.
//!
//! # Notes on `dt` configuration
//!
//! The FTLE Lyapunov estimator is sensitive to the relationship between the
//! generation step used to create the test series and the analysis `dt` passed
//! to `chaos_score`.  All detectors expose a public `dt` field so tests can
//! use a consistent value.  Tests that exercise the *chaotic* branch configure
//! `dt = 0.01` to match the Lorenz integration step; tests that exercise the
//! *stable* / *low-chaos* branch leave `dt` at the default `1.0`.

use phantom::{
    ftle::{chaos_score, regime_changed},
    WeatherEdgeDetector, PriceRegimeDetector, ArgusDetector, KalshiSignalEngine,
};
use phantom::detectors::weather::EdgeConfidence;
use phantom::detectors::argus::AnomalySeverity;
use phantom::detectors::price::TradingSignal;
use phantom::signals::kalshi::SignalAction;

// ---------------------------------------------------------------------------
// Test data generators
// ---------------------------------------------------------------------------

/// Lorenz attractor — ground-truth chaotic time series.
///
/// Uses the standard parameters (sigma=10, rho=28, beta=8/3) and Euler
/// integration with step `dt`.  Returns the X coordinate at each step.
///
/// When `dt = 0.01` this produces a well-sampled chaotic trajectory;
/// passing the same `dt` to `chaos_score` yields a score of 1.0.
fn lorenz_series(n: usize, dt: f64) -> Vec<f64> {
    let (sigma, rho, beta) = (10.0_f64, 28.0_f64, 8.0 / 3.0);
    let (mut x, mut y, mut z) = (1.0_f64, 1.0_f64, 1.0_f64);
    let mut series = Vec::with_capacity(n);
    for _ in 0..n {
        let dx = sigma * (y - x) * dt;
        let dy = (x * (rho - z) - y) * dt;
        let dz = (x * y - beta * z) * dt;
        x += dx;
        y += dy;
        z += dz;
        series.push(x);
    }
    series
}

/// Stable sinusoidal series — low chaos expected.
///
/// A pure sinusoid is fully periodic and predictable; `chaos_score` returns
/// a value well below 0.3 regardless of the `dt` used.
fn stable_series(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| (i as f64 * 0.1).sin() * 10.0 + 50.0)
        .collect()
}

/// Stable first half, then chaotic second half — regime shift at the midpoint.
///
/// The stable portion is a sinusoid; the chaotic portion is a Lorenz X series
/// generated with `dt = 0.01`.  The caller should also pass `dt = 0.01` to
/// the analysis function to ensure the chaos contrast is visible.
fn regime_shift_series(n: usize) -> Vec<f64> {
    let half = n / 2;
    let mut s: Vec<f64> = (0..half)
        .map(|i| (i as f64 * 0.1).sin() * 5.0 + 50.0)
        .collect();
    s.extend(lorenz_series(n - half, 0.01));
    s
}

/// Temperatures all clearly above `target` by at least `margin` degrees.
fn temps_above(n: usize, target: f64, margin: f64) -> Vec<f64> {
    (0..n)
        .map(|i| target + margin + (i as f64 * 0.05).sin())
        .collect()
}

/// Temperatures all clearly below `target` by at least `margin` degrees.
fn temps_below(n: usize, target: f64, margin: f64) -> Vec<f64> {
    (0..n)
        .map(|i| target - margin + (i as f64 * 0.05).sin() * 0.1)
        .collect()
}

/// Monotone ascending price series — each step increments by a tiny constant.
///
/// A perfectly monotone series has zero neighbor-divergence in the embedded
/// phase space, so `chaos_score` returns 0.0 and `PriceRegimeDetector`
/// classifies it as `TradingSignal::Trending`.
fn trending_prices(n: usize) -> Vec<f64> {
    (0..n).map(|i| 100.0 + i as f64 * 0.01).collect()
}

/// Stable altitudes with minimal variation — nominal cruise flight profile.
fn stable_altitudes(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| 35_000.0 + (i as f64 * 0.05).sin() * 50.0)
        .collect()
}

// ---------------------------------------------------------------------------
// FTLE core tests (1–5)
// ---------------------------------------------------------------------------

#[test]
fn test_lorenz_is_chaotic() {
    // The Lorenz attractor is the canonical chaotic system.
    // With 500 points at dt=0.01, the FTLE estimator should classify it above 0.5.
    let series = lorenz_series(500, 0.01);
    let score = chaos_score(&series, 0.01)
        .expect("chaos_score should succeed on a 500-point Lorenz series");
    assert!(
        score > 0.5,
        "Lorenz chaos score should exceed 0.5, got {:.4}",
        score
    );
}

#[test]
fn test_stable_sine_is_not_chaotic() {
    // A pure sinusoid is perfectly predictable — chaos score should be well below 0.3.
    let series = stable_series(300);
    let score = chaos_score(&series, 1.0)
        .expect("chaos_score should succeed on a 300-point sinusoid");
    assert!(
        score < 0.3,
        "Stable sinusoidal series should have chaos score < 0.3, got {:.4}",
        score
    );
}

#[test]
fn test_regime_change_detected() {
    // History = stable sinusoid, recent = Lorenz attractor.
    // Both windows use dt=0.01.  The stable window scores ~0.07 and the Lorenz
    // window scores 1.0, so the absolute difference (~0.93) far exceeds the
    // threshold of 0.15.
    let history = stable_series(200);
    let recent = lorenz_series(200, 0.01);
    let changed = regime_changed(&history, &recent, 0.01, 0.15)
        .expect("regime_changed should succeed with 200-point windows");
    assert!(
        changed,
        "regime_changed should detect the stable→chaotic transition (sine→Lorenz)"
    );
}

#[test]
fn test_no_regime_change_in_stable() {
    // Two consecutive windows of the same sinusoidal series should not differ
    // in chaos score enough to trigger a regime change.
    let full = stable_series(400);
    let history = full[..200].to_vec();
    let recent = full[200..].to_vec();
    let changed = regime_changed(&history, &recent, 0.01, 0.15)
        .expect("regime_changed should succeed on stable windows");
    assert!(
        !changed,
        "Two stable sinusoidal windows should NOT produce a regime change signal"
    );
}

#[test]
fn test_short_series_errors() {
    // The FTLE pipeline embeds the series into a phase space (m=3, tau=1),
    // then needs at least k_fit+2=14 embedded vectors to fit a slope, and
    // the Theiler window (w=20) further restricts valid neighbor pairs.
    // A 5-point series cannot satisfy these constraints — an Err is expected.
    let tiny: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let result = chaos_score(&tiny, 0.01);
    assert!(
        result.is_err(),
        "chaos_score on a 5-point series should return an error, got Ok({:?})",
        result.ok()
    );
}

// ---------------------------------------------------------------------------
// WeatherEdgeDetector tests (6–10)
// ---------------------------------------------------------------------------

#[test]
fn test_weather_above_threshold() {
    // All temperatures are at least 5°F above the target market threshold.
    // Every reading passes the "above" filter, so probability should be > 0.95.
    let detector = WeatherEdgeDetector::new();
    let temps = temps_above(120, 70.0, 5.0); // all ≈ 75–76°F, target = 70°F
    let edge = detector
        .analyze_temperature(&temps, 70.0, true)
        .expect("analyze_temperature should succeed");
    assert!(
        edge.probability > 0.95,
        "When all temps are above target, probability should exceed 0.95, got {:.4}",
        edge.probability
    );
    assert_eq!(edge.edge, (edge.probability - 0.5).abs());
}

#[test]
fn test_weather_below_threshold() {
    // All temperatures are at least 8°F below the target.
    // Every reading passes the "below" filter, so probability should be > 0.95.
    let detector = WeatherEdgeDetector::new();
    let temps = temps_below(120, 40.0, 8.0); // all ≈ 32°F, target = 40°F
    let edge = detector
        .analyze_temperature(&temps, 40.0, false)
        .expect("analyze_temperature should succeed");
    assert!(
        edge.probability > 0.95,
        "When all temps are below target, probability should exceed 0.95, got {:.4}",
        edge.probability
    );
}

#[test]
fn test_weather_regime_shift_detected() {
    // A 300-point series with a stable sinusoidal first half and a Lorenz
    // second half should trigger detect_regime_shift.
    //
    // The detector's dt is set to 0.01 to match the Lorenz generation step,
    // so both windows are scored consistently.  The chaos difference between
    // the two halves (~0.07 vs 1.0) clearly exceeds the default threshold (0.2).
    let mut detector = WeatherEdgeDetector::new();
    detector.dt = 0.01;

    let temps = regime_shift_series(300); // 150 stable + 150 Lorenz
    let result = detector
        .detect_regime_shift(&temps)
        .expect("detect_regime_shift should not return an error");
    assert!(
        result.is_some(),
        "A stable→chaotic series should trigger a regime shift detection"
    );
    let edge = result.unwrap();
    assert!(
        edge.chaos_score > 0.5,
        "Detected regime shift chaos_score should exceed 0.5, got {:.4}",
        edge.chaos_score
    );
}

#[test]
fn test_weather_no_regime_shift() {
    // A purely stable sinusoidal series should not trigger a regime shift,
    // because both halves of the series have essentially the same (near-zero)
    // chaos score.
    let mut detector = WeatherEdgeDetector::new();
    detector.dt = 0.01;

    let temps = stable_series(300);
    let result = detector
        .detect_regime_shift(&temps)
        .expect("detect_regime_shift should not error on stable data");
    assert!(
        result.is_none(),
        "Stable sinusoidal temps should NOT produce a regime shift signal"
    );
}

#[test]
fn test_weather_edge_confidence_low_when_high_spread() {
    // Interleaving extreme hot and cold values forces model_spread (std dev)
    // far above 3°C — that maps to EdgeConfidence::Low per the threshold table.
    let detector = WeatherEdgeDetector::new();
    let temps: Vec<f64> = (0..120)
        .map(|i| if i % 2 == 0 { 10.0_f64 } else { 90.0_f64 })
        .collect();
    let edge = detector
        .analyze_temperature(&temps, 50.0, true)
        .expect("analyze_temperature should succeed");
    assert_eq!(
        edge.confidence,
        EdgeConfidence::Low,
        "Very high spread (>>3°C) should yield Low confidence, got {:?}",
        edge.confidence
    );
}

// ---------------------------------------------------------------------------
// PriceRegimeDetector tests (11–13)
// ---------------------------------------------------------------------------

#[test]
fn test_price_trending_signal() {
    // A monotone ascending price series has zero phase-space divergence.
    // chaos_score returns 0.0, so the signal is Trending.
    // Default windows: history_window=200, recent_window=50 → need 250+ points.
    let detector = PriceRegimeDetector::new(); // dt=1.0 default
    let prices = trending_prices(300);
    let signal = detector
        .analyze("BTC-TEST", &prices)
        .expect("analyze should succeed on a 300-point price series");
    assert_eq!(
        signal.signal,
        TradingSignal::Trending,
        "Monotone ascending prices should produce a Trending signal, got {:?}",
        signal.signal
    );
    assert!(
        signal.current_chaos < 0.4,
        "Trending prices should have low current chaos, got {:.4}",
        signal.current_chaos
    );
}

#[test]
fn test_price_breakout_signal() {
    // History (200 points, monotone) followed by a Lorenz tail (100 points).
    // With dt=0.01 the history scores 0.0 and the recent window scores 1.0,
    // producing a regime change → BreakoutSignal.
    //
    // recent_window is increased to 100 so the Lorenz tail is long enough
    // for the FTLE estimator to find valid nearest-neighbor pairs.
    let mut detector = PriceRegimeDetector::new();
    detector.dt = 0.01;
    detector.recent_window = 100;
    detector.history_window = 200;

    let mut prices = trending_prices(200);
    prices.extend(lorenz_series(100, 0.01));

    let signal = detector
        .analyze("BTC-BREAKOUT", &prices)
        .expect("analyze should succeed");
    assert_eq!(
        signal.signal,
        TradingSignal::BreakoutSignal,
        "Stable→chaotic price series should produce BreakoutSignal, got {:?}",
        signal.signal
    );
    assert!(
        signal.regime_changed,
        "regime_changed flag should be true for a breakout scenario"
    );
    assert!(
        signal.current_chaos > signal.historical_chaos,
        "Recent chaos ({:.4}) should exceed historical chaos ({:.4}) in a breakout",
        signal.current_chaos,
        signal.historical_chaos
    );
}

#[test]
fn test_price_insufficient_data() {
    // A short series (50 points, well below history_window+recent_window=250)
    // should return a graceful fallback rather than panicking or erroring.
    let detector = PriceRegimeDetector::new();
    let prices = trending_prices(50);
    let signal = detector
        .analyze("BTC-SHORT", &prices)
        .expect("analyze should return Ok even with insufficient data");
    assert!(
        !signal.description.is_empty(),
        "Insufficient data result should include a description"
    );
    assert_eq!(
        signal.asset, "BTC-SHORT",
        "Asset name should be preserved in the fallback result"
    );
    // The documented fallback for insufficient data is Trending with no chaos.
    assert_eq!(
        signal.signal,
        TradingSignal::Trending,
        "Insufficient data should fall back to Trending signal"
    );
    assert!(!signal.regime_changed);
}

// ---------------------------------------------------------------------------
// ArgusDetector tests (14–16)
// ---------------------------------------------------------------------------

#[test]
fn test_flight_nominal() {
    // Stable cruise altitudes vary by only ±50 ft — essentially predictable.
    // The default detector (dt=1.0) scores this near 0.0 → Low severity.
    let detector = ArgusDetector::new(); // dt=1.0 default
    let altitudes = stable_altitudes(200);
    let anomaly = detector
        .analyze_flight_track("FLIGHT-NOMINAL", &altitudes)
        .expect("analyze_flight_track should succeed on stable altitudes");
    assert_eq!(
        anomaly.severity,
        AnomalySeverity::Low,
        "Stable cruise altitudes should yield Low severity, got {:?}",
        anomaly.severity
    );
    assert_eq!(anomaly.feed, "FLIGHT-NOMINAL");
    assert!(
        anomaly.chaos_score < 0.4,
        "Nominal flight chaos score should be below 0.4, got {:.4}",
        anomaly.chaos_score
    );
}

#[test]
fn test_flight_chaotic() {
    // A Lorenz-derived altitude profile is strongly chaotic.
    // With dt=0.01 (matching the generation step) the full-series chaos score
    // reaches 1.0 → High severity.  The regime_changed flag may also be set
    // (if the two halves differ enough), which would elevate to Critical.
    let mut detector = ArgusDetector::new();
    detector.dt = 0.01;

    let altitudes = lorenz_series(300, 0.01);
    let anomaly = detector
        .analyze_flight_track("FLIGHT-CHAOS", &altitudes)
        .expect("analyze_flight_track should succeed on a Lorenz altitude series");

    let is_elevated = matches!(
        anomaly.severity,
        AnomalySeverity::High | AnomalySeverity::Critical
    );
    assert!(
        is_elevated,
        "Chaotic altitude profile should yield High or Critical severity, got {:?}",
        anomaly.severity
    );
    assert!(
        anomaly.chaos_score > 0.5,
        "Chaotic flight should have chaos_score > 0.5, got {:.4}",
        anomaly.chaos_score
    );
}

#[test]
fn test_seismic_anomaly() {
    // Lorenz-based seismic magnitudes represent a genuinely irregular, chaotic
    // event sequence.  With dt=0.01 the chaos score is 1.0 → elevated severity.
    // Absolute values are taken so magnitudes stay positive and plausible.
    let mut detector = ArgusDetector::new();
    detector.dt = 0.01;

    let magnitudes: Vec<f64> = lorenz_series(300, 0.01)
        .into_iter()
        .map(|v| v.abs() * 0.5 + 0.5)
        .collect();

    let anomaly = detector
        .analyze_seismic_series(&magnitudes)
        .expect("analyze_seismic_series should succeed on 300-point data");

    assert_eq!(anomaly.feed, "usgs-seismic");
    assert!(
        anomaly.chaos_score > 0.5,
        "Chaotic seismic data should produce chaos_score > 0.5, got {:.4}",
        anomaly.chaos_score
    );
    let is_elevated = matches!(
        anomaly.severity,
        AnomalySeverity::Medium | AnomalySeverity::High | AnomalySeverity::Critical
    );
    assert!(
        is_elevated,
        "Lorenz-based seismic series should yield elevated severity, got {:?}",
        anomaly.severity
    );
}

// ---------------------------------------------------------------------------
// KalshiSignalEngine tests (17–18)
// ---------------------------------------------------------------------------

#[test]
fn test_kalshi_weather_signal_action() {
    // Case 1: Temperatures all well above the target produce probability ≈ 1.0,
    // edge ≈ 0.5.  With spread < 1°C (all readings tightly clustered around
    // target + margin + small sine wobble) confidence is High or Medium, and
    // edge (0.5) exceeds enter_threshold (0.10) → Enter.
    let engine = KalshiSignalEngine::new();

    let temps_high = temps_above(120, 60.0, 10.0); // all ≈ 70°F, target = 60°F
    let signal = engine
        .weather_signal(&temps_high, 60.0, true)
        .expect("weather_signal should succeed");
    assert_ne!(
        signal.action,
        SignalAction::Skip,
        "A clear above-threshold temperature signal should be Enter or Watch, not Skip"
    );
    assert_eq!(
        signal.direction, "YES",
        "Temps consistently above target should produce direction=YES"
    );

    // Case 2: Temperatures split exactly 50/50 above and below the target
    // produce edge = |0.5 - 0.5| = 0.0, which is below enter_threshold → Skip.
    let temps_mixed: Vec<f64> = (0..120)
        .map(|i| if i % 2 == 0 { 59.9_f64 } else { 60.1_f64 })
        .collect();
    let signal_mixed = engine
        .weather_signal(&temps_mixed, 60.0, true)
        .expect("weather_signal should succeed on mixed temps");
    assert_eq!(
        signal_mixed.action,
        SignalAction::Skip,
        "A zero-edge signal should be Skip, got {:?} (edge={:.4})",
        signal_mixed.action,
        signal_mixed.edge
    );
}

#[test]
fn test_kalshi_price_regime_signal() {
    // Verify that the signal engine correctly routes Trending → Enter and
    // BreakoutSignal → Watch.
    //
    // The price_detector is configured with dt=0.01 and recent_window=100 so
    // the Lorenz tail in the breakout scenario produces a measurable chaos score.
    let mut engine = KalshiSignalEngine::new();
    engine.price_detector.dt = 0.01;
    engine.price_detector.recent_window = 100;
    engine.price_detector.history_window = 200;

    // Trending: 300 points of monotone prices → Trending → Enter, direction=YES.
    let prices_stable = trending_prices(300);
    let signal_stable = engine
        .price_regime_signal("KXBTC-TREND", &prices_stable)
        .expect("price_regime_signal should succeed on trending data");
    assert_eq!(
        signal_stable.action,
        SignalAction::Enter,
        "Trending prices should produce Enter action, got {:?}",
        signal_stable.action
    );
    assert_eq!(
        signal_stable.direction, "YES",
        "Trending signal should point YES"
    );
    assert!(
        signal_stable.market_type.contains("KXBTC-TREND"),
        "market_type should include the ticker, got '{}'",
        signal_stable.market_type
    );

    // Breakout: 200 trending + 100 Lorenz → BreakoutSignal → Watch, direction=YES.
    let mut prices_breakout = trending_prices(200);
    prices_breakout.extend(lorenz_series(100, 0.01));
    let signal_breakout = engine
        .price_regime_signal("KXBTC-BREAK", &prices_breakout)
        .expect("price_regime_signal should succeed on breakout data");
    assert_eq!(
        signal_breakout.action,
        SignalAction::Watch,
        "Breakout prices should produce Watch action, got {:?}",
        signal_breakout.action
    );
    assert_eq!(
        signal_breakout.direction, "YES",
        "Breakout signal direction should be YES"
    );
}
