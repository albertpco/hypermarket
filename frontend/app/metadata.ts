import { Metadata } from "next";

export const metadata: Metadata = {
  title: "HyperMarket - Prediction Markets",
  description: "Decentralized prediction markets powered by Hyperliquid",
  keywords: [
    "prediction markets",
    "decentralized",
    "hyperliquid",
    "trading",
    "blockchain",
    "ethereum",
  ],
  authors: [{ name: "HyperMarket Team" }],
  creator: "HyperMarket Team",
  publisher: "HyperMarket",
  robots: {
    index: true,
    follow: true,
  },
  openGraph: {
    type: "website",
    locale: "en_US",
    url: "https://hypermarket.xyz",
    siteName: "HyperMarket",
    title: "HyperMarket - Prediction Markets",
    description: "Decentralized prediction markets powered by Hyperliquid",
    images: [
      {
        url: "https://hypermarket.xyz/og-image.png",
        width: 1200,
        height: 630,
        alt: "HyperMarket - Prediction Markets",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "HyperMarket - Prediction Markets",
    description: "Decentralized prediction markets powered by Hyperliquid",
    images: ["https://hypermarket.xyz/twitter-image.png"],
    creator: "@hypermarket",
  },
  viewport: {
    width: "device-width",
    initialScale: 1,
    maximumScale: 1,
  },
  icons: {
    icon: "/favicon.ico",
    shortcut: "/favicon-16x16.png",
    apple: "/apple-touch-icon.png",
  },
  manifest: "/site.webmanifest",
  themeColor: "#0ea5e9",
  colorScheme: "light",
};
