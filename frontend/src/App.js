import React, { useEffect, useState } from 'react';

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
    const interval = setInterval(fetchStockData, 5000); // Refresh every 5 seconds

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold mb-6 text-center">Real-Time Stock Tracker</h1>
      {loading ? (
        <p>Loading...</p>
      ) : error ? (
        <p className="text-red-500">{error}</p>
      ) : (
        <div className="grid grid-cols-2 gap-4">
          {Object.entries(stockData).map(([ticker, info]) => (
            <div key={ticker} className="p-4 border border-gray-300 rounded-xl shadow-lg">
              <h2 className="font-bold text-lg">{ticker}</h2>
              {info.error ? (
                <p className="text-red-500">{info.error}</p>
              ) : (
                <p>Latest Close: {info.latest_close}</p>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default App;