use tokio;
use std::path::PathBuf;

// Import from parent crate
#[path = "../data/mod.rs"]
mod data;
#[path = "../models/mod.rs"] 
mod models;
#[path = "../yahoo_data.rs"]
mod yahoo_data;
#[path = "../alternative_data.rs"]
mod alternative_data;

use data::get_historical_data;

#[tokio::main]
async fn main() {
    println!("Testing data integration...\n");
    
    println!("Waiting 60 seconds to avoid rate limiting...");
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    
    let data = get_historical_data();
    
    println!("\n=== Data Summary ===");
    for (asset, prices) in &data {
        println!("{:?}: {} data points", asset, prices.len());
        
        // Check CAD prices
        let with_cad = prices.iter().filter(|p| p.price_cad.is_some()).count();
        println!("  - With CAD prices: {}", with_cad);
        
        // Show first and last prices
        if let Some(first) = prices.first() {
            println!("  - First: {} USD = {:.2}, CAD = {}", 
                first.date, 
                first.price_usd, 
                first.price_cad.map(|c| format!("{:.2}", c)).unwrap_or("N/A".to_string())
            );
        }
        if let Some(last) = prices.last() {
            println!("  - Last: {} USD = {:.2}, CAD = {}", 
                last.date, 
                last.price_usd,
                last.price_cad.map(|c| format!("{:.2}", c)).unwrap_or("N/A".to_string())
            );
        }
    }
}