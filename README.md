# GridPlace

GridPlace is a full-stack, production-grade Web3 pixel ad wall designed for high-throughput environments like the Monad testnet. It serves as a reference architecture for building robust, scalable, and maintainable decentralized applications.

## Architecture Overview

The project follows a clean, decoupled architecture where each component has a single responsibility. The data flows in a unidirectional manner from the blockchain to the frontend, ensuring data integrity and consistency.

```
[ On-Chain: Monad ] <=> [ Indexer (Rust) ] <=> [ Database (PostgreSQL) ] <=> [ Backend API (Rust) ] <=> [ Frontend (React) ]
       ^                                                                                                      ^
       |                                                                                                      |
[ User Wallet ]                                                                                        [ IPFS for Metadata ]
```

- **Smart Contract**: Stores only essential data (hashes and ownership) and emits events.
- **Indexer**: A standalone Rust service that listens for contract events via WebSocket, decodes them, and writes the structured data into a PostgreSQL database. This is the single source of truth for the off-chain state.
- **Backend API**: A stateless Rust (Axum) service that reads from the database to serve data to the frontend. It also handles IPFS uploads.
- **Frontend**: A React application that interacts with user wallets and fetches data exclusively from the Backend API.

## Key Features

- **Modular & Scalable**: Each service (Backend, Indexer) is independent and can be scaled separately.
- **Indexer-First Design**: The application state is built by an event-sourcing pattern, providing robustness and easy replayability. The frontend never directly queries the blockchain.
- **On-Chain/Off-Chain Separation**: Minimizes on-chain footprint and gas costs by storing only a `bytes32` hash of the IPFS CID on-chain. All rich metadata lives on IPFS.
- **Gas Optimized Contract**: Utilizes struct packing, `unchecked` math, and an efficient pricing model to reduce transaction costs.
- **Upgradeable & Secure**: Implements the UUPS proxy pattern for upgradeability and follows security best practices.
- **Developer Experience**: Leverages Foundry for contract development, `sqlx-cli` for compile-time checked SQL queries, and a Dockerized environment.

## Technology Stack

- **Smart Contracts**:
  - Solidity `^0.8.20`
  - Foundry (Forge, Anvil, Cast)
  - OpenZeppelin Upgradeable Contracts (UUPS)
- **Backend API**:
  - Rust, Tokio
  - Axum (Web Framework)
  - SQLx (PostgreSQL)
  - Reqwest (HTTP Client for IPFS)
- **Indexer**:
  - Rust, Tokio
  - Ethers-rs (WebSocket event subscription)
  - SQLx (PostgreSQL)
- **Database**:
  - PostgreSQL
- **DevOps**:
  - Docker & Docker Compose
  - `.env` for configuration management

## Project Structure

```
.
├── backend/         # Rust (Axum) RESTful API server
├── contracts/       # Solidity smart contracts (Foundry)
├── frontend/        # React frontend (placeholder)
├── indexer/         # Rust standalone event indexer
├── ipfs/            # Docker-compose for local IPFS node
└── README.md
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Foundry](https://getfoundry.sh/)
- [Docker](https://www.docker.com/get-started)
- [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli): `cargo install sqlx-cli --no-default-features --features rustls,postgres`

### 1. Run Infrastructure

Start the PostgreSQL database and a local IPFS node.

```sh
docker-compose -f ./ipfs/docker-compose.yml up -d
```

### 2. Deploy Contract

First, start a local testnet node.

```sh
anvil
```

In a new terminal, set your private key from the Anvil output in `contracts/.env` and deploy.

```sh
cd contracts
cp .env.example .env
# Edit .env with a private key from Anvil
forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast
```
Copy the deployed proxy contract address.

### 3. Configure and Run Services

Both the `backend` and `indexer` services require a `.env` file.

```sh
# In the /backend directory
cp .env.example .env
# Edit .env with your DB and IPFS settings

# In the /indexer directory
cp .env.example .env
# Edit .env with your RPC, DB, and the deployed contract address
```

Run the database migrations using `sqlx-cli`.

```sh
# From the /backend directory
export DATABASE_URL="postgres://postgres:root@localhost/gridplace" # Or your URL
sqlx migrate run
```

Finally, run the services (each in its own terminal).

```sh
# Terminal 1: Indexer
cd indexer
cargo run

# Terminal 2: Backend
cd backend
cargo run
```

### 4. Test the Flow

Trigger a `paint` event using `cast`.

```sh
cast send 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
"paint(uint256,uint32,bytes32)" 123 0xFF00FF \
0x1234567890123456789012345678901234567890123456789012345678901234 \
--value 0.01ether \
--private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
--rpc-url http://127.0.0.1:8545
```

You should see the event being processed in the Indexer's terminal. You can then query the data via the Backend API: `curl http://127.0.0.1:3000/grid/123/0`.
