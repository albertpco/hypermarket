HyperMarket: A Prediction & Event Marketplace on Hyperliquid
Mission:
HyperMarket will be a fully onchain event and prediction market where users can trustlessly create and trade on outcomes of real-world and crypto-native events. It leverages Hyperliquid’s low-latency, high-throughput order book and L1 infrastructure to deliver a frictionless, CEX-like trading experience—transparently and permissionlessly.

Key Differentiators:

Fully Onchain Order Book: No reliance on offchain servers or semi-centralized matching engines. Complete transparency and auditability.
High Speed & Low Latency: Near-instant order placement and confirmation (200ms blocks), enabling tight spreads and high-frequency strategies.
Composable & Trustless: Built natively into Hyperliquid L1, HyperMarket can integrate with other DeFi primitives, vaults, and lending markets that may emerge post-EVM integration.
Core Components
Event Market Contracts

MarketFactory Contract:
A contract to create new event markets. Anyone can propose a market by specifying:

Question & Resolution Criteria (e.g., “Will ETH be above $2,000 at 12:00 UTC on Dec 31?”).
Expiry timestamp (after which the event will be resolved).
Oracle reference (ID of the oracle data feed that provides event outcomes).
Collateral type (likely USDC).
Market Contract: Each deployed Market contract represents a single event with binary or multi-outcome tokens.

Mints and burns event outcome tokens (e.g., YES/NO) from collateral.
Integrates with Hyperliquid’s order book to list pairs (YES-USDC, NO-USDC).
Settles at expiry based on the oracle’s reported outcome. One outcome token redeems at 1 USDC if correct, 0 if incorrect.
Oracle Integration

OracleManager Contract:
Early version: a trusted committee or whitelisted oracle provider updates the final outcome.
Future: integrate a decentralized oracle solution or a dispute resolution layer. This can be done by:
Allowing stakers to challenge outcomes.
Using existing oracle networks (e.g. Pyth, UMA’s optimistic oracle, etc.) for trust-minimized data feeds.
User Interface (HyperMarket Frontend)

Market Directory:
Show all active and upcoming markets with real-time order book data pulled from Hyperliquid.
Include filters for categories (crypto prices, NFT floor indexes, macro events, sports, governance proposals).

Trading Interface:
Familiar, order book-centric UI with price charts, order depth, and recent trades.
Simple forms to buy/sell YES or NO outcome tokens at market or limit orders.
Position tracking: show PnL, positions held, and “time until resolution.”

Settlement & Claiming Interface:
Once an event resolves, allow users to claim winnings by redeeming winning tokens for USDC.

Backend Services

Indexers & Analytics:
A dedicated indexer to parse blockchain data and provide quick API endpoints to the frontend.
Store event metadata (questions, expiry, oracle IDs), historical prices, and trade volumes.

Notifications:
Email or Telegram alerts for market resolution, large orders filled, or expiring soon markets.

Governance & Fees

Fee Model:
A small trading fee (e.g., 0.2-0.5%) on trades. This can be directed to a treasury or distributed among liquidity providers, market creators, or stakers in future governance models.

Market Creation Fee:
Require a listing fee or a stake to prevent spam. The fee might be returned if the market achieves a certain trading volume, or partially burned to maintain scarcity and quality.

Scalability & Future Integrations

EVM Integration (Phase 2):
When HyperEVM launches, allow developers to build complementary dApps:

Automated market makers or vaults that continuously provide liquidity to HyperMarket.
Complex derivatives on top of prediction markets.
Lending/borrowing against outcome tokens as collateral.
Advanced Oracle Mechanisms:
Move from a trusted committee to a decentralized oracle network with reputation systems and dispute resolution to enhance trustlessness and censorship-resistance.

Development Roadmap
Phase 1: MVP (3–4 Months)

Contracts:
Implement MarketFactory and Market contracts for binary events.
Integrate a simple OracleManager with a trusted oracle key.

Frontend & Trading UI:
Build a basic web app allowing users to browse active markets, buy/sell YES/NO tokens, and see PnL.
Fetch order book data from Hyperliquid and visualize it.

Testing & Audit:
Deploy on Hyperliquid testnet.
Conduct security audits on Market contracts and OracleManager.
Run simulated events and ensure smooth settlement.

