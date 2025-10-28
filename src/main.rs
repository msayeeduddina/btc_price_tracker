mod models;
mod data;
mod yahoo_data;
mod alternative_data;

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Legend};
use std::collections::HashMap;
use chrono::NaiveDate;
use crate::models::{Asset, RepresentationMode};
use crate::data::{PriceData, get_historical_data};

struct PurchasingPowerApp {
    data: HashMap<Asset, Vec<PriceData>>,
    selected_commodity: Asset,
    representation_mode: RepresentationMode,
    date_slider_value: f32, // 0.0 to 1.0 representing start date position
}

impl PurchasingPowerApp {
    fn new() -> Self {
        Self {
            data: get_historical_data(),
            selected_commodity: Asset::ConsumerBasket,
            representation_mode: RepresentationMode::PricePerUnit,
            date_slider_value: 0.75, // Start at 75% through the data (showing recent history)
        }
    }

    fn calculate_btc_values(&self) -> Vec<[f64; 2]> {
        let btc_data = match self.data.get(&Asset::Bitcoin) {
            Some(data) => data,
            None => return Vec::new(),
        };
        let commodity_data = match self.data.get(&self.selected_commodity) {
            Some(data) => data,
            None => return Vec::new(),
        };
        
        let mut points = Vec::new();
        let mut exact_matches = 0;
        let mut nearest_matches = 0;
        
        // Find matching dates
        for btc_point in btc_data {
            // First try exact match
            if let Some(commodity_point) = commodity_data.iter().find(|p| p.date == btc_point.date) {
                exact_matches += 1;
                let value = match self.representation_mode {
                    RepresentationMode::UnitsPerCurrency => btc_point.price_usd / commodity_point.price_usd,
                    RepresentationMode::PricePerUnit => commodity_point.price_usd / btc_point.price_usd,
                };
                let x = btc_point.date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as f64 / 86400.0;
                points.push([x, value]);
            } else {
                // If no exact match, find the nearest date
                let mut nearest_date = None;
                let mut min_days_diff = i64::MAX;
                
                for commodity_point in commodity_data {
                    let days_diff = (btc_point.date - commodity_point.date).num_days().abs();
                    if days_diff < min_days_diff {
                        min_days_diff = days_diff;
                        nearest_date = Some(commodity_point);
                    }
                }
                
                if let Some(commodity_point) = nearest_date {
                    // Only use nearest date if within 30 days
                    if min_days_diff <= 30 {
                        nearest_matches += 1;
                        let value = match self.representation_mode {
                            RepresentationMode::UnitsPerCurrency => btc_point.price_usd / commodity_point.price_usd,
                            RepresentationMode::PricePerUnit => commodity_point.price_usd / btc_point.price_usd,
                        };
                        let x = btc_point.date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as f64 / 86400.0;
                        points.push([x, value]);
                    }
                }
            }
        }
        
        // Debug output - only print when commodity changes or on first run
        static mut LAST_COMMODITY: Option<Asset> = None;
        unsafe {
            if LAST_COMMODITY != Some(self.selected_commodity) {
                println!("Commodity: {:?} - Exact matches: {}, Nearest matches: {}, Total points: {}", 
                         self.selected_commodity, exact_matches, nearest_matches, points.len());
                LAST_COMMODITY = Some(self.selected_commodity);
            }
        }
        
        points
    }
    
    fn calculate_usd_values(&self) -> Vec<[f64; 2]> {
        let commodity_data = match self.data.get(&self.selected_commodity) {
            Some(data) => data,
            None => return Vec::new(),
        };
        
        commodity_data.iter().map(|p| {
            let x = p.date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as f64 / 86400.0;
            let value = match self.representation_mode {
                RepresentationMode::UnitsPerCurrency => 1.0 / p.price_usd,
                RepresentationMode::PricePerUnit => p.price_usd,
            };
            [x, value]
        }).collect()
    }
    
    fn calculate_cad_values(&self) -> Vec<[f64; 2]> {
        let commodity_data = match self.data.get(&self.selected_commodity) {
            Some(data) => data,
            None => return Vec::new(),
        };
        
        commodity_data.iter().filter_map(|p| {
            p.price_cad.map(|cad_price| {
                let x = p.date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as f64 / 86400.0;
                let value = match self.representation_mode {
                    RepresentationMode::UnitsPerCurrency => 1.0 / cad_price,
                    RepresentationMode::PricePerUnit => cad_price,
                };
                [x, value]
            })
        }).collect()
    }
}

