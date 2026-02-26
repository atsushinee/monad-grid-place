# GridPlace

GridPlace is a full-stack, production-grade Web3 pixel ad wall designed for high-throughput environments like the Monad testnet. It serves as a reference architecture for building robust, scalable, and maintainable decentralized applications, featuring an end-to-end, event-driven data pipeline.

## Architecture: The "Industrial-Grade" Data Flow

The project follows a strictly decoupled architecture, ensuring high performance, low gas costs, and a clean separation of concerns. The data flow is unidirectional and designed to provide a responsive user experience while maintaining on-chain data integrity.

**Core Principle**: The blockchain is treated as a "write-only" command bus. The frontend reads exclusively from a fast, indexed, off-chain database, never directly from the chain.

**The Flow:**

```
1. Frontend (Paint Action)
   - POST /upload -> Backend API (uploads metadata to IPFS, returns CID & hash)
   - POST /cache  -> Backend API (caches CID/hash mapping for the Indexer)
   - writeContract('paint') -> Blockchain (sends only essential data)
     |
     v
2. Blockchain (Monad)
   - Executes `paint` transaction.
   - Emits `Painted` event (contains only indexed data like `cidHash`).
     |
     v
3. Indexer (Rust)
   - Listens for `Painted` event via WebSocket.
   - GET /cache/:cidHash -> Backend API (retrieves the original CID).
   - Writes full pixel data (including original CID) to PostgreSQL Database.
     |
     v
4. Backend API (Rust)
   - GET /grid -> Frontend (serves pixel data for display).
   - Reads from PostgreSQL.
   - Fetches full metadata from IPFS using the stored CID.
   - Returns enriched data to the frontend.
     |
     v
5. Frontend (View)
   - Displays the pixel grid, populated with data from the Backend API.
```

## Key Features

- **Industrial-Grade Data Pipeline**: Implements a sophisticated cache-and-lookup pattern between the frontend, backend, and indexer to keep on-chain transactions minimal and fast.
- **Indexer-First Design**: The application state is built via event-sourcing. The frontend is a pure representation of the indexed database state.
- **On-Chain/Off-Chain Separation**: On-chain storage is minimized to a `bytes32` hash of the IPFS CID. All rich metadata (links, descriptions) lives on IPFS, fetched by the backend.
- **Gas-Optimized & Upgradeable Contract**: Utilizes struct packing and the UUPS proxy pattern.
- **Performant Frontend**:
  - Built with **Vite + React + TypeScript**.
  - **Wagmi + Viem** for state-of-the-art wallet and blockchain interaction.
  - **Zoom & Pan**: Canvas-like navigation for a smooth user experience.
  - **Virtualization-Ready**: The grid rendering logic is architected to easily support virtualized rendering, ensuring performance even with millions of pixels.
- **Robust Backend Services**: Both the API and Indexer are built in Rust for performance and safety, featuring asynchronous operations, clear service separation, and compile-time checked SQL queries via `sqlx`.

## Technology Stack

- **Smart Contracts**:
  - Solidity `^0.8.20`, Foundry (Forge, Anvil, Cast)
  - OpenZeppelin Upgradeable Contracts (UUPS)
- **Backend API**:
  - Rust, Tokio, Axum, SQLx (PostgreSQL)
  - `DashMap` for in-memory caching.
- **Indexer**:
  - Rust, Tokio, Ethers-rs (WebSocket), SQLx
  - `Reqwest` for cache lookups from the Backend API.
- **Frontend**:
  - Vite, React, TypeScript, Tailwind CSS
  - Wagmi & Viem for Web3 state management.
  - `@tanstack/react-query` for data fetching.
- **Database**: PostgreSQL
- **DevOps**: Docker & Docker Compose, `.env` configuration.

## Project Structure

```
.
├── backend/         # Rust (Axum) API: IPFS uploads, caching, data serving
├── contracts/       # Solidity smart contracts (Foundry)
├── frontend/        # React DApp (Vite, Wagmi, Tailwind)
├── indexer/         # Rust standalone event indexer
├── ipfs/            # Docker-compose for local IPFS & PostgreSQL
└── README.md
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) & `sqlx-cli`
- [Foundry](https://getfoundry.sh/)
- [Docker](https://www.docker.com/get-started)
- [Node.js](https://nodejs.org/) & `pnpm` (or `npm`/`yarn`)

### 1. Run Infrastructure

Start PostgreSQL, IPFS, and a local testnet node.

```sh
# Start DB and IPFS
docker-compose -f ./ipfs/docker-compose.yml up -d

# Start blockchain node with CORS enabled
anvil --host 0.0.0.0 --cors-origins "*"
```

### 2. Deploy Contract

In a new terminal, set your private key from the Anvil output in `contracts/.env` and deploy.

```sh
cd contracts
cp .env.example .env
# Edit .env with a private key from Anvil
forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast
```
**Copy the new proxy contract address.**

### 3. Configure and Run Services

Update the `.env` files in `backend`, `indexer`, and the contract address in `frontend/src/components/PaintModal.tsx` with the new deployment details.

Run the database migrations:
```sh
# From the /backend directory
export DATABASE_URL="postgres://postgres:root@localhost/gridplace" # Or your URL
sqlx migrate run
```

Run each service in its own terminal:
```sh
# Terminal 1: Backend
cd backend
cargo run

# Terminal 2: Indexer
cd indexer
cargo run

# Terminal 3: Frontend
cd frontend
pnpm install
pnpm dev
```

### 4. Test the Full Flow
1. Open the frontend URL in your browser.
2. Connect your MetaMask wallet (ensure it's on the Anvil network and you've imported a funded account).
3. Click a pixel, enter a link, choose a color, and click "Paint".
4. Observe the logs in the `backend` and `indexer` terminals to see the cache-and-lookup flow in action.
5. Once the transaction is confirmed, the grid will auto-update, and clicking the pixel again will show the link you entered.