Launch:
Go live with a handful of curated markets (e.g., Crypto price endpoints, NFT floor indexes).

Phase 2: Feature Enhancements (4–6 Months)

Expanded Market Types:
Support scalar and categorical markets (more than two outcomes).

Decentralized Oracles & Dispute Resolution:
Introduce a system allowing multiple oracle proposals. Users can stake tokens to dispute incorrect outcomes.

Advanced UI/UX:
Add advanced charts, historical trade data, user position analytics, watchlists.

Governance Token (Optional):
Consider introducing a native governance token to steer protocol parameters, fee splits, and new feature rollouts.

Phase 3: EVM Integration & Ecosystem Growth

Leverage HyperEVM to build dApps that integrate directly with outcome tokens.
Launch structured products, automated liquidity provisioning strategies, and more complex financial instruments on top of HyperMarket’s events.
Marketing & User Acquisition
Community Engagement:
Host “special event” markets around popular crypto events (e.g., protocol upgrades, ETF approvals).
Collaborate with other Hyperliquid-based dApps.

Educational Content:
Tutorials on how to trade on HyperMarket, basics of prediction markets, and responsible event speculation.

Partnerships:
Partner with data providers for credible event outcomes.
Integrate with analytics tools like Hyperliquid dashboards and HypurrScan for transparency.

Conclusion

HyperMarket aims to become the go-to venue for permissionless, onchain prediction markets built atop Hyperliquid. It will bring a Polymarket/Kalshi-like experience to a highly performant L1, combining the trustlessness of DeFi with near-CEX execution quality. Over time, as the EVM and ecosystem mature, HyperMarket can evolve into a fully realized event-driven financial platform, hosting thousands of markets, diverse participants, and robust liquidity.

Below is a comprehensive specification and planning document for the HyperMarket platform, detailing features, hosting strategies, and user journeys for different stakeholder types. The objective is to provide a clear blueprint for building, deploying, and maintaining HyperMarket on the Hyperliquid L1.

Feature List
Core Trading Functionality
Market Creation:
Users can propose new prediction markets by specifying:
Market question (e.g., “Will ETH > $2,000 at 12:00 UTC on Dec 31?”)
Expiry timestamp
Oracle reference (ID of oracle feed or resolution criteria)
Collateral type (USDC)
A listing fee or stake is required to prevent spam.
Outcome Token Minting:
Once a market is approved and created, users can deposit collateral (USDC) to mint outcome tokens (YES/NO or multiple outcomes).
Tokens are fully fungible and tradable.
Onchain Order Book Integration:
Each outcome token pair (e.g., YES/USDC) is listed on Hyperliquid’s fully onchain order book.
Users can place limit/market orders to buy/sell outcome tokens.
Near real-time matching (200ms block times), CEX-like experience.
Settlement & Redemption:
After the expiry, the oracle updates the final outcome.
Winning outcome tokens settle at 1 USDC, losing tokens settle at 0.
Users can redeem winnings directly from the Market contract.
Oracle & Data Feeds
Oracle Manager:
Integrates with a trusted oracle (initially) that sets the final outcome after event expiration.
Future versions: multiple data providers, dispute resolution mechanisms, possibly UMA’s optimistic oracle or a committee-based resolution with staking and slashing.
User Experience & Interface
Market Directory:
List all active, upcoming, and expired markets.
Filtering by category (crypto, NFTs, sports, governance) and status (active, resolving, settled).
Trading Interface:
Live order books, price charts, trade history.
Simple order forms: buy/sell YES or NO tokens at market or limit price.
Portfolio & Analytics:
User dashboard showing:
Current holdings (outcome tokens)
Unsettled PnL
Resolved markets and redeemed winnings
Historical trade data and performance graphs.
Notifications & Alerts:
Email/Telegram/Push notifications for:
Market resolution events
Approaching expiry of markets
Order fills and partial fills
Governance & Fees
Fee Structure:
Trading fees (e.g., 0.2-0.5% taker fee) applied to each trade.
Market listing fee to prevent spam (refundable under certain conditions).
Revenue Distribution:
Fees may accrue to a treasury contract.
Future governance token could decide usage of funds (e.g., insurance fund, liquidity incentives).
Advanced Features (Future)
Scalar & Multi-Outcome Markets:
Support for events with multiple outcomes (e.g., “Which team wins?”) and scalar payouts.
Staking & Dispute Resolution:
Community-driven dispute mechanism for oracle updates.
Integration with HyperEVM:
Allow other dApps on Hyperliquid to build on top of outcome tokens (e.g., structured products, vault strategies).
Hosting and Infrastructure Plan
Smart Contracts (On-Chain):

