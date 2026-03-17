use anyhow::Result;
use phantom::signals::KalshiSignalEngine;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let live = args.iter().any(|a| a == "--live");
    let api = args.iter().any(|a| a == "--api");

    println!("Phantom Anomaly Detection Engine");
    println!("================================");
    
    if api {
        println!("Mode: API (starting axum server on port 8080)\n");
        phantom::api::start_server().await;
        Ok(())
    } else if live {
        println!("Mode: LIVE (fetching real data)\n");
        run_live().await
    } else {
        println!("Mode: DEMO (synthetic data — use --live for real feeds)\n");
        run_demo()
    }
}

async fn run_live() -> Result<()> {
    use phantom::feeds;

    let engine = KalshiSignalEngine::new();
    let mut signals = Vec::new();

    // --- Weather (NYC) ---
    println!("Fetching Open-Meteo weather data (NYC)...");
    match feeds::open_meteo::fetch_temperature(40.7128, -74.0060, 200).await {
        Ok(temps) => {
            println!("  Got {} temperature readings", temps.len());
            let signal = engine.weather_signal(&temps, 62.0, true)?;
            println!("  Weather signal: {:?}", signal.action);
            println!("  {}", signal.reason);
            signals.push(signal);

            if let Some(shift) = engine.regime_shift_signal(&temps)? {
                println!("  Regime shift: {:?} — {}", shift.action, shift.reason);
                signals.push(shift);
            }
        }
        Err(e) => eprintln!("  Weather fetch failed: {}", e),
    }

    // --- Seismic (USGS) ---
    println!("\nFetching USGS seismic data...");
    match feeds::usgs::fetch_magnitudes("all_day").await {
        Ok(mags) => {
            println!("  Got {} earthquake magnitudes", mags.len());
            if mags.len() >= 50 {
                let detector = phantom::ArgusDetector::new();
                let anomaly = detector.analyze_seismic_series(&mags)?;
                println!("  Seismic anomaly: {:?} — {}", anomaly.severity, anomaly.description);
            }
        }
        Err(e) => eprintln!("  Seismic fetch failed: {}", e),
    }

    // --- Flight data ---
    println!("\nFetching ADS-B flight data...");
    match feeds::opensky::fetch_states(None).await {
        Ok(states) => {
            println!("  Got {} aircraft states", states.len());
        }
        Err(e) => eprintln!("  Flight data fetch failed: {}", e),
    }

    // --- BTC price ---
    println!("\nFetching BTC price history (30 days)...");
    match feeds::prices::fetch_btc_price_history(30).await {
        Ok(prices) => {
            println!("  Got {} price points", prices.len());
            let signal = engine.price_regime_signal("BTC-USD", &prices)?;
            println!("  Price regime: {:?}", signal.action);
            println!("  {}", signal.reason);
            signals.push(signal);
        }
        Err(e) => eprintln!("  BTC price fetch failed: {}", e),
    }

    // --- Kalshi markets (if credentials available) ---
    match feeds::KalshiClient::new() {
        Ok(client) => {
            println!("\nFetching Kalshi markets...");
            match client.get_markets("KXHIGHNY").await {
                Ok(markets) => {
                    println!("  Got {} KXHIGHNY markets", markets.len());
                    for m in markets.iter().take(3) {
                        println!("    {} — {:?}", m.ticker, m.title);
                    }
                }
                Err(e) => eprintln!("  Kalshi market fetch failed: {}", e),
            }
        }
        Err(_) => {
            println!("\nKalshi: no credentials found, skipping.");
        }
    }

    if !signals.is_empty() {
        engine.print_report(&signals);
    }

    Ok(())
}

fn run_demo() -> Result<()> {
    let engine = KalshiSignalEngine::new();

    // Simulate 200 hourly temperature readings (NYC, March)
    let temps: Vec<f64> = (0..200)
        .map(|i| {
            let base = 55.0 + (i as f64 * 0.05).sin() * 8.0;
            let noise = ((i * 7 + 3) % 17) as f64 * 0.3 - 2.5;
            base + noise
        })
        .collect();

    // Weather signal: will NYC be above 62°F?
    let signal = engine.weather_signal(&temps, 62.0, true)?;
    println!("Weather signal: {:?}", signal.action);
    println!("  {}", signal.reason);

    // Regime shift check
    if let Some(shift) = engine.regime_shift_signal(&temps)? {
        println!("\nRegime shift: {:?}", shift.action);
        println!("  {}", shift.reason);
    } else {
        println!("\nNo regime shift detected.");
    }

    // Price series demo
    let prices: Vec<f64> = (0..300)
        .map(|i| {
            let trend = i as f64 * 0.02;
            let chaos = if i > 200 { ((i * 13) % 31) as f64 * 0.8 } else { ((i * 3) % 7) as f64 * 0.2 };
            60.0 + trend + chaos
        })
        .collect();

    let price_signal = engine.price_regime_signal("KALSHI-BTC-60K-MAR", &prices)?;
    println!("\nPrice regime: {:?}", price_signal.action);
    println!("  {}", price_signal.reason);

    engine.print_report(&[signal, price_signal]);

    Ok(())
}
