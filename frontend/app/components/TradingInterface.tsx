'use client';

import { useState } from 'react';
import { Market } from '../types/market';
import { useWallet } from '../contexts/WalletContext';
import { MarketService } from '../services/marketService';

interface TradingInterfaceProps {
  market: Market;
}

export function TradingInterface({ market }: TradingInterfaceProps) {
  const { account } = useWallet();
  const marketService = MarketService.getInstance();

  const [yesAmount, setYesAmount] = useState('');
  const [noAmount, setNoAmount] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const handleMintYes = async () => {
    if (!account || !yesAmount) return;

    try {
      setIsLoading(true);
      setError(null);
      setSuccessMessage(null);

      await marketService.mintYesTokens(market.id, parseFloat(yesAmount));
      setSuccessMessage(`Successfully minted ${yesAmount} YES tokens`);
      setYesAmount('');
    } catch (err) {
      console.error('Error minting YES tokens:', err);
      setError('Failed to mint YES tokens');
    } finally {
      setIsLoading(false);
    }
  };

  const handleMintNo = async () => {
    if (!account || !noAmount) return;

    try {
      setIsLoading(true);
      setError(null);
      setSuccessMessage(null);

      await marketService.mintNoTokens(market.id, parseFloat(noAmount));
      setSuccessMessage(`Successfully minted ${noAmount} NO tokens`);
      setNoAmount('');
    } catch (err) {
      console.error('Error minting NO tokens:', err);
      setError('Failed to mint NO tokens');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-semibold mb-4">Trade</h2>

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

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* YES Token Trading */}
        <div>
          <h3 className="text-lg font-medium mb-2">YES Tokens</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Amount
              </label>
              <div className="flex space-x-2">
                <input
                  type="number"
                  value={yesAmount}
                  onChange={(e) => setYesAmount(e.target.value)}
                  placeholder="0.00"
                  className="flex-1 rounded-md border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  disabled={isLoading}
                />
                <button
                  onClick={handleMintYes}
                  disabled={!account || !yesAmount || isLoading}
                  className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isLoading ? 'Loading...' : 'Buy YES'}
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* NO Token Trading */}
        <div>
          <h3 className="text-lg font-medium mb-2">NO Tokens</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Amount
              </label>
              <div className="flex space-x-2">
                <input
                  type="number"
                  value={noAmount}
                  onChange={(e) => setNoAmount(e.target.value)}
                  placeholder="0.00"
                  className="flex-1 rounded-md border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  disabled={isLoading}
                />
                <button
                  onClick={handleMintNo}
                  disabled={!account || !noAmount || isLoading}
                  className="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isLoading ? 'Loading...' : 'Buy NO'}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="mt-6 text-sm text-gray-500">
        <p>
          Note: Trading requires collateral in {market.collateralToken}. Make sure
          you have sufficient balance before trading.
        </p>
      </div>
    </div>
  );
} 