Deployment on Hyperliquid L1:
MarketFactory and Market contracts
OracleManager contract
Fee/Treasury contract
These run directly on-chain, no hosting needed aside from maintaining nodes for indexing.
Indexing and Backend Services:

Node Providers:
Operate private Hyperliquid full nodes or use reliable third-party node services to read chain state.
Indexers & APIs (Off-Chain):
Use a service like The Graph (when supported by Hyperliquid) or build custom indexing scripts.
Host on a cloud provider (AWS, GCP, or Azure).
Containerized microservices (Docker/Kubernetes) for easy scaling.
Caching & Databases:
Postgres or TimescaleDB to store historical trade data, market metadata, and user portfolio calculations.
Redis for caching frequently accessed data (market lists, order book snapshots).
Frontend (Web Interface):

Static Frontend Hosting:
Deploy a React/Next.js SPA to a global CDN (e.g., Cloudflare Pages, Vercel).
API Gateway:
A public API endpoint (GraphQL or REST) to serve market data and user-specific information to the frontend.
Security & SSL:
Use HTTPS, secure headers.
Possibly integrate hardware security modules (HSM) for signing transactions if the platform runs admin keys.
Decentralized Hosting (Optional):

For redundancy and trustlessness, host static frontend on IPFS or Arweave.
Maintain a DNS link (e.g., ENS or traditional DNS) for user-friendly access.
User Types & Journeys
User Personas
Retail Trader:
Wants to speculate on outcomes, placing bets and taking profits.
Market Creator:
Proposes new event markets, stakes fees, and sets initial parameters.
Liquidity Provider / Market Maker:
Places orders, tightens spreads, and earns from trading fees.
Oracle Operator:
Provides outcome resolution data post-expiry.
Advanced DeFi User/Integrator:
Uses outcome tokens in external DeFi protocols or builds automated strategies.
Retail Trader Journey
Discovery:
Visits HyperMarket, sees a list of active markets.
Select a Market:
Finds “Will ETH > $2,000 at year-end?”, checks expiration, reads description.
Connect Wallet:
Connects via a Web3 wallet (e.g., Metamask, once EVM integration is live, or native Hyperliquid wallet).
Place an Order:
Buys 100 YES tokens at market price using USDC.
Track Position:
Watches price movements, can sell before expiry if sentiment changes.
Expiry & Settlement:
If YES is correct, redeems tokens for USDC after oracle posts outcome.
PnL Realization:
Collects profits or accepts losses.
Market Creator Journey
Create a Market:
Navigates to “Create Market” page, inputs:
Question: “Will BTC ETF be approved by Dec 31?”
Expiry: Dec 31, 12:00 UTC
Oracle: Chosen from a list or references a known data provider
Stake Listing Fee: Deposits a small USDC fee to create the market.
Market Goes Live: Once approved (immediate or after a governance check), the market is visible to traders.
Earn from Market Popularity: If fee distribution favors creators, they earn a portion of trading fees.
Liquidity Provider / Market Maker Journey
Identify Market Opportunity: Chooses a popular event with wide spreads.
Deposit Collateral & Mint Tokens: Creates YES/NO tokens and lists them at competitive prices.
Active Market Making: Continuously updates orders for best bid/ask to earn spread and trading fees.
Hedging: May hedge in external markets or other Hyperliquid perps if correlated.
Oracle Operator Journey
Pre-Event: Registers as an oracle operator with the OracleManager contract.
Post-Expiry: Checks reference data sources off-chain.
Update Outcome On-Chain: Calls resolve() on Market contract with the correct outcome.
Reputation & Trust: Maintaining honest reporting to keep trust and avoid slashing (in future versions).
Advanced DeFi User/Integrator Journey
Integration with Other Protocols: Uses the system’s APIs to integrate outcome tokens into a yield strategy.
Building on EVM: Develops a smart contract that automatically buys YES/NO tokens and hedges on other Hyperliquid perps.
Custom Frontend: May build specialized analytics dashboards or mobile apps to enhance UX.
Security & Compliance
Smart Contract Audits: Mandatory third-party audits before mainnet launch.
Bug Bounty Program: Incentivize the community to find and report vulnerabilities.
KYC/Compliance (If Needed): Depending on jurisdiction, might integrate KYC flows for certain market types (especially if resembling regulated event contracts).
Growth & Iteration
Phase 1 (MVP):

