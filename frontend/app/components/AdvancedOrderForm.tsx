'use client';

import { useState } from 'react';
import { Market } from '../types/market';
import { useWallet } from '../contexts/WalletContext';
import { MarketService } from '../services/marketService';

interface AdvancedOrderFormProps {
  market: Market;
  onOrderPlaced: () => void;
}

type OrderType = 'limit' | 'stop';
type OrderSide = 'yes' | 'no';

interface OrderFormState {
  type: OrderType;
  side: OrderSide;
  amount: string;
  price: string;
}

export function AdvancedOrderForm({ market, onOrderPlaced }: AdvancedOrderFormProps) {
  const { account } = useWallet();
  const marketService = MarketService.getInstance();

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const [orderForm, setOrderForm] = useState<OrderFormState>({
    type: 'limit',
    side: 'yes',
    amount: '',
    price: '',
  });

  const handleInputChange = (field: keyof OrderFormState, value: string) => {
    setOrderForm((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!account) return;

    try {
      setIsLoading(true);
      setError(null);
      setSuccessMessage(null);

      const amount = parseFloat(orderForm.amount);
      const price = parseFloat(orderForm.price);

      if (isNaN(amount) || isNaN(price)) {
        throw new Error('Invalid amount or price');
      }

      if (orderForm.type === 'limit') {
        await marketService.placeLimitOrder(
          market.id,
          orderForm.side,
          amount,
          price
        );
      } else {
        await marketService.placeStopOrder(
          market.id,
          orderForm.side,
          amount,
          price
        );
      }

      setSuccessMessage(`Successfully placed ${orderForm.type} order`);
      setOrderForm((prev) => ({
        ...prev,
        amount: '',
        price: '',
      }));
      onOrderPlaced();
    } catch (err) {
      console.error('Error placing order:', err);
      setError('Failed to place order');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-semibold mb-4">Advanced Orders</h2>

      {error && (
        <div className="mb-4 bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded">
          {error}
        </div>
      )}

      {successMessage && (
        <div className="mb-4 bg-green-50 border border-green-200 text-green-600 px-4 py-3 rounded">
          {successMessage}
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-4">
        {/* Order Type Selection */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Order Type
          </label>
          <div className="flex space-x-2">
            {(['limit', 'stop'] as const).map((type) => (
              <button
                key={type}
                type="button"
                onClick={() => handleInputChange('type', type)}
                className={`flex-1 px-3 py-2 rounded-md text-sm font-medium ${
                  orderForm.type === type
                    ? 'bg-blue-500 text-white'
                    : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
                }`}
              >
                {type.charAt(0).toUpperCase() + type.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {/* Order Side Selection */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Token Type
          </label>
          <div className="flex space-x-2">
            {(['yes', 'no'] as const).map((side) => (
              <button
                key={side}
                type="button"
                onClick={() => handleInputChange('side', side)}
                className={`flex-1 px-3 py-2 rounded-md text-sm font-medium ${
                  orderForm.side === side
                    ? 'bg-blue-500 text-white'
                    : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
                }`}
              >
                {side.toUpperCase()}
              </button>
            ))}
          </div>
        </div>

        {/* Amount Input */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Amount
          </label>
          <input
            type="number"
            value={orderForm.amount}
            onChange={(e) => handleInputChange('amount', e.target.value)}
            placeholder="0.00"
            className="w-full rounded-md border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
            disabled={isLoading}
            required
          />
        </div>

        {/* Price Input */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Price
          </label>
          <input
            type="number"
            value={orderForm.price}
            onChange={(e) => handleInputChange('price', e.target.value)}
            placeholder="0.00"
            className="w-full rounded-md border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
            disabled={isLoading}
            required
          />
        </div>

        {/* Submit Button */}
        <button
          type="submit"
          disabled={!account || isLoading}
          className="w-full px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? 'Placing Order...' : 'Place Order'}
        </button>
      </form>

      <div className="mt-4 text-sm text-gray-500">
        <p>
          Note: {orderForm.type === 'limit' ? 'Limit' : 'Stop'} orders will be
          executed when the market price reaches your specified price.
        </p>
      </div>
    </div>
  );
} 