impl eframe::App for PurchasingPowerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Bitcoin Purchasing Power vs Real Assets");
            
            // Check if we have data
            if self.data.is_empty() {
                ui.separator();
                ui.colored_label(egui::Color32::RED, "Failed to fetch data from Yahoo Finance!");
                ui.label("Please check your internet connection and restart the application.");
                return;
            }
            
            // Check if we have Bitcoin data specifically
            if !self.data.contains_key(&Asset::Bitcoin) {
                ui.separator();
                ui.colored_label(egui::Color32::YELLOW, "Warning: Bitcoin data not available");
                ui.label("The chart will not display properly without Bitcoin price data.");
            }
            
            ui.horizontal(|ui| {
                ui.label("Compare Bitcoin against:");
                egui::ComboBox::from_label("Asset")
                    .selected_text(self.selected_commodity.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_commodity, Asset::ConsumerBasket, "🛒 Consumer Basket (Blended)");
                        ui.separator();
                        ui.selectable_value(&mut self.selected_commodity, Asset::Gold, "Gold (per oz)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Silver, "Silver (per oz)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Oil, "Crude Oil (per barrel)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::NaturalGas, "Natural Gas (per MMBtu)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Copper, "Copper (per lb)");
                        ui.separator();
                        ui.selectable_value(&mut self.selected_commodity, Asset::Wheat, "Wheat (per bushel)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Corn, "Corn (per bushel)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Soybeans, "Soybeans (per bushel)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Rice, "Rice (per cwt)");
                        ui.separator();
                        ui.selectable_value(&mut self.selected_commodity, Asset::Beef, "Beef (per pound)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Coffee, "Coffee (per lb)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Sugar, "Sugar (per lb)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Cotton, "Cotton (per lb)");
                        ui.selectable_value(&mut self.selected_commodity, Asset::Lumber, "Lumber (per 1000 bd ft)");
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Display mode:");
                ui.radio_value(&mut self.representation_mode, RepresentationMode::UnitsPerCurrency, "Units per Currency");
                ui.radio_value(&mut self.representation_mode, RepresentationMode::PricePerUnit, "Price per Unit");
            });

            ui.separator();

            let plot_height = ui.available_height() * 0.85;
            // Calculate data bounds for x-axis range
            let btc_points = self.calculate_btc_values();
            let usd_points = self.calculate_usd_values();
            let cad_points = self.calculate_cad_values();
            
            // Get the full data range
            let full_range = if !btc_points.is_empty() {
                let min_x = btc_points.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
                let max_x = btc_points.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
                [min_x, max_x]
            } else {
                [0.0, 1.0]
            };
            
            // Slider controls the start date (0.0 = earliest data, 1.0 = most recent data)
            let total_range = full_range[1] - full_range[0];
            let start_offset = total_range * self.date_slider_value as f64;
            
            // Calculate the visible date range - from slider position to end
            let x_bounds = [
                full_range[0] + start_offset,
                full_range[1]
            ];
            
            
            // Date range slider
            ui.horizontal(|ui| {
                ui.label("Start date:");
                
                // Calculate the date that corresponds to the slider position
                let slider_date = if !btc_points.is_empty() {
                    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
                        + chrono::Duration::days(x_bounds[0] as i64)
                } else {
                    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()
                };
                
                let response = ui.add(egui::Slider::new(&mut self.date_slider_value, 0.0..=0.95)
                    .show_value(false)
                    .clamp_to_range(true));
                
                // Force continuous updates
                if response.changed() || response.dragged() {
                    ctx.request_repaint();
                }
                
                // Show the current date at slider position
                ui.label(format!("{}", slider_date.format("%Y-%m-%d")));
                
                // Show the full range being displayed
                if !btc_points.is_empty() {
                    let end_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
                        + chrono::Duration::days(x_bounds[1] as i64);
                    ui.label(format!("to {}", end_date.format("%Y-%m-%d")));
                }
            });
            
            ui.separator();
            
            // Combined chart with two Y-axes
            Plot::new("combined_chart")
                .height(plot_height)
                .x_axis_formatter(|grid_mark, _, _| {
                    let timestamp = grid_mark.value * 86400.0;
                    let date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
                        + chrono::Duration::seconds(timestamp as i64);
                    format!("{}", date.format("%Y-%m"))
                })
                .y_axis_formatter(|grid_mark, _, _| {
                    format!("{:.1}%", grid_mark.value)
                })
                .legend(Legend::default())
                .show_axes([true, true])
                .auto_bounds([false, false].into())
                .allow_zoom(false)
                .allow_drag(false)
                .allow_scroll(false)
                .show(ui, |plot_ui| {
                    // Find the first point in the visible range for base values
                    let btc_base_idx = btc_points.iter().position(|p| p[0] >= x_bounds[0]).unwrap_or(0);
                    let usd_base_idx = usd_points.iter().position(|p| p[0] >= x_bounds[0]).unwrap_or(0);
                    let cad_base_idx = cad_points.iter().position(|p| p[0] >= x_bounds[0]).unwrap_or(0);
                    
                    // Always show all lines as percentage change from start
                    if !btc_points.is_empty() && !usd_points.is_empty() {
                        let btc_base = btc_points.get(btc_base_idx).map(|p| p[1]).unwrap_or(1.0);
                        let usd_base = usd_points.get(usd_base_idx).map(|p| p[1]).unwrap_or(1.0);
                        
                        // Convert to percentage change
                        let btc_pct: Vec<[f64; 2]> = btc_points.iter()
                            .skip(btc_base_idx)
                            .filter(|p| p[0] >= x_bounds[0] && p[0] <= x_bounds[1])
                            .map(|p| [p[0], ((p[1] / btc_base) - 1.0) * 100.0])
                            .collect();
                            
                        let usd_pct: Vec<[f64; 2]> = usd_points.iter()
                            .skip(usd_base_idx)
                            .filter(|p| p[0] >= x_bounds[0] && p[0] <= x_bounds[1])
                            .map(|p| [p[0], ((p[1] / usd_base) - 1.0) * 100.0])
                            .collect();
                        
                        // Calculate Y-axis bounds from visible data
                        let mut min_y: f64 = 0.0;
                        let mut max_y: f64 = 0.0;
                        
                        // Check all data series for min/max
                        for points in [&btc_pct, &usd_pct] {
                            for p in points {
                                min_y = min_y.min(p[1]);
                                max_y = max_y.max(p[1]);
                            }
                        }
                        
                        // Set the plot bounds first
                        // Add padding to Y-axis bounds
                        let y_padding = (max_y - min_y) * 0.1;
                        if y_padding > 0.0 {
                            min_y -= y_padding;
                            max_y += y_padding;
                        } else {
                            // If all values are the same, add some default padding
                            min_y -= 5.0;
                            max_y += 5.0;
                        }
                        
                        plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                            [x_bounds[0], min_y],
                            [x_bounds[1], max_y]
                        ));
                        
                        // BTC line
                        let btc_name = match self.representation_mode {
                            RepresentationMode::UnitsPerCurrency => format!("{} per BTC (% change)", self.selected_commodity.unit()),
                            RepresentationMode::PricePerUnit => format!("BTC per {} (% change)", self.selected_commodity.unit()),
                        };
                        // Draw USD line first (thicker)
                        let usd_name = match self.representation_mode {
                            RepresentationMode::UnitsPerCurrency => format!("{} per Dollar (% change)", self.selected_commodity.unit()),
                            RepresentationMode::PricePerUnit => format!("USD per {} (% change)", self.selected_commodity.unit()),
                        };
                        let line = Line::new(PlotPoints::from(usd_pct))
                            .name(usd_name)
                            .color(egui::Color32::from_rgb(0, 128, 255))
                            .width(2.5);
                        plot_ui.line(line);
                        
                        // Then BTC line
                        let line = Line::new(PlotPoints::from(btc_pct))
                            .name(btc_name)
                            .color(egui::Color32::from_rgb(255, 165, 0))
                            .width(2.0);
                        plot_ui.line(line);
                        
                        // CAD line
                        if !cad_points.is_empty() {
                            let cad_base = cad_points.get(cad_base_idx).map(|p| p[1]).unwrap_or(1.0);
                            
                            let cad_pct: Vec<[f64; 2]> = cad_points.iter()
                                .skip(cad_base_idx)
                                .filter(|p| p[0] >= x_bounds[0] && p[0] <= x_bounds[1])
                                .map(|p| [p[0], ((p[1] / cad_base) - 1.0) * 100.0])
                                .collect();
                            
                            // Update min/max with CAD data
                            for p in &cad_pct {
                                min_y = min_y.min(p[1]);
                                max_y = max_y.max(p[1]);
                            }
                            
                            let cad_name = match self.representation_mode {
                                RepresentationMode::UnitsPerCurrency => format!("{} per CAD (% change)", self.selected_commodity.unit()),
                                RepresentationMode::PricePerUnit => format!("CAD per {} (% change)", self.selected_commodity.unit()),
                            };
                            let line = Line::new(PlotPoints::from(cad_pct))
                                .name(cad_name)
                                .color(egui::Color32::from_rgb(220, 50, 50)) // Red
                                .width(1.5);
                            plot_ui.line(line);
                        }
                    }
                });
            
            ui.separator();
            ui.label("Understanding the chart:");
            match self.representation_mode {
                RepresentationMode::UnitsPerCurrency => {
                    ui.label("• All lines show % change in purchasing power from start date");
                    ui.label("• Orange: How many more/fewer commodity units 1 BTC can buy");
                    ui.label("• Blue: How many more/fewer commodity units $1 USD can buy");
                    ui.label("• Red: How many more/fewer commodity units $1 CAD can buy");
                    ui.label("• Same scale makes comparison fair and accurate");
                },
                RepresentationMode::PricePerUnit => {
                    ui.label("• All lines show % change in price from start date");
                    ui.label("• Orange: % change in BTC needed per unit of commodity");
                    ui.label("• Blue: % change in USD needed per unit of commodity");
                    ui.label("• Red: % change in CAD needed per unit of commodity");
                    ui.label("• Negative % means commodity got cheaper");
                },
            }
            ui.separator();
            ui.label("Use the slider to adjust the start date of the chart");
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Bitcoin Purchasing Power Tracker",
        native_options,
        Box::new(|_cc| Box::new(PurchasingPowerApp::new())),
    )
}