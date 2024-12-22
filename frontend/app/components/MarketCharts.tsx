'use client';

import { useEffect, useRef, useState } from 'react';
import { Market } from '../types/market';
import { MarketService } from '../services/marketService';

interface MarketChartsProps {
  market: Market;
}

interface ChartData {
  timestamp: number;
  price: number;
  volume: number;
}

export function MarketCharts({ market }: MarketChartsProps) {
  const [chartData, setChartData] = useState<ChartData[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedTimeframe, setSelectedTimeframe] = useState<'1D' | '1W' | '1M' | 'ALL'>('1D');

  const marketService = MarketService.getInstance();
  const chartContainerRef = useRef<HTMLDivElement>(null);

  const fetchChartData = async () => {
    try {
      setIsLoading(true);
      setError(null);

      // TODO: Implement chart data fetching from the market service
      const data = await marketService.getMarketChartData(market.id, selectedTimeframe);
      setChartData(data);
    } catch (err) {
      console.error('Error fetching chart data:', err);
      setError('Failed to load chart data');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchChartData();
  }, [market.id, selectedTimeframe]);

  const renderChart = () => {
    if (isLoading) {
      return (
        <div className="flex justify-center items-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
        </div>
      );
    }

    if (error) {
      return (
        <div className="flex justify-center items-center h-64">
          <div className="text-red-600">{error}</div>
        </div>
      );
    }

    if (!chartData.length) {
      return (
        <div className="flex justify-center items-center h-64">
          <div className="text-gray-500">No chart data available</div>
        </div>
      );
    }

    return (
      <div className="h-64" ref={chartContainerRef}>
        {/* TODO: Implement chart visualization using a charting library */}
        <div className="text-center text-gray-500">
          Chart visualization will be implemented using a charting library
        </div>
      </div>
    );
  };

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-xl font-semibold">Market Activity</h2>
        <div className="flex space-x-2">
          {(['1D', '1W', '1M', 'ALL'] as const).map((timeframe) => (
            <button
              key={timeframe}
              onClick={() => setSelectedTimeframe(timeframe)}
              className={`px-3 py-1 rounded-md text-sm font-medium ${
                selectedTimeframe === timeframe
                  ? 'bg-blue-500 text-white'
                  : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
              }`}
            >
              {timeframe}
            </button>
          ))}
        </div>
      </div>

      <div className="space-y-6">
        {/* Price Chart */}
        <div>
          <h3 className="text-lg font-medium mb-2">Price History</h3>
          {renderChart()}
        </div>

        {/* Volume Chart */}
        <div>
          <h3 className="text-lg font-medium mb-2">Trading Volume</h3>
          {renderChart()}
        </div>
      </div>

      <div className="mt-4 text-sm text-gray-500">
        <p>Note: Chart data is updated in real-time as trades occur.</p>
      </div>
    </div>
  );
} 