//! Integration tests for external feed clients.
//! These make real network calls — run with:
//!   cargo test --test feeds_integration -- --ignored

use anyhow::Result;

#[tokio::test]
#[ignore] // requires network
async fn test_open_meteo_temperature() -> Result<()> {
    // NYC coordinates
    let temps = phantom::feeds::open_meteo::fetch_temperature(40.7128, -74.0060, 48).await?;
    assert!(!temps.is_empty(), "should return temperature data");
    assert!(temps.len() >= 24, "should have at least 24 hours of data");
    // Sanity check: temps should be in a reasonable range (-40 to 130°F)
    for &t in &temps {
        assert!(t > -40.0 && t < 130.0, "temperature {} out of range", t);
    }
    println!("Open-Meteo: got {} temperature readings", temps.len());
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_open_meteo_precipitation() -> Result<()> {
    let precip = phantom::feeds::open_meteo::fetch_precipitation(40.7128, -74.0060, 48).await?;
    assert!(!precip.is_empty());
    for &p in &precip {
        assert!(p >= 0.0, "precipitation should be non-negative");
    }
    println!("Open-Meteo: got {} precipitation readings", precip.len());
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_open_meteo_ensemble() -> Result<()> {
    let ensemble = phantom::feeds::open_meteo::fetch_ensemble(40.7128, -74.0060).await?;
    assert!(!ensemble.temperature_2m.is_empty(), "should have ensemble temps");
    println!(
        "Ensemble: {} temp readings, {} precip readings",
        ensemble.temperature_2m.len(),
        ensemble.precipitation.len()
    );
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_usgs_earthquakes() -> Result<()> {
    let events = phantom::feeds::usgs::fetch_earthquakes("all_day").await?;
    // There are almost always earthquakes in a 24h window
    assert!(!events.is_empty(), "should have seismic events");
    for event in &events {
        assert!(event.magnitude >= -2.0 && event.magnitude < 12.0);
    }
    println!("USGS: got {} seismic events", events.len());
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_usgs_magnitudes() -> Result<()> {
    let mags = phantom::feeds::usgs::fetch_magnitudes("all_day").await?;
    assert!(!mags.is_empty());
    println!("USGS: got {} magnitudes, range {:.1}-{:.1}",
        mags.len(),
        mags.iter().cloned().fold(f64::INFINITY, f64::min),
        mags.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    );
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_coingecko_btc_prices() -> Result<()> {
    let prices = phantom::feeds::prices::fetch_btc_price_history(7).await?;
    assert!(!prices.is_empty(), "should have BTC price data");
    for &p in &prices {
        assert!(p > 0.0, "BTC price should be positive");
    }
    println!("CoinGecko: got {} BTC price points", prices.len());
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_opensky_states() -> Result<()> {
    let states = phantom::feeds::opensky::fetch_states(None).await?;
    // adsb.lol fallback should always work
    assert!(!states.is_empty(), "should have aircraft states");
    println!("OpenSky/ADS-B: got {} aircraft states", states.len());
    // Check first few have valid coordinates
    for state in states.iter().take(5) {
        assert!(state.lat >= -90.0 && state.lat <= 90.0);
        assert!(state.lon >= -180.0 && state.lon <= 180.0);
    }
    Ok(())
}

#[tokio::test]
#[ignore] // requires Kalshi credentials
async fn test_kalshi_auth() -> Result<()> {
    let client = phantom::feeds::KalshiClient::new()?;
    // Fetch a known series — weather markets
    let markets = client.get_markets("KXHIGHNY").await?;
    println!("Kalshi: got {} markets for KXHIGHNY", markets.len());
    for m in markets.iter().take(3) {
        println!("  {} — {:?}", m.ticker, m.title);
    }
    Ok(())
}

/// End-to-end: fetch real data → run through detector → get signal
#[tokio::test]
#[ignore]
async fn test_weather_feed_to_signal() -> Result<()> {
    // Fetch real weather data
    let temps = phantom::feeds::open_meteo::fetch_temperature(40.7128, -74.0060, 200).await?;

    // Run through the signal engine
    let engine = phantom::KalshiSignalEngine::new();
    let signal = engine.weather_signal(&temps, 62.0, true)?;

    println!("Live weather signal: {:?} — {}", signal.action, signal.reason);
    Ok(())
}

/// End-to-end: seismic data → ArgusDetector
#[tokio::test]
#[ignore]
async fn test_seismic_feed_to_detector() -> Result<()> {
    let mags = phantom::feeds::usgs::fetch_magnitudes("all_day").await?;

    if mags.len() >= 50 {
        let detector = phantom::ArgusDetector::new();
        let anomaly = detector.analyze_seismic_series(&mags)?;
        println!("Live seismic anomaly: {:?} — {}", anomaly.severity, anomaly.description);
    } else {
        println!("Not enough seismic data for analysis ({} points)", mags.len());
    }
    Ok(())
}
