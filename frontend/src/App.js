import React, { useEffect, useState } from 'react';
import './App.css';

function App() {
  const [stockData, setStockData] = useState({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchStockData = async () => {
      try {
        setLoading(true);
        setError(null);

        const response = await fetch('http://127.0.0.1:8000/stocks', {
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
        setStockData(data);
      } catch (error) {
        setError(error.message);
      } finally {
        setLoading(false);
      }
    };

    fetchStockData();
    const interval = setInterval(fetchStockData, 2000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="app-container">
      <h1 className="app-title">📈 Real-Time Stock Tracker</h1>
      <div className="stock-grid">
        {Object.entries(stockData).map(([ticker, info]) => (
          <div key={ticker} className="stock-card">
            <h2>{ticker}</h2>
            {info.error ? (
              <p className="error">{info.error}</p>
            ) : (
              loading ? (
                <div className="loading">Loading...</div>
              ) : error ? (
                <div className="error">{error}</div>
              ) : (
                <p>Latest Close: {info.latest_close}</p>
              )
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

export default App;