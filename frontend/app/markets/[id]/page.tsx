'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { useWallet } from '../../contexts/WalletContext';
import { MarketService } from '../../services/marketService';
import { TradingInterface } from '../../components/TradingInterface';
import { Market, MarketStatus } from '../../types/market';
import { MarketCharts } from '../../components/MarketCharts';
import { AdvancedOrderForm } from '../../components/AdvancedOrderForm';
import { ActiveOrdersPanel } from '../../components/ActiveOrdersPanel';
import { PositionManagement } from '../../components/PositionManagement';

interface MarketDetailProps {
  params: {
    id: string;
  };
}

export default function MarketDetail({ params }: MarketDetailProps) {
  const { id } = params;
  const { account } = useWallet();
  const marketService = MarketService.getInstance();

  const [market, setMarket] = useState<Market | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchMarket = async () => {
    if (!id) return;

    try {
      setIsLoading(true);
      setError(null);
      const marketData = await marketService.getMarket(id);
      
      if (!marketData) {
        setError('Market not found');
        return;
      }

      setMarket(marketData);
    } catch (err) {
      console.error('Error fetching market:', err);
      setError('Failed to load market data');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchMarket();
  }, [id]);

  const handleOrderUpdate = () => {
    fetchMarket();
  };

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
      <div className="flex justify-center items-center min-h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (error || !market) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded">
          {error || 'Failed to load market'}
        </div>
        <Link href="/">
          <span className="mt-4 inline-block text-blue-500 hover:text-blue-600">
            ← Back to Markets
          </span>
        </Link>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <Link href="/">
        <span className="inline-block mb-6 text-blue-500 hover:text-blue-600">
          ← Back to Markets
        </span>
      </Link>

      {/* Market Header */}
      <div className="bg-white rounded-lg shadow p-6 mb-6">
        <div className="flex justify-between items-start mb-4">
          <h1 className="text-2xl font-bold">{market.question}</h1>
          <span
            className={`px-3 py-1 rounded-full text-sm font-medium ${getStatusColor(
              market.status
            )}`}
          >
            {MarketStatus[market.status]}
          </span>
        </div>

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
              {market.volume24h?.toFixed(2)} {market.collateralToken}
            </div>
          </div>
          <div>
            <span className="text-gray-600">Total Liquidity:</span>
            <div className="font-medium">
              {market.totalLiquidity?.toFixed(2)} {market.collateralToken}
            </div>
          </div>
        </div>

        {market.status === MarketStatus.Resolved && (
          <div className="mt-4 p-4 bg-blue-50 rounded-lg">
            <span className="text-blue-600 font-medium">
              Resolved Outcome: {market.resolvedOutcome ? 'Yes' : 'No'}
            </span>
          </div>
        )}
      </div>

      {/* Market Charts */}
      <div className="mb-6">
        <MarketCharts market={market} />
      </div>

      {/* Trading Interface and Advanced Features */}
      {market.status === MarketStatus.Active ? (
        account ? (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Trading Interface */}
            <div className="lg:col-span-2">
              <TradingInterface market={market} />
            </div>

            {/* Advanced Trading Features */}
            <div className="space-y-6">
              {/* Position Management */}
              <PositionManagement
                market={market}
                onOrderPlaced={handleOrderUpdate}
              />

              {/* Advanced Orders */}
              <AdvancedOrderForm
                market={market}
                onOrderPlaced={handleOrderUpdate}
              />

              {/* Active Orders */}
              <ActiveOrdersPanel
                market={market}
                onOrderCancelled={handleOrderUpdate}
              />
            </div>
          </div>
        ) : (
          <div className="bg-yellow-50 border border-yellow-200 text-yellow-600 px-4 py-3 rounded">
            Please connect your wallet to trade in this market
          </div>
        )
      ) : (
        <div className="bg-gray-50 border border-gray-200 text-gray-600 px-4 py-3 rounded">
          This market is no longer active for trading
        </div>
      )}
    </div>
  );
} 