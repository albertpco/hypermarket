'use client';

import { useEffect, useState } from 'react';
import { Market, MarketPosition } from '../types/market';
import { useWallet } from '../contexts/WalletContext';
import { MarketService } from '../services/marketService';

interface PositionManagementProps {
  market: Market;
  onOrderPlaced: () => void;
}

export function PositionManagement({ market, onOrderPlaced }: PositionManagementProps) {
  const { account } = useWallet();
  const marketService = MarketService.getInstance();

  const [position, setPosition] = useState<MarketPosition | null>(null);
  const [burnAmount, setBurnAmount] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const fetchPosition = async () => {
    if (!account) return;

    try {
      setIsLoading(true);
      setError(null);
      const userPosition = await marketService.getUserPosition(market.id, account);
      setPosition(userPosition);
    } catch (err) {
      console.error('Error fetching position:', err);
      setError('Failed to load position data');
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchPosition();
  }, [market.id, account]);

  const handleBurnTokens = async (type: 'yes' | 'no') => {
    if (!account || !burnAmount || !position) return;

    try {
      setIsLoading(true);
      setError(null);
      setSuccessMessage(null);

      const amount = parseFloat(burnAmount);
      if (isNaN(amount)) {
        throw new Error('Invalid amount');
      }

      if (type === 'yes') {
        await marketService.burnYesTokens(market.id, amount);
      } else {
        await marketService.burnNoTokens(market.id, amount);
      }

      setSuccessMessage(`Successfully burned ${amount} ${type.toUpperCase()} tokens`);
      setBurnAmount('');
      await fetchPosition();
      onOrderPlaced();
    } catch (err) {
      console.error('Error burning tokens:', err);
      setError('Failed to burn tokens');
    } finally {
      setIsLoading(false);
    }
  };

  const handleClaimWinnings = async () => {
    if (!account || !position) return;

    try {
      setIsLoading(true);
      setError(null);
      setSuccessMessage(null);

      await marketService.claimWinnings(market.id);
      setSuccessMessage('Successfully claimed winnings');
      await fetchPosition();
    } catch (err) {
      console.error('Error claiming winnings:', err);
      setError('Failed to claim winnings');
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold mb-4">Your Position</h2>
        <div className="flex justify-center items-center h-32">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold mb-4">Your Position</h2>
        <div className="bg-red-50 border border-red-200 text-red-600 px-4 py-3 rounded">
          {error}
        </div>
      </div>
    );
  }

  if (!position) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold mb-4">Your Position</h2>
        <div className="text-center text-gray-500 py-8">
          No position in this market
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-semibold mb-4">Your Position</h2>

      {successMessage && (
        <div className="mb-4 bg-green-50 border border-green-200 text-green-600 px-4 py-3 rounded">
          {successMessage}
        </div>
      )}

      <div className="space-y-6">
        {/* Token Holdings */}
        <div className="grid grid-cols-2 gap-4">
          <div>
            <h3 className="text-lg font-medium mb-2">YES Tokens</h3>
            <div className="text-2xl font-bold text-green-600">
              {position.yesTokens.toFixed(2)}
            </div>
          </div>
          <div>
            <h3 className="text-lg font-medium mb-2">NO Tokens</h3>
            <div className="text-2xl font-bold text-red-600">
              {position.noTokens.toFixed(2)}
            </div>
          </div>
        </div>

        {/* Token Burning */}
        <div>
          <h3 className="text-lg font-medium mb-2">Burn Tokens</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Amount
              </label>
              <div className="flex space-x-2">
                <input
                  type="number"
                  value={burnAmount}
                  onChange={(e) => setBurnAmount(e.target.value)}
                  placeholder="0.00"
                  className="flex-1 rounded-md border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  disabled={isLoading}
                />
                <button
                  onClick={() => handleBurnTokens('yes')}
                  disabled={!account || !burnAmount || isLoading}
                  className="px-4 py-2 bg-green-500 text-white rounded-md hover:bg-green-600 focus:outline-none focus:ring-2 focus:ring-green-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Burn YES
                </button>
                <button
                  onClick={() => handleBurnTokens('no')}
                  disabled={!account || !burnAmount || isLoading}
                  className="px-4 py-2 bg-red-500 text-white rounded-md hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Burn NO
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Claim Winnings */}
        {market.status === 'Resolved' && position.claimableWinnings > 0 && (
          <div>
            <h3 className="text-lg font-medium mb-2">Claim Winnings</h3>
            <div className="flex items-center justify-between">
              <div className="text-2xl font-bold text-green-600">
                {position.claimableWinnings.toFixed(2)} {market.collateralToken}
              </div>
              <button
                onClick={handleClaimWinnings}
                disabled={isLoading}
                className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Claim Winnings
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
} 