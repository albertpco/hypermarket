'use client';

import React, { createContext, useContext, useState, useEffect } from 'react';
import { ethers } from 'ethers';

interface WalletContextType {
  account: string | null;
  connect: () => Promise<void>;
  disconnect: () => void;
  isConnecting: boolean;
  error: string | null;
}

const WalletContext = createContext<WalletContextType>({
  account: null,
  connect: async () => {},
  disconnect: () => {},
  isConnecting: false,
  error: null,
});

export function useWallet() {
  return useContext(WalletContext);
}

interface WalletProviderProps {
  children: React.ReactNode;
}

export function WalletProvider({ children }: WalletProviderProps) {
  const [account, setAccount] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Check if MetaMask is installed
    if (typeof window !== 'undefined' && window.ethereum) {
      // Listen for account changes
      window.ethereum.on('accountsChanged', (accounts: string[]) => {
        if (accounts.length > 0) {
          setAccount(accounts[0]);
        } else {
          setAccount(null);
        }
      });

      // Listen for chain changes
      window.ethereum.on('chainChanged', () => {
        window.location.reload();
      });
    }

    return () => {
      if (window.ethereum) {
        window.ethereum.removeListener('accountsChanged', () => {});
        window.ethereum.removeListener('chainChanged', () => {});
      }
    };
  }, []);

  const connect = async () => {
    if (typeof window === 'undefined' || !window.ethereum) {
      setError('Please install MetaMask to use this application');
      return;
    }

    try {
      setIsConnecting(true);
      setError(null);

      const provider = new ethers.providers.Web3Provider(window.ethereum);
      const accounts = await provider.send('eth_requestAccounts', []);

      if (accounts.length > 0) {
        setAccount(accounts[0]);
      }
    } catch (err) {
      console.error('Error connecting wallet:', err);
      setError('Failed to connect wallet');
    } finally {
      setIsConnecting(false);
    }
  };

  const disconnect = () => {
    setAccount(null);
  };

  return (
    <WalletContext.Provider
      value={{
        account,
        connect,
        disconnect,
        isConnecting,
        error,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
} 