import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useLocalStorage } from "./useLocalStorage";

export const useWallet = () => {
  const [account, setAccount] = useState<string | null>(null);
  const [balance, setBalance] = useState<string>("0");
  const [provider, setProvider] =
    useState<ethers.providers.Web3Provider | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [connected, setConnected] = useLocalStorage("walletConnected", false);

  useEffect(() => {
    const init = async () => {
      if (window.ethereum && connected) {
        const web3Provider = new ethers.providers.Web3Provider(window.ethereum);
        setProvider(web3Provider);

        try {
          const accounts = await web3Provider.listAccounts();
          if (accounts.length > 0) {
            setAccount(accounts[0]);
            const balance = await web3Provider.getBalance(accounts[0]);
            setBalance(ethers.utils.formatEther(balance));
          }
        } catch (error) {
          console.error("Failed to get accounts:", error);
          setConnected(false);
        }
      }
    };

    init();
  }, [connected]);

  useEffect(() => {
    if (window.ethereum) {
      window.ethereum.on("accountsChanged", (accounts: string[]) => {
        if (accounts.length > 0) {
          setAccount(accounts[0]);
        } else {
          setAccount(null);
          setConnected(false);
        }
      });

      window.ethereum.on("chainChanged", () => {
        window.location.reload();
      });
    }

    return () => {
      if (window.ethereum) {
        window.ethereum.removeAllListeners();
      }
    };
  }, []);

  const connect = async () => {
    if (!window.ethereum) {
      throw new Error("No Ethereum wallet found");
    }

    setConnecting(true);
    try {
      const web3Provider = new ethers.providers.Web3Provider(window.ethereum);
      await web3Provider.send("eth_requestAccounts", []);

      const accounts = await web3Provider.listAccounts();
      setAccount(accounts[0]);
      setProvider(web3Provider);
      setConnected(true);

      const balance = await web3Provider.getBalance(accounts[0]);
      setBalance(ethers.utils.formatEther(balance));
    } catch (error) {
      console.error("Failed to connect wallet:", error);
      throw error;
    } finally {
      setConnecting(false);
    }
  };

  const disconnect = () => {
    setAccount(null);
    setProvider(null);
    setConnected(false);
    setBalance("0");
  };

  const signMessage = async (message: string): Promise<string> => {
    if (!provider || !account) {
      throw new Error("Wallet not connected");
    }

    const signer = provider.getSigner();
    return await signer.signMessage(message);
  };

  return {
    account,
    balance,
    provider,
    connecting,
    connected,
    connect,
    disconnect,
    signMessage,
  };
};
