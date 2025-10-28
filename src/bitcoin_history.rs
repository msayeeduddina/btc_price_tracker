use chrono::{DateTime, Utc, NaiveDate};
use anyhow::Result;
use crate::data::PriceData;

/// Try to fetch extended Bitcoin history from CoinDesk
pub async fn fetch_extended_btc_history(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<PriceData>> {
    use reqwest::Client;
    use serde_json::Value;
    
    let client = Client::new();
    
    // CoinDesk provides BTC price data from 2010-07-17
    // Format: https://api.coindesk.com/v1/bpi/historical/close.json?start=YYYY-MM-DD&end=YYYY-MM-DD
    
    let start_str = start.format("%Y-%m-%d").to_string();
    let end_str = end.format("%Y-%m-%d").to_string();
    
    let url = format!(
        "https://api.coindesk.com/v1/bpi/historical/close.json?start={}&end={}",
        start_str, end_str
    );
    
    println!("Fetching extended BTC history from CoinDesk...");
    
    let response = client.get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;
        
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("CoinDesk API error: {}", response.status()));
    }
    
    let json: Value = response.json().await?;
    
    // Parse the bpi object which contains date -> price mapping
    let bpi = json["bpi"].as_object()
        .ok_or_else(|| anyhow::anyhow!("No BPI data in response"))?;
        
    let mut data = Vec::new();
    
    for (date_str, price_value) in bpi {
        if let (Ok(date), Some(price)) = (
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d"),
            price_value.as_f64()
        ) {
            data.push(PriceData {
                date,
                price_usd: price,
                price_cad: None, // Will be calculated later
            });
        }
    }
    
    // Sort by date
    data.sort_by_key(|p| p.date);
    
    println!("Fetched {} days of historical BTC data from CoinDesk", data.len());
    
    Ok(data)
}

/// Merge Yahoo and CoinDesk data, preferring Yahoo where available
pub fn merge_btc_data(yahoo_data: Vec<PriceData>, coindesk_data: Vec<PriceData>) -> Vec<PriceData> {
    use std::collections::HashMap;
    
    // Create a map of dates to prices, starting with CoinDesk data
    let mut date_map: HashMap<NaiveDate, PriceData> = HashMap::new();
    
    // Add CoinDesk data first
    for price in coindesk_data {
        date_map.insert(price.date, price);
    }
    
    // Override with Yahoo data where available (usually more accurate)
    for price in yahoo_data {
        date_map.insert(price.date, price);
    }
    
    // Convert back to sorted vec
    let mut merged: Vec<PriceData> = date_map.into_values().collect();
    merged.sort_by_key(|p| p.date);
    
    merged
}