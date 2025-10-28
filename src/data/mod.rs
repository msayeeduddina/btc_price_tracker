use chrono::{NaiveDate, Utc, TimeZone};
use std::collections::HashMap;
use crate::models::Asset;
use crate::yahoo_data::{YahooDataFetcher};
use crate::alternative_data::{AlternativeDataFetcher};

#[derive(Debug, Clone)]
pub struct PriceData {
    pub date: NaiveDate,
    pub price_usd: f64,
    pub price_cad: Option<f64>, // Will be calculated from USD * USD/CAD rate
}

// Get data from Yahoo Finance by default, fall back to alternative sources
pub fn get_historical_data() -> HashMap<Asset, Vec<PriceData>> {
    println!("Fetching data from Yahoo Finance...");
    match fetch_yahoo_data() {
        Ok(data) => {
            if data.is_empty() {
                println!("Yahoo Finance returned no data, trying alternative sources...");
                fetch_alternative_data()
            } else {
                println!("Successfully fetched Yahoo Finance data");
                data
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch Yahoo Finance data: {}", e);
            eprintln!("Trying alternative data sources...");
            fetch_alternative_data()
        }
    }
}

fn fetch_yahoo_data() -> anyhow::Result<HashMap<Asset, Vec<PriceData>>> {
    use tokio::runtime::Runtime;
    
    let rt = Runtime::new()?;
    rt.block_on(async {
        let fetcher = YahooDataFetcher::new();
        let start = Utc.with_ymd_and_hms(1999, 1, 1, 0, 0, 0).unwrap(); // Extended to 1999 for max commodity history
        let end = Utc::now();
        
        fetcher.fetch_historical_data(start, end).await
    })
}

fn fetch_alternative_data() -> HashMap<Asset, Vec<PriceData>> {
    use tokio::runtime::Runtime;
    
    match Runtime::new() {
        Ok(rt) => {
            rt.block_on(async {
                let fetcher = AlternativeDataFetcher::new();
                let start = Utc.with_ymd_and_hms(1999, 1, 1, 0, 0, 0).unwrap(); // Extended to 1999 for max commodity history
                let end = Utc::now();
                
                match fetcher.fetch_historical_data(start, end).await {
                    Ok(data) => {
                        println!("Successfully fetched alternative data");
                        data
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch alternative data: {}", e);
                        HashMap::new()
                    }
                }
            })
        }
        Err(e) => {
            eprintln!("Failed to create runtime for alternative data: {}", e);
            HashMap::new()
        }
    }
}

