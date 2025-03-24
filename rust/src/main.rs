use rocket::{get, routes, Rocket, serde::json::Json, http::{Method, Header, Status}, Request, Response};
use serde_json::json;
use rocket::fairing::{Fairing, Info, Kind};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use serde_json::Value;
use tokio::signal;

#[macro_use] extern crate rocket;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "CORS Middleware",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "http://localhost:3000"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "GET, POST, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type, Cache-Control"));
        response.set_header(Header::new("Cache-Control", "no-store"));

        if request.method() == Method::Options {
            response.set_status(Status::Ok);
        }
    }
}

#[get("/stocks?<interval>&<ticker>&<alphabetical>&<percent_change>&<price_range>")]
fn get_stocks(
    ticker: Option<String>,
    interval: Option<String>,
    alphabetical: Option<String>,
    percent_change: Option<String>,
    price_range: Option<String>) -> (Status, Json<Value>) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let code = r#"
import yfinance as yf
import json

def analyze_stock(ticker, interval, alphabetical, percent_change, price_range):

    try:
        stock = yf.Ticker(ticker)
        hist = stock.history(period=interval, interval="1d", auto_adjust=True)

        if hist.empty:
            return json.dumps({"error": "No data available"})

        latest_close = hist['Close'].iloc[-1] if not hist.empty else None
        history = [{"time": str(index), "close": close} for index, close in zip(hist.index, hist['Close'])]

        if percent_change:
            percent_change_value = float(percent_change) / 100
            hist['pct_change'] = hist['Close'].pct_change()
            if hist['pct_change'].iloc[-1] < percent_change_value:
                return json.dumps({"error": "Stock does not meet percent change criteria"})

        if price_range:
            price_range = price_range.replace(',', '.')  # Ensure decimal compatibility
            price_parts = price_range.split('-')

            if len(price_parts) != 2:
                return json.dumps({"error": "Invalid price range format. Expected format: min-max (e.g., 500-1000)"})

            min_price, max_price = map(float, price_parts)

            if latest_close < min_price or latest_close > max_price:
                return json.dumps({"error": "Stock price is out of the specified range"})

        return json.dumps({"latest_close": latest_close, "history": history})

    except Exception as e:
        return json.dumps({"error": str(e)})
"#;

        match PyModule::from_code(py, code, "stock_analysis.py", "stock_analysis") {
            Ok(module) => {
                let analyze_stock = module.getattr("analyze_stock").unwrap();
                
                let ticker_str = ticker.unwrap_or_else(|| "AAPL".to_string());
                let interval_str = interval.unwrap_or_else(|| "1d".to_string());
                let alphabetical_str = alphabetical.unwrap_or_else(|| "".to_string());
                let percent_change_str = percent_change.unwrap_or_else(|| "".to_string());
                let price_range_str = price_range.unwrap_or_else(|| "".to_string());

                match analyze_stock.call1((ticker_str, interval_str, alphabetical_str, percent_change_str, price_range_str)) {
                    Ok(result) => {
                        let json_string: String = result.extract().unwrap();
                        match serde_json::from_str(&json_string) {
                            Ok(parsed_json) => (Status::Ok, Json(parsed_json)),
                            Err(_) => (Status::InternalServerError, Json(json!({"error": "Failed to parse JSON response"})))
                        }
                    }
                    Err(e) => {
                        let error_message = e.to_string();
                        (Status::InternalServerError, Json(json!({"error": error_message})))
                    }
                }
            }
            Err(e) => {
                let error_message = e.to_string();
                (Status::InternalServerError, Json(json!({"error": error_message})))
            }
        }
    })
}

#[options("/stocks")]
fn options_stocks() -> (Status, ()) {
    (Status::Ok, ())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let server = rocket::build()
        .attach(CORS)
        .mount("/", routes![get_stocks, options_stocks])
        .ignite()
        .await?;

    let shutdown_handle = server.shutdown();
    
    tokio::spawn(async move {
        if let Err(_) = signal::ctrl_c().await {
            eprintln!("Failed to listen for shutdown signal.");
        }
        println!("Shutting down server...");
        shutdown_handle.notify();
    });

    server.launch().await?;
    Ok(())
}