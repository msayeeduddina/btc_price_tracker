use tokio;
use chrono::{Utc, TimeZone};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("Testing historical data availability...\n");
    
    // Test different assets and their historical data availability
    let test_tickers = vec![
        ("BTC-USD", "Bitcoin", vec![2010, 2013, 2015, 2017, 2020]),
        ("GC=F", "Gold", vec![1990, 2000, 2010, 2015, 2020]),
        ("CAD=X", "USD/CAD", vec![1990, 2000, 2010, 2015, 2020]),
        ("^GSPC", "S&P 500", vec![1950, 1980, 2000, 2010, 2020]), // For reference
    ];
    
    use reqwest::Client;
    let client = Client::new();
    
    for (ticker, name, years) in test_tickers {
        println!("\n=== Testing {} ({}) ===", name, ticker);
        
        for year in years {
            let start = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap();
            let end = Utc.with_ymd_and_hms(year, 12, 31, 0, 0, 0).unwrap();
            
            let url = format!(
                "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval=1d",
                ticker, start.timestamp(), end.timestamp()
            );
            
            match client.get(&url)
                .header("User-Agent", "Mozilla/5.0")
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => {
                                if let Some(result) = json["chart"]["result"].get(0) {
                                    if let Some(timestamps) = result["timestamp"].as_array() {
                                        println!("  {} - ✓ {} data points", year, timestamps.len());
                                    } else {
                                        println!("  {} - ✗ No data", year);
                                    }
                                } else {
                                    println!("  {} - ✗ Invalid response", year);
                                }
                            }
                            Err(e) => println!("  {} - ✗ Parse error: {}", year, e),
                        }
                    } else {
                        println!("  {} - ✗ HTTP {}", year, response.status());
                    }
                }
                Err(e) => println!("  {} - ✗ Request failed: {}", year, e),
            }
            
            // Wait between requests
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
    
    // Test alternative BTC sources
    println!("\n\n=== Alternative Bitcoin Data Sources ===");
    
    // CoinGecko (limited history in free tier)
    println!("\nTesting CoinGecko API...");
    let coingecko_url = "https://api.coingecko.com/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days=max&interval=daily";
    match client.get(coingecko_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(prices) = json["prices"].as_array() {
                            println!("CoinGecko: {} data points available", prices.len());
                            if let Some(first) = prices.first() {
                                if let Some(ts) = first[0].as_i64() {
                                    let date = Utc.timestamp_millis_opt(ts).single()
                                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                                        .unwrap_or("Unknown".to_string());
                                    println!("Earliest date: {}", date);
                                }
                            }
                        }
                    }
                    Err(e) => println!("Parse error: {}", e),
                }
            } else {
                println!("HTTP error: {}", response.status());
            }
        }
        Err(e) => println!("Request failed: {}", e),
    }
}