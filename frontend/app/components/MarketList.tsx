'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { Market, MarketStatus } from '../types/market';
import { MarketService } from '../services/marketService';

export function MarketList() {
  const [markets, setMarkets] = useState<Market[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchMarkets = async () => {
      try {
        setIsLoading(true);
        setError(null);
        
        // TODO: Implement actual market fetching
        const sampleMarkets: Market[] = [
          {
            id: '1',
            question: 'Will Bitcoin reach $100,000 by the end of 2024?',
            description: 'Market for predicting Bitcoin price milestone',
            status: MarketStatus.Active,
            expiryTimestamp: Date.now() + 86400000 * 30, // 30 days from now
            oracleId: '0x1234...5678',
            collateralToken: 'ETH',
            volume24h: 150.5,
            totalLiquidity: 1000.0,
          },
          {
            id: '2',
            question: 'Will Ethereum complete the Cancun upgrade in Q1 2024?',
            description: 'Market for predicting Ethereum upgrade timeline',
            status: MarketStatus.Active,
            expiryTimestamp: Date.now() + 86400000 * 60, // 60 days from now
            oracleId: '0x9876...4321',
            collateralToken: 'ETH',
            volume24h: 75.25,
            totalLiquidity: 500.0,
          },
        ];

        setMarkets(sampleMarkets);
      } catch (err) {
        console.error('Error fetching markets:', err);
        setError('Failed to load markets');
      } finally {
        setIsLoading(false);
      }
    };

    fetchMarkets();
  }, []);

  const getStatusColor = (status: MarketStatus) => {
    switch (status) {
      case MarketStatus.Active:
        return 'bg-green-100 text-green-800';
      case MarketStatus.Expired:
        return 'bg-yellow-100 text-yellow-800';
      case MarketStatus.Resolved:
        return 'bg-blue-100 text-blue-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center min-h-[200px]">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded">
        {error}
      </div>
    );
  }

  return (
    <div className="grid gap-6">
      {markets.map((market) => (
        <Link key={market.id} href={`/markets/${market.id}`}>
          <div className="bg-white rounded-lg shadow hover:shadow-md transition-shadow p-6">
            <div className="flex justify-between items-start mb-4">
              <h2 className="text-xl font-semibold">{market.question}</h2>
              <span
                className={`px-3 py-1 rounded-full text-sm font-medium ${getStatusColor(
                  market.status
                )}`}
              >
                {MarketStatus[market.status]}
              </span>
            </div>

            <p className="text-gray-600 mb-4">{market.description}</p>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <span className="text-gray-600">Expiry:</span>
                <div className="font-medium">
                  {formatTimestamp(market.expiryTimestamp)}
                </div>
              </div>
              <div>
                <span className="text-gray-600">Oracle:</span>
                <div className="font-medium">{market.oracleId}</div>
              </div>
              <div>
                <span className="text-gray-600">24h Volume:</span>
                <div className="font-medium">
                  {market.volume24h.toFixed(2)} {market.collateralToken}
                </div>
              </div>
              <div>
                <span className="text-gray-600">Total Liquidity:</span>
                <div className="font-medium">
                  {market.totalLiquidity.toFixed(2)} {market.collateralToken}
                </div>
              </div>
            </div>
          </div>
        </Link>
      ))}
    </div>
  );
} 