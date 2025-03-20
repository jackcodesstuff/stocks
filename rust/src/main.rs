use rocket::{get, routes, Rocket, serde::json::Json, http::{Method, Header}};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};
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

        // Allow OPTIONS preflight requests
        if request.method() == Method::Options {
            response.set_status(rocket::http::Status::Ok);
        }
    }
}

#[get("/stocks")]
fn get_stocks() -> (rocket::http::Status, Json<Value>) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let code = r#"
import yfinance as yf
import json
import pytz
from datetime import datetime

def analyze_stocks(tickers: list[str]):
    analysis_results = {}
    #now = datetime.now(pytz.timezone("US/Eastern"))
    #market_open = now.replace(hour=9, minute=30, second=0, microsecond=0)
    #market_close = now.replace(hour=16, minute=0, second=0, microsecond=0)
    
    #if now < market_open or now > market_close:
    #    return json.dumps({"error": "Market is currently closed. Please try again during market hours."})

    for ticker in tickers:
        stock = yf.Ticker(ticker)
        try:
            hist = stock.history(period="1d", interval="1m", auto_adjust=True, proxy=None)
            if hist.empty:
                analysis_results[ticker] = {"error": "No data available"}
                continue

            latest_close = hist['Close'].iloc[-1] if not hist.empty else None
            print(f"Fetched data for {ticker} at {datetime.now()} - Latest Close: {latest_close}")
            analysis_results[ticker] = {"latest_close": latest_close}
        except Exception as e:
            analysis_results[ticker] = {"error": str(e)}

    return json.dumps(analysis_results)
"#;

        let module = PyModule::from_code(py, code, "stock_analysis.py", "stock_analysis").unwrap();
        let analyze_stocks = module.getattr("analyze_stocks").unwrap();

        let stocks = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "META", "NVDA", "AMZN", "AMD", "RIVN", "NFLX"];
        let result: PyObject = analyze_stocks.call1((stocks,)).unwrap().into();

        let result: &PyAny = result.as_ref(py);
        let json_string: String = result.extract().unwrap();

        (rocket::http::Status::Ok, Json(serde_json::from_str(&json_string).unwrap()))
    })
}

#[options("/stocks")]
fn options_stocks() -> (rocket::http::Status, ()) {
    (rocket::http::Status::Ok, ())
}

#[launch]
fn rocket() -> Rocket<rocket::Build> {
    rocket::build()
        .attach(CORS)
        .mount("/", routes![get_stocks, options_stocks])
}