import React, { useState } from 'react';
import { useWallet } from '../hooks/useWallet';
import { useMarket } from '../hooks/useMarket';
import { OrderType, Side } from '../types';

interface TradingInterfaceProps {
  marketId: string;
}

export const TradingInterface: React.FC<TradingInterfaceProps> = ({ marketId }) => {
  const { account, balance } = useWallet();
  const { market, orderBook, placeOrder } = useMarket(marketId);
  
  const [side, setSide] = useState<Side>('buy');
  const [price, setPrice] = useState('');
  const [amount, setAmount] = useState('');
  const [orderType, setOrderType] = useState<OrderType>('limit');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!account || !price || !amount) return;

    try {
      await placeOrder({
        side,
        price: parseFloat(price),
        amount: parseFloat(amount),
        orderType,
      });

      // Reset form
      setPrice('');
      setAmount('');
    } catch (error) {
      console.error('Failed to place order:', error);
      // Add error handling
    }
  };

  if (!market) return <div>Loading...</div>;

  return (
    <div className="bg-white rounded-lg shadow-lg p-6">
      <div className="flex justify-between mb-6">
        <h2 className="text-xl font-semibold">{market.question}</h2>
        <div className="text-sm text-gray-500">
          Balance: {balance} {market.collateral_token}
        </div>
      </div>

      <div className="flex space-x-2 mb-6">
        <button
          className={`flex-1 py-2 rounded ${
            side === 'buy'
              ? 'bg-green-500 text-white'
              : 'bg-gray-100 text-gray-700'
          }`}
          onClick={() => setSide('buy')}
        >
          Buy YES
        </button>
        <button
          className={`flex-1 py-2 rounded ${
            side === 'sell'
              ? 'bg-red-500 text-white'
              : 'bg-gray-100 text-gray-700'
          }`}
          onClick={() => setSide('sell')}
        >
          Sell NO
        </button>
      </div>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Order Type
          </label>
          <select
            value={orderType}
            onChange={(e) => setOrderType(e.target.value as OrderType)}
            className="w-full border rounded-md px-3 py-2"
          >
            <option value="limit">Limit</option>
            <option value="market">Market</option>
          </select>
        </div>

        {orderType === 'limit' && (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Price (USDC)
            </label>
            <input
              type="number"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              min="0"
              step="0.01"
              className="w-full border rounded-md px-3 py-2"
              placeholder="0.00"
            />
          </div>
        )}

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Amount
          </label>
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            min="0"
            step="1"
            className="w-full border rounded-md px-3 py-2"
            placeholder="0"
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="text-sm text-gray-500">Total Cost</p>
            <p className="text-lg font-medium">
              ${(parseFloat(price || '0') * parseFloat(amount || '0')).toFixed(2)}
            </p>
          </div>
          <div>
            <p className="text-sm text-gray-500">Max Profit</p>
            <p className="text-lg font-medium">
              ${(parseFloat(amount || '0') * (1 - parseFloat(price || '0'))).toFixed(2)}
            </p>
          </div>
        </div>

        <button
          type="submit"
          disabled={!account || !price || !amount}
          className="w-full bg-blue-500 text-white py-3 rounded-md hover:bg-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Place Order
        </button>
      </form>

      <div className="mt-6">
        <h3 className="text-lg font-medium mb-2">Order Book</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <h4 className="text-sm font-medium text-green-600 mb-2">Bids</h4>
            {orderBook.bids.map((bid, i) => (
              <div
                key={i}
                className="flex justify-between text-sm py-1"
              >
                <span>${bid.price.toFixed(2)}</span>
                <span>{bid.size}</span>
              </div>
            ))}
          </div>
          <div>
            <h4 className="text-sm font-medium text-red-600 mb-2">Asks</h4>
            {orderBook.asks.map((ask, i) => (
              <div
                key={i}
                className="flex justify-between text-sm py-1"
              >
                <span>${ask.price.toFixed(2)}</span>
                <span>{ask.size}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}; 