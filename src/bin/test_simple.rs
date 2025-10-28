use yahoo_finance_api as yahoo;

#[tokio::main]
async fn main() {
    println!("Simple Yahoo Finance test\n");
    
    // Test with known working ticker first
    let provider = yahoo::YahooConnector::new().unwrap();
    
    println!("Testing BTC-USD (known working)...");
    match provider.get_latest_quotes("BTC-USD", "1d").await {
        Ok(response) => {
            println!("BTC-USD: Success!");
            if let Ok(quotes) = response.quotes() {
                println!("Got {} quotes", quotes.len());
                if let Some(last) = quotes.last() {
                    println!("Latest: ${:.2}", last.close);
                }
            }
        }
        Err(e) => println!("BTC-USD failed: {:?}", e),
    }
    
    // Wait before next request
    println!("\nWaiting 3 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Now test CAD=X
    println!("\nTesting CAD=X...");
    match provider.get_latest_quotes("CAD=X", "1d").await {
        Ok(response) => {
            println!("CAD=X: Success!");
            if let Ok(quotes) = response.quotes() {
                println!("Got {} quotes", quotes.len());
                if let Some(last) = quotes.last() {
                    println!("Latest: ${:.4}", last.close);
                }
            }
        }
        Err(e) => println!("CAD=X failed: {:?}", e),
    }
    
    // Try CADUSD=X which is the actual ticker on Yahoo Finance
    println!("\nWaiting 3 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    println!("\nTesting CADUSD=X...");
    match provider.get_latest_quotes("CADUSD=X", "1d").await {
        Ok(response) => {
            println!("CADUSD=X: Success!");
            if let Ok(quotes) = response.quotes() {
                println!("Got {} quotes", quotes.len());
                if let Some(last) = quotes.last() {
                    println!("Latest: ${:.4}", last.close);
                }
            }
        }
        Err(e) => println!("CADUSD=X failed: {:?}", e),
    }
}