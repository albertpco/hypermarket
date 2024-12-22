# HyperMarket

A decentralized prediction market platform built on Hyperliquid, allowing users to create and trade binary outcome markets.

## Features

- Create and manage prediction markets
- Trade YES/NO tokens
- Integration with Hyperliquid's order book
- Oracle system for market resolution
- Collateral management
- Real-time market data and trading interface

## Project Structure

```
hypermarket/
├── contracts/           # Rust smart contracts
│   └── src/
│       ├── market.rs    # Market contract implementation
│       ├── oracle.rs    # Oracle system
│       └── events.rs    # Event handling
├── backend/            # Backend API server
│   └── src/
│       └── schema.rs   # Database schema
└── frontend/          # Next.js frontend application
    └── src/
        ├── components/ # React components
        ├── hooks/      # Custom React hooks
        └── types/      # TypeScript types
```

## Prerequisites

- Node.js (v16 or later)
- Rust (latest stable)
- PostgreSQL
- Git

## Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/hypermarket.git
   cd hypermarket
   ```

2. Install frontend dependencies:

   ```bash
   cd frontend
   npm install
   ```

3. Install Rust dependencies:

   ```bash
   cd ../contracts
   cargo build
   ```

4. Set up environment variables:

   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

5. Start the development servers:

   ```bash
   # Terminal 1: Frontend
   cd frontend
   npm run dev

   # Terminal 2: Backend
   cd backend
   cargo run

   # Terminal 3: Contracts (if needed)
   cd contracts
   cargo run
   ```

## Development

- Frontend runs on: http://localhost:3000
- Backend API runs on: http://localhost:8000
- Contracts interact with Hyperliquid testnet

## Testing

```bash
# Run frontend tests
cd frontend
npm test

# Run contract tests
cd contracts
cargo test

# Run backend tests
cd backend
cargo test
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
