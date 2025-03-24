use rocket::{get, routes, Rocket, serde::json::Json, http::{Method, Header, Status}, Request, Response};
use serde_json::json;
use rocket::fairing::{Fairing, Info, Kind};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use serde_json::Value;

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

#[get("/stocks?<interval>")]
fn get_stocks(interval: Option<String>) -> (Status, Json<Value>) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let code = r#"
import yfinance as yf
import json

def analyze_stocks(tickers, interval):
    analysis_results = {}
    
    for ticker in tickers:
        stock = yf.Ticker(ticker)
        try:
            hist = stock.history(period=interval, interval="1d", auto_adjust=True)
            if hist.empty:
                analysis_results[ticker] = {"error": "No data available"}
                continue

            latest_close = hist['Close'].iloc[-1] if not hist.empty else None
            history = [{"time": str(index), "close": close} for index, close in zip(hist.index, hist['Close'])]

            analysis_results[ticker] = {"latest_close": latest_close, "history": history}
        except Exception as e:
            analysis_results[ticker] = {"error": str(e)}

    return json.dumps(analysis_results)
"#;

        match PyModule::from_code(py, code, "stock_analysis.py", "stock_analysis") {
            Ok(module) => {
                let analyze_stocks = module.getattr("analyze_stocks").unwrap();
                let stocks = vec![
                    "AAPL", "GOOGL", "MSFT", "TSLA", "META", "NVDA", "AMZN", "AMD", 
                    "RIVN", "NFLX", "GIS", "GM", "K", "BA", "DIS", "SBUX"
                ];
                let interval_str = interval.unwrap_or_else(|| "1d".to_string());

                match analyze_stocks.call1((stocks, interval_str)) {
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

#[launch]
fn rocket() -> Rocket<rocket::Build> {
    rocket::build()
        .attach(CORS)
        .mount("/", routes![get_stocks, options_stocks])
}