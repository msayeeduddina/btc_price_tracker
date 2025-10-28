use yahoo_finance_api as yahoo;
use tokio;
use chrono::{Utc, TimeZone};
use yahoo_finance_api::time::OffsetDateTime;

#[tokio::main]
async fn main() {
    println!("Testing CAD=X fetch...\n");
    
    // Wait a bit before starting to avoid any rate limiting from previous runs
    println!("Waiting 30 seconds before starting tests...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    
    let provider = match yahoo::YahooConnector::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create Yahoo connector: {}", e);
            return;
        }
    };
    
    // Try just CAD=X first
    let tickers = vec!["CAD=X"];
    
    for ticker in tickers {
        println!("\nTesting ticker: {}", ticker);
        
        // Try to get just the last week of data
        let end = Utc::now();
        let start = end - chrono::Duration::days(7);
        
        let start_time = match OffsetDateTime::from_unix_timestamp(start.timestamp()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to convert start time: {}", e);
                continue;
            }
        };
        
        let end_time = match OffsetDateTime::from_unix_timestamp(end.timestamp()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to convert end time: {}", e);
                continue;
            }
        };
        
        match provider.get_quote_history(ticker, start_time, end_time).await {
            Ok(response) => {
                println!("Successfully got response for {}!", ticker);
                match response.quotes() {
                    Ok(quotes) => {
                        println!("Got {} quotes", quotes.len());
                        if let Some(last) = quotes.last() {
                            println!("Latest quote: timestamp={}, close={}", last.timestamp, last.close);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse quotes: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch {}: {:?}", ticker, e);
            }
        }
        
        // Add delay between attempts
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}