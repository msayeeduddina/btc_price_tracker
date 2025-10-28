use tokio;
use chrono::{Utc, TimeZone};

#[tokio::main]
async fn main() {
    println!("Testing full historical range for each asset...\n");
    
    let tickers = vec![
        ("BTC-USD", "Bitcoin"),
        ("GC=F", "Gold"),
        ("SI=F", "Silver"),
        ("CL=F", "Crude Oil"),
        ("NG=F", "Natural Gas"),
        ("HG=F", "Copper"),
        ("ZW=F", "Wheat"),
        ("ZC=F", "Corn"),
        ("ZS=F", "Soybeans"),
        ("KC=F", "Coffee"),
        ("SB=F", "Sugar"),
        ("CT=F", "Cotton"),
        ("LE=F", "Live Cattle"),
        ("ZR=F", "Rice"),
        ("LBS=F", "Lumber"),
        ("CAD=X", "USD/CAD"),
    ];
    
    use reqwest::Client;
    let client = Client::new();
    
    // Test from 1990 to now for all assets
    let start = Utc.with_ymd_and_hms(1990, 1, 1, 0, 0, 0).unwrap();
    let end = Utc::now();
    
    println!("Fetching data from 1990 to present for all assets...\n");
    
    for (ticker, name) in tickers {
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
                                    if !timestamps.is_empty() {
                                        // Get first and last dates
                                        let first_ts = timestamps.first()
                                            .and_then(|v| v.as_i64())
                                            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
                                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                                            .unwrap_or("Unknown".to_string());
                                        
                                        let last_ts = timestamps.last()
                                            .and_then(|v| v.as_i64())
                                            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
                                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                                            .unwrap_or("Unknown".to_string());
                                        
                                        println!("{:<20} ({:<8}): {} to {} ({} points)", 
                                            name, ticker, first_ts, last_ts, timestamps.len());
                                    } else {
                                        println!("{:<20} ({:<8}): No data available", name, ticker);
                                    }
                                } else {
                                    println!("{:<20} ({:<8}): No timestamp data", name, ticker);
                                }
                            }
                        }
                        Err(e) => println!("{:<20} ({:<8}): Parse error: {}", name, ticker, e),
                    }
                } else {
                    println!("{:<20} ({:<8}): HTTP {} - No data before certain date", name, ticker, response.status());
                }
            }
            Err(e) => println!("{:<20} ({:<8}): Request failed: {}", name, ticker, e),
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    println!("\n\nTrying alternative approach for Bitcoin pre-2015...");
    
    // Try getting BTC data from blockchain.info or other sources
    println!("Note: Most free APIs require authentication for historical BTC data");
    println!("Options for extended BTC history:");
    println!("1. Blockchain.info API (requires API key)");
    println!("2. CoinDesk API (limited history)"); 
    println!("3. Cryptocompare API (requires API key)");
    println!("4. Manual CSV import from sources like:");
    println!("   - https://www.investing.com/crypto/bitcoin/historical-data");
    println!("   - https://finance.yahoo.com/quote/BTC-USD/history (manual download)");
}