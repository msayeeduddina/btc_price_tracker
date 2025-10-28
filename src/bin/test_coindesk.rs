use tokio;
use chrono::{Utc, TimeZone};

#[tokio::main]
async fn main() {
    println!("Testing CoinDesk BTC API...\n");
    
    use reqwest::Client;
    let client = Client::new();
    
    // Test different date ranges
    let test_ranges = vec![
        ("2010-07-17", "2010-07-31", "First BTC data"),
        ("2011-01-01", "2011-01-31", "Early 2011"),
        ("2013-01-01", "2013-01-31", "Early 2013"),
        ("2014-01-01", "2014-01-31", "Early 2014"),
    ];
    
    for (start, end, desc) in test_ranges {
        let url = format!(
            "https://api.coindesk.com/v1/bpi/historical/close.json?start={}&end={}",
            start, end
        );
        
        println!("Testing {} ({} to {})...", desc, start, end);
        
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
                            if let Some(bpi) = json["bpi"].as_object() {
                                println!("✓ Success! Got {} data points", bpi.len());
                                // Show first few prices
                                let mut dates: Vec<_> = bpi.keys().collect();
                                dates.sort();
                                for date in dates.iter().take(3) {
                                    if let Some(price) = bpi.get(*date).and_then(|v| v.as_f64()) {
                                        println!("  {} -> ${:.2}", date, price);
                                    }
                                }
                            } else {
                                println!("✗ No BPI data in response");
                            }
                        }
                        Err(e) => println!("✗ Parse error: {}", e),
                    }
                } else {
                    println!("✗ HTTP error: {}", response.status());
                }
            }
            Err(e) => println!("✗ Request failed: {}", e),
        }
        
        println!();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    // Test a large range
    println!("Testing large range (2010-2014)...");
    let url = "https://api.coindesk.com/v1/bpi/historical/close.json?start=2010-07-17&end=2014-12-31";
    
    match client.get(url)
        .header("User-Agent", "Mozilla/5.0")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(bpi) = json["bpi"].as_object() {
                            println!("✓ Success! Got {} data points for 2010-2014", bpi.len());
                        }
                    }
                    Err(e) => println!("✗ Parse error: {}", e),
                }
            } else {
                println!("✗ HTTP error: {}", response.status());
                if let Ok(text) = response.text().await {
                    println!("Response: {}", text);
                }
            }
        }
        Err(e) => println!("✗ Request failed: {}", e),
    }
}