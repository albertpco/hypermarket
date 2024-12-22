import React from 'react';
import { formatDistance } from 'date-fns';
import { Market } from '../types';

interface MarketCardProps {
  market: Market;
  onSelect: (marketId: string) => void;
}

export const MarketCard: React.FC<MarketCardProps> = ({ market, onSelect }) => {
  const timeUntilExpiry = formatDistance(
    new Date(market.expiry_timestamp * 1000),
    new Date(),
    { addSuffix: true }
  );

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active':
        return 'bg-green-100 text-green-800';
      case 'expired':
        return 'bg-yellow-100 text-yellow-800';
      case 'resolved':
        return 'bg-blue-100 text-blue-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <div 
      className="bg-white rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow cursor-pointer"
      onClick={() => onSelect(market.id)}
    >
      <div className="flex justify-between items-start mb-4">
        <h3 className="text-lg font-semibold text-gray-900 flex-grow">
          {market.question}
        </h3>
        <span className={`px-2 py-1 rounded-full text-sm ${getStatusColor(market.status)}`}>
          {market.status}
        </span>
      </div>

      <div className="grid grid-cols-2 gap-4 mb-4">
        <div>
          <p className="text-sm text-gray-500">Current Price</p>
          <p className="text-lg font-medium">
            ${Number(market.current_price).toFixed(2)}
          </p>
        </div>
        <div>
          <p className="text-sm text-gray-500">Volume</p>
          <p className="text-lg font-medium">
            ${Number(market.total_volume).toLocaleString()}
          </p>
        </div>
      </div>

      <div className="flex justify-between items-center text-sm text-gray-500">
        <span>Expires {timeUntilExpiry}</span>
        <span>{market.collateral_token}</span>
      </div>

      <div className="mt-4 flex space-x-2">
        <button 
          className="flex-1 bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 transition-colors"
          onClick={(e) => {
            e.stopPropagation();
            // Add trading modal logic
          }}
        >
          Trade
        </button>
        <button 
          className="flex-1 bg-gray-100 text-gray-700 px-4 py-2 rounded hover:bg-gray-200 transition-colors"
          onClick={(e) => {
            e.stopPropagation();
            // Add details modal logic
          }}
        >
          Details
        </button>
      </div>
    </div>
  );
}; 