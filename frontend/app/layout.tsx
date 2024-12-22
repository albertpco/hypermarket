'use client';

import { Inter } from 'next/font/google';
import { WalletProvider } from './contexts/WalletContext';
import './globals.css';

const inter = Inter({ subsets: ['latin'], variable: '--font-inter' });

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className={inter.variable}>
      <body>
        <WalletProvider>
          <main className="min-h-screen bg-gray-50">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
              {children}
            </div>
          </main>
        </WalletProvider>
      </body>
    </html>
  );
}
