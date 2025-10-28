use yahoo_finance_api as yahoo;
use tokio;
use chrono::{Utc, TimeZone};
use yahoo_finance_api::time::OffsetDateTime;

#[tokio::main]
async fn main() {
    println!("Testing Yahoo Finance API connection...\n");
    
    let provider = match yahoo::YahooConnector::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create Yahoo connector: {}", e);
            return;
        }
    };
    
    // Test with a simple, well-known ticker
    let ticker = "AAPL";
    println!("Testing with ticker: {}", ticker);
    
    // Try to get just the last month of data
    let end = Utc::now();
    let start = end - chrono::Duration::days(30);
    
    let start_time = match OffsetDateTime::from_unix_timestamp(start.timestamp()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to convert start time: {}", e);
            return;
        }
    };
    
    let end_time = match OffsetDateTime::from_unix_timestamp(end.timestamp()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to convert end time: {}", e);
            return;
        }
    };
    
    println!("Fetching data from {} to {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"));
    
    match provider.get_quote_history(ticker, start_time, end_time).await {
        Ok(response) => {
            println!("Successfully got response from Yahoo Finance!");
            match response.quotes() {
                Ok(quotes) => {
                    println!("Got {} quotes", quotes.len());
                    if let Some(first) = quotes.first() {
                        println!("First quote: timestamp={}, close=${:.2}", first.timestamp, first.close);
                    }
                    if let Some(last) = quotes.last() {
                        println!("Last quote: timestamp={}, close=${:.2}", last.timestamp, last.close);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse quotes: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch from Yahoo Finance: {}", e);
            eprintln!("Error details: {:?}", e);
        }
    }
    
    // Now test Bitcoin
    println!("\nTesting Bitcoin (BTC-USD)...");
    match provider.get_quote_history("BTC-USD", start_time, end_time).await {
        Ok(response) => {
            match response.quotes() {
                Ok(quotes) => {
                    println!("Got {} BTC quotes", quotes.len());
                    if let Some(last) = quotes.last() {
                        println!("Latest BTC price: ${:.2}", last.close);
                    }
                }
                Err(e) => eprintln!("Failed to parse BTC quotes: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch BTC data: {}", e);
        }
    }
}