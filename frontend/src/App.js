import React, { useEffect, useState } from 'react';
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';
import './App.css';

function App() {
  const [stockData, setStockData] = useState({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [selectedInterval, setSelectedInterval] = useState('1d');
  const [selectedFetchFrequency, setSelectedFetchFrequency] = useState(5000);
  const [firstLetter, setFirstLetter] = useState('A');
  const [percentChange, setPercentChange] = useState(0);
  const [priceRange, setPriceRange] = useState([0, 500]);

  useEffect(() => {
    const fetchStockData = async () => {
      try {
        setLoading(true);
        setError(null);

        const stocks = ["AAPL", "GOOGL", "MSFT", "TSLA", "META", "NVDA", "AMZN", "AMD", "RIVN", "NFLX", "GIS", "GM", "K", "BA", "DIS", "SBUX"];

        const fetchPromises = stocks.map(async (ticker) => {
          const response = await fetch(`http://127.0.0.1:8000/stocks?interval=${selectedInterval}&ticker=${ticker}&alphabetical=${firstLetter}&percentChange=${percentChange}&priceRange=${priceRange}`, {
            method: 'GET',
            headers: {
              'Content-Type': 'application/json',
              'Cache-Control': 'no-cache'
            },
            mode: 'cors',
          });

          if (!response.ok) {
            throw new Error(`Server Error: ${response.statusText}`);
          }

          const data = await response.json();
          return { ticker, data };
        });

        const results = await Promise.all(fetchPromises);

        setStockData(Object.fromEntries(results.map(({ ticker, data }) => [ticker, data])));
      } catch (error) {
        setError(error.message);
      } finally {
        setLoading(false);
      }
    };

    fetchStockData();
    const interval = setInterval(fetchStockData, selectedFetchFrequency);
    return () => clearInterval(interval);
  }, [selectedInterval, selectedFetchFrequency, firstLetter, percentChange, priceRange]);

  return (
    <div className="app-container">
      <h1 className="app-title">📈 FireB🦅rd</h1>
      <div className="interval-bar">
        <label>Select Time Interval: </label>
        <select value={selectedInterval} onChange={(e) => setSelectedInterval(e.target.value)}>
          <option value="1d">1 Day</option>
          <option value="5d">5 Days</option>
          <option value="1mo">1 Month</option>
          <option value="3mo">3 Months</option>
          <option value="6mo">6 Months</option>
          <option value="1y">1 Year</option>
          <option value="5y">5 Years</option>
        </select>

        <label>Select Update Frequency (seconds): </label>
        <select value={selectedFetchFrequency} onChange={(e) => setSelectedFetchFrequency(e.target.value)}>
          <option value={3000}>3</option>
          <option value={5000}>5</option>
          <option value={10000}>10</option>
          <option value={15000}>15</option>
          <option value={20000}>20</option>
          <option value={25000}>25</option>
          <option value={30000}>30</option>
        </select>
      </div>
      <div className='sort-by-bar'>
        <label>Alphabetical Order</label>
        <select value={firstLetter} onChange={(e) => setFirstLetter(e.target.value)}>
          {Array.from({ length: 26 }, (_, i) => String.fromCharCode(65 + i)).map(letter => (
            <option key={letter} value={letter}>{letter}</option>
          ))}
        </select>

        <label>Percent Change</label>
        <input 
          type="range" 
          min="-100" 
          max="100" 
          value={percentChange} 
          onChange={(e) => setPercentChange(e.target.value)} 
        />
        <span>{percentChange}%</span>

        <label>Price Range</label>
        <input 
          type="range" 
          min="0" 
          max="1000" 
          step="10"
          value={priceRange} 
          onChange={(e) => setPriceRange(e.target.value)} 
        />
        <span>${priceRange}</span>
      </div>

      <div className="stock-grid">
        {stockData && typeof stockData === "object" && Object.keys(stockData).length > 0 ? (
          Object.entries(stockData).map(([ticker, info]) => (
            <div key={ticker} className="stock-card">
              <h2>{ticker}</h2>
              {info && typeof info === "object" ? (
                info.error ? (
                  <p className="error">{info.error}</p>
                ) : loading ? (
                  <div className="loading">Loading...</div>
                ) : error ? (
                  <div className="error">{error}</div>
                ) : (
                  <div>
                    <p>Latest Close: {info.latest_close}</p>
                    {info.history && Array.isArray(info.history) && info.history.length > 0 ? (
                      <ResponsiveContainer width="100%" height={200}>
                        <LineChart data={info.history}>
                          <XAxis dataKey="time" hide={true} />
                          <YAxis domain={["auto", "auto"]} />
                          <Tooltip />
                          <Line type="monotone" dataKey="close" stroke="#82ca9d" dot={false} />
                        </LineChart>
                      </ResponsiveContainer>
                    ) : (
                      <p className="error">No historical data available</p>
                    )}
                  </div>
                )
              ) : (
                <p className="error">Invalid stock data</p>
              )}
            </div>
          ))
        ) : (
          <p className="error">No stock data available</p>
        )}
      </div>
    </div>
  );
}

export default App;