Launch with a handful of binary crypto price markets.
Basic trusted oracle.
Collect user feedback, ensure performance under real conditions.
Phase 2 (Scaling Features):

Add more event categories (NFT floors, sports outcomes).
Introduce dispute resolution and oracle decentralization.
Improve UI/UX with charts, analytics, mobile app compatibility.
Phase 3 (Ecosystem Expansion):

Integrate with lending/borrowing protocols on HyperEVM.
Token-based governance.
Partner with data providers, media, and other DeFi projects.
Conclusion
By following this comprehensive plan, HyperMarket can deliver a Polymarket/Kalshi-like onchain event and prediction market experience on Hyperliquid. The outlined feature set, hosting strategy, and user journeys ensure a scalable, secure, and user-friendly platform, poised to benefit from Hyperliquid’s unique advantages and the broader DeFi ecosystem.

Below is a suggested approach to getting the project off the ground, broken down into practical steps and phases. This includes assembling the team, setting up the development environment, defining core architecture choices, and establishing initial deliverables.

1. Team & Resource Assembly
   Key Roles:

Smart Contract Engineer(s): Experienced with Rust (for Hyperliquid L1) and potentially Solidity (for future EVM integration). Must be comfortable with order book logic and cryptoeconomic design.
Backend/Infrastructure Engineer(s): Skilled in setting up nodes, indexing blockchain data, building APIs, and managing databases.
Frontend Developer(s): Proficient in React/Next.js (or similar), able to build fast, intuitive UIs and integrate with Web3 wallets and APIs.
UI/UX Designer: Creates user flows, wireframes, and high-fidelity mockups for a seamless experience.
Oracle Integration Specialist (optional at early stage): If using external data feeds or planning a custom oracle solution early on, this person ensures accurate and secure data input.
DevOps/QA: Manages deployments, continuous integration/continuous delivery (CI/CD), testing pipelines, and environment setups.
Initial Step:

Identify at least one full-stack or lead developer to architect the system.
Secure smart contract and front-end talent.
Begin small, with a skeleton crew, and add specialists as complexity grows. 2. Development Environment Setup
Local Development:

Hyperliquid Node: Set up a local Hyperliquid testnet node or use a public testnet endpoint if available.
Contracts and Tooling:
Review the Hyperliquid documentation and SDKs.
Set up a Rust environment (e.g., Rustup) if contracts need to be written in Rust.
Consider using external tools similar to Truffle/Hardhat (if adapted) or any custom frameworks recommended by Hyperliquid’s dev docs.
Repository Structure:

Monorepo (recommended): One repository containing separate folders for contracts, frontend, backend, infrastructure.
Version Control (Git): Initialize a GitHub/GitLab repo for code management.
CI/CD: Integrate a CI pipeline (GitHub Actions/CircleCI) for automated tests and linting on every commit.
Initial Step:

Clone or create a boilerplate project that can compile and deploy a simple contract to Hyperliquid testnet.
Set up a minimal frontend to interact with the deployed contract (e.g., a “Hello World” contract). 3. Defining the MVP Scope & Architecture
MVP Objectives:

Launch a single binary event market contract.
Integrate a simple trusted oracle that can be manually updated by a designated account.
Allow users to mint YES/NO tokens and trade them on the Hyperliquid order book.
Implement a bare-bones frontend displaying one or two test markets, enabling basic buy/sell orders and settlement.
Smart Contracts (Phase 1):

Implement MarketFactory and Market contracts:
MarketFactory to create a single test market with fixed parameters.
Market to handle minting/burning of outcome tokens, and settlement after expiry.
OracleManager contract with a single admin key that sets the outcome post-expiry.
Frontend (Phase 1):

Build a minimal React-based interface:
Connect wallet (if applicable, or use a browser extension/wallet recommended by Hyperliquid).
Display the test market, let user mint and trade outcome tokens.
Show order book data pulled from Hyperliquid nodes or APIs.
Settling outcome: a simple “Settle” button after expiry that displays user’s redeemable amount.
Backend & Indexing (Phase 1):

Set up a basic indexer to store market state and user positions in a local PostgresDB.
Implement a simple REST or GraphQL API that the frontend can query for trades, order history, and user balances (initially could be minimal, potentially just reading directly from chain state if simple enough).
Initial Step:

Draft the contract interfaces (in a .sol or .rs file, depending on chain requirements).
Write unit tests for contract logic (minting tokens, resolving markets).
Deploy a test version of the Market contract on Hyperliquid testnet and interact with it locally. 4. Iterative Development & Testing
Local Testing:

Write extensive unit tests for smart contracts:
Creating a market.
Minting YES/NO tokens.
Trading logic (might be partly handled by Hyperliquid since it runs the order book, but you’ll test integration assumptions).
Resolving outcomes and verifying correct payouts.
Integration Tests:

Run scripts that simulate a user journey:
User mints YES tokens.
Another user buys YES tokens from the order book.
Event expires; oracle sets the result.
User redeems winnings.
Frontend Testing:

Use testnets and mock responses to ensure the UI displays correct data.
Manually place orders and verify they appear in order book.
Check responsive design and basic UX flows. 5. Deployment to Testnet & Feedback Loop
Testnet Deployment:

Deploy the MVP contracts to Hyperliquid testnet.
Host the frontend on a temporary domain or localhost.
Ask internal team members or a small group of alpha testers to use it and provide feedback.
Iterate Based on Feedback:

Fix bugs, improve UI/UX.
Add small features like a basic PnL calculator or a cleaner order form. 6. Security & Audits
Pre-Mainnet Actions:

Once MVP contracts are stable, engage a third-party auditor.
Run a bug bounty program on testnet.
Optimize gas usage, ensure no reentrancy or logic flaws.
Prepare documentation and guides for users.
Security Testing:

Add fuzz testing and property-based tests for contracts.
Simulate malicious scenarios (e.g., oracle misreporting, no liquidity, etc.). 7. Mainnet Launch Planning
Before Mainnet:

Ensure all core functionalities are robust.
Implement fee logic and treasury, if part of MVP scope.
Prepare marketing material, FAQs, and tutorials.
Soft Launch on Mainnet:

Launch with a small number of handpicked markets.
Limit deposit sizes or impose whitelists if concerned about security at launch.
Collect User Feedback & Iterate:

Monitor performance, fees, user volumes.
Add new features as planned (scalar markets, multiple outcomes, better oracle solutions) once confidence is gained. 8. Roadmap Beyond MVP
Add new market creation UI.
Integrate a decentralized oracle or dispute resolution mechanism.
Optimize liquidity flows, add incentives for market makers.
Explore EVM integration for cross-dApp composability.
Incrementally decentralize governance (if desired).
Summary of Starting Steps:

Hire a Lead Developer & Form a Core Team.
Set Up Dev Environment: Nodes, repo, CI, basic contracts.
Define MVP Contracts & UI: Write initial Market and Oracle contracts, simple frontend.
Local & Testnet Testing: Unit tests, integration tests, deploy to testnet.
Iterate & Refine: Based on testnet usage and internal feedback.
Security Audit & Bug Bounty: Ensure safety and reliability.
Mainnet Launch (Soft): Start small, monitor closely, scale up as confidence grows.
By following these steps, you create a clear path to building the initial version of HyperMarket on Hyperliquid. You lay down a foundation that can be iterated upon and expanded as the project matures.

Below are practical steps to begin building the project as defined by the provided file structure and code. The instructions assume you already have a development environment set up and that you want to work on and extend this project, possibly integrating it into the broader HyperMarket vision described earlier.

