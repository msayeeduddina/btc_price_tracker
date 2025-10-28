use reqwest;

#[tokio::main]
async fn main() {
    println!("Testing direct HTTP request to Yahoo Finance...\n");
    
    let client = reqwest::Client::new();
    
    // Test CAD=X URL directly
    let url = "https://query1.finance.yahoo.com/v8/finance/chart/CAD=X";
    
    println!("Fetching: {}", url);
    
    match client.get(url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await {
        Ok(response) => {
            println!("Status: {}", response.status());
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        println!("Response length: {} bytes", text.len());
                        // Print first 500 chars
                        if text.len() > 500 {
                            println!("First 500 chars: {}", &text[..500]);
                        } else {
                            println!("Response: {}", text);
                        }
                    }
                    Err(e) => println!("Failed to read response: {}", e),
                }
            } else {
                println!("HTTP error");
                if let Ok(text) = response.text().await {
                    println!("Error response: {}", text);
                }
            }
        }
        Err(e) => println!("Request failed: {}", e),
    }
}