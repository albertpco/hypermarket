import { MarketList } from './components/MarketList';

export default function Home() {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="text-center mb-12">
        <h1 className="text-4xl font-bold mb-4">
          Welcome to HyperMarket
        </h1>
        <p className="text-xl text-gray-600 mb-8">
          Decentralized prediction markets powered by Hyperliquid
        </p>
        <a
          href="/markets/create"
          className="inline-block bg-blue-500 text-white px-6 py-3 rounded-lg hover:bg-blue-600 transition-colors"
        >
          Create New Market
        </a>
      </div>

      <MarketList />
    </div>
  );
}