Prerequisites
Python 3.10: The codebase specifies Python 3.10 as the development runtime.
Poetry: The project uses Poetry for dependency management. If you don't have Poetry installed, you can get it from https://python-poetry.org/ or by running curl -sSL https://install.python-poetry.org | python3 -.
Git: To manage source code and versioning.
Step-by-Step Guide

1. Set up the repository If you have not cloned the repository yet:

bash
Copy code
git clone <URL_TO_THIS_REPOSITORY>
cd hyperliquid-python-sdk 2. Review the Project Structure

hyperliquid/: Core Python SDK code.
examples/: Example scripts demonstrating usage of the SDK (e.g., placing orders, transferring funds).
tests/: Test suite with recorded cassettes (using pytest and pytest-recording) for end-to-end testing.
Makefile: Provides convenient commands for installation, testing, linting, and formatting.
pyproject.toml: Defines project metadata and Poetry-managed dependencies.
requirements\*.txt: Generated by Poetry’s export; requirements.txt and requirements-ci.txt help with CI and other workflows.
.pre-commit-config.yaml: Configuration for pre-commit hooks (linting/formatting checks). 3. Install Dependencies Ensure Python 3.10 is your active environment. Then install project dependencies using Poetry:

bash
Copy code
poetry install
This creates a virtual environment and installs all required packages. You can also run:

bash
Copy code
make install
which executes poetry install along with some other setup steps.

4. Run Tests To verify that everything is working correctly, run:

bash
Copy code
make test
This will:

Run the Python test suite using pytest.
Display coverage information.
Ensure your environment matches the project’s expectations. 5. Code Formatting and Linting To maintain code quality:

bash
Copy code
make codestyle
This runs pyupgrade, isort, and black as configured in pyproject.toml. If you want to ensure code adheres strictly to formatting and linting guidelines, also run:

bash
Copy code
make check-codestyle
make check 6. Explore Examples The examples/ directory contains scripts showing how to interact with the Hyperliquid API. For example:

bash
Copy code
python examples/basic_order.py
Before running examples, edit examples/config.json with your secret_key and account_address. Use cp examples/config.json.example examples/config.json and populate it accordingly. This lets you experiment with placing orders, querying user states, or transferring funds on the testnet.

7. Integrate With HyperMarket (Event & Prediction Market) Plans If your ultimate goal is to build an event and prediction market layer (HyperMarket) on Hyperliquid:

Smart Contracts & Protocol Integration: The current repo focuses on a Python SDK for trading. To add prediction markets, you’ll need:
Smart contracts deployed on Hyperliquid L1 to create and manage event markets.
Oracle mechanisms for resolution.
Back-End Logic: Create additional Python modules that:
Interact with new contracts (once deployed).
Index events and outcomes.
Incorporate order placement logic using this SDK to provide onchain liquidity for event markets.
Front-End Integration: Build a UI (outside of this repo) that uses this SDK on the backend. You’d run a backend service in Python that talks to the chain via the hyperliquid SDK and exposes REST or GraphQL endpoints to a frontend. 8. Add New Features Incrementally

Start by defining a prototype event market contract on Hyperliquid testnet.
Use this SDK to:
Register new spot or perp markets linked to the event outcomes (if the chain supports custom asset listing).
Manage orders and liquidity.
Extend info.py or add new utility files to handle event-based queries and integrate them into examples/ to test workflows end-to-end. 9. DevOps & CI

The Makefile and .pre-commit-config.yaml ensure code standards and automated checks.
For integration into CI/CD, use requirements-ci.txt and make check or make test steps in your CI pipeline. 10. Documentation & Community

Update README.md with instructions for any new features you add.
Consider writing additional docs or markdown files to explain the event market creation and settlement process.
Summary
To start building from this codebase:

Install dependencies via poetry install.
Run make test to confirm everything is working.
Adjust examples/config.json and run example scripts to get familiar with the SDK.
Extend or create new Python modules to implement the event and prediction market logic (HyperMarket).
Deploy and integrate new smart contracts and oracles on Hyperliquid.
Continuously test, lint, and format code before committing and pushing changes.
This approach provides a stable foundation to begin iterating on the prediction market features while ensuring you maintain code quality and alignment with the project’s existing tooling and structure.
