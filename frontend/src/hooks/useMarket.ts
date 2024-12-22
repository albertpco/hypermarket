import { useState, useEffect } from "react";
import { useWallet } from "./useWallet";
import { Market, OrderBook, PlaceOrderParams } from "../types";
import { ethers } from "ethers";

const API_URL = process.env.NEXT_PUBLIC_API_URL;

export const useMarket = (marketId: string) => {
  const { provider, account, signMessage } = useWallet();
  const [market, setMarket] = useState<Market | null>(null);
  const [orderBook, setOrderBook] = useState<OrderBook>({ bids: [], asks: [] });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchMarket = async () => {
      try {
        const response = await fetch(`${API_URL}/markets/${marketId}`);
        if (!response.ok) throw new Error("Failed to fetch market");
        const data = await response.json();
        setMarket(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to fetch market");
      }
    };

    fetchMarket();
  }, [marketId]);

  useEffect(() => {
    const fetchOrderBook = async () => {
      try {
        const response = await fetch(
          `${API_URL}/markets/${marketId}/orderbook`
        );
        if (!response.ok) throw new Error("Failed to fetch order book");
        const data = await response.json();
        setOrderBook(data);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to fetch order book"
        );
      }
    };

    const interval = setInterval(fetchOrderBook, 5000); // Update every 5 seconds
    fetchOrderBook();

    return () => clearInterval(interval);
  }, [marketId]);

  const placeOrder = async ({
    side,
    price,
    amount,
    orderType,
  }: PlaceOrderParams) => {
    if (!provider || !account) {
      throw new Error("Wallet not connected");
    }

    try {
      // Create order signature
      const message = JSON.stringify({
        action: "place_order",
        market_id: marketId,
        side,
        price,
        amount,
        order_type: orderType,
        timestamp: Date.now(),
      });

      const signature = await signMessage(message);

      // Submit order to API
      const response = await fetch(`${API_URL}/orders`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          market_id: marketId,
          side,
          price,
          amount,
          order_type: orderType,
          signature,
          message,
        }),
      });

      if (!response.ok) {
        throw new Error("Failed to place order");
      }

      const order = await response.json();
      return order;
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to place order");
      throw err;
    }
  };

  const getUserPosition = async () => {
    if (!account) return null;

    try {
      const response = await fetch(
        `${API_URL}/markets/${marketId}/positions/${account}`
      );
      if (!response.ok) throw new Error("Failed to fetch position");
      return await response.json();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch position");
      return null;
    }
  };

  const getUserOrders = async () => {
    if (!account) return [];

    try {
      const response = await fetch(
        `${API_URL}/markets/${marketId}/orders/${account}`
      );
      if (!response.ok) throw new Error("Failed to fetch orders");
      return await response.json();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch orders");
      return [];
    }
  };

  const cancelOrder = async (orderId: number) => {
    if (!provider || !account) {
      throw new Error("Wallet not connected");
    }

    try {
      const message = JSON.stringify({
        action: "cancel_order",
        market_id: marketId,
        order_id: orderId,
        timestamp: Date.now(),
      });

      const signature = await signMessage(message);

      const response = await fetch(`${API_URL}/orders/${orderId}`, {
        method: "DELETE",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          signature,
          message,
        }),
      });

      if (!response.ok) {
        throw new Error("Failed to cancel order");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to cancel order");
      throw err;
    }
  };

  return {
    market,
    orderBook,
    loading,
    error,
    placeOrder,
    getUserPosition,
    getUserOrders,
    cancelOrder,
  };
};
