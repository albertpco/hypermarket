'use client';

import { useEffect, useState } from 'react';
import { Market } from '../types/market';
import { useWallet } from '../contexts/WalletContext';
import { MarketService } from '../services/marketService';

interface ActiveOrdersPanelProps {
  market: Market;
  onOrderCancelled: () => void;
}

interface Order {
  id: string;
  type: 'limit' | 'stop';
  side: 'yes' | 'no';
  amount: number;
  price: number;
  timestamp: number;
}

export function ActiveOrdersPanel({ market, onOrderCancelled }: ActiveOrdersPanelProps) {
  const { account } = useWallet();
  const marketService = MarketService.getInstance();

  const [orders, setOrders] = useState<Order[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchOrders = async () => {
    if (!account) return;

    try {
      setIsLoading(true);
      setError(null);
      const activeOrders = await marketService.getActiveOrders(market.id, account);
      setOrders(activeOrders);
    } catch (err) {
      console.error('Error fetching orders:', err);
      setError('Failed to load active orders');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchOrders();
  }, [market.id, account]);

  const handleCancelOrder = async (orderId: string) => {
    if (!account) return;

    try {
      setError(null);
      await marketService.cancelOrder(market.id, orderId);
      await fetchOrders();
      onOrderCancelled();
    } catch (err) {
      console.error('Error cancelling order:', err);
      setError('Failed to cancel order');
    }
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  if (isLoading) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold mb-4">Active Orders</h2>
        <div className="flex justify-center items-center h-32">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold mb-4">Active Orders</h2>
        <div className="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-semibold mb-4">Active Orders</h2>

      {orders.length === 0 ? (
        <div className="text-center text-gray-500 py-8">
          No active orders
        </div>
      ) : (
        <div className="space-y-4">
          {orders.map((order) => (
            <div
              key={order.id}
              className="border border-gray-200 rounded-lg p-4"
            >
              <div className="flex justify-between items-start mb-2">
                <div>
                  <span
                    className={`inline-block px-2 py-1 rounded text-sm font-medium ${
                      order.side === 'yes'
                        ? 'bg-green-100 text-green-800'
                        : 'bg-red-100 text-red-800'
                    }`}
                  >
                    {order.side.toUpperCase()}
                  </span>
                  <span className="ml-2 text-gray-600">
                    {order.type.charAt(0).toUpperCase() + order.type.slice(1)} Order
                  </span>
                </div>
                <button
                  onClick={() => handleCancelOrder(order.id)}
                  className="text-red-600 hover:text-red-700 text-sm font-medium"
                >
                  Cancel
                </button>
              </div>

              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-gray-600">Amount:</span>
                  <div className="font-medium">
                    {order.amount.toFixed(2)} {market.collateralToken}
                  </div>
                </div>
                <div>
                  <span className="text-gray-600">Price:</span>
                  <div className="font-medium">
                    {order.price.toFixed(2)} {market.collateralToken}
                  </div>
                </div>
                <div className="col-span-2">
                  <span className="text-gray-600">Placed:</span>
                  <div className="font-medium">
                    {formatTimestamp(order.timestamp)}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
} 