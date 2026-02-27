# GridPlace

**GridPlace** is a fully decentralized, on-chain pixel art marketplace and collaborative canvas built on the Monad blockchain. It combines blockchain technology with IPFS distributed storage to create a trustless, transparent, and community-owned digital art platform.

## 🎯 Project Vision

GridPlace reimagines the concept of collaborative digital art through a decentralized lens:

- **True Ownership**: Every pixel is an NFT that you truly own, recorded immutably on-chain
- **Censorship Resistance**: No central authority can modify or remove your artwork
- **Community Governance**: The canvas evolves through decentralized coordination
- **Composability**: Open APIs and on-chain data enable third-party integrations and derivatives

## 🏗️ Architecture: On-Chain + Off-Chain Hybrid Model

GridPlace implements a sophisticated hybrid architecture that balances decentralization, performance, and cost:

### Design Philosophy

1. **On-Chain for Trust**: Critical state (ownership, provenance) lives on-chain for verifiability
2. **IPFS for Permanence**: Rich metadata stored on IPFS ensures data persistence without bloating the chain
3. **Off-Chain for Performance**: Indexed database provides fast queries for frontend rendering
4. **Event-Driven Sync**: Indexer listens to chain events and maintains off-chain state consistency

### Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        USER INTERACTION                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         FRONTEND                                 │
│  • React + TypeScript                                           │
│  • Wagmi + Viem for Web3                                        │
│  • Multi-select & Box-select UI                                 │
│  • Real-time canvas rendering                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      BACKEND API (Rust)                          │
│  1. Generate Snapshot (merge old + new pixels)                  │
│  2. Upload to IPFS → Get CID                                    │
│  3. Calculate Price (only charge for new pixels)                │
│  4. Return cidHash & totalPrice                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SMART CONTRACT (On-Chain)                     │
│  • paintArea(indices, cidHash)                                  │
│  • Verify payment (only for pixels you don't own)               │
│  • Update ownership on-chain                                    │
│  • Emit AreaPainted event                                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      INDEXER (Rust)                              │
│  1. Listen to AreaPainted events                                │
│  2. Fetch CID from backend cache                                │
│  3. Download snapshot from IPFS                                 │
│  4. Parse & store pixel data to PostgreSQL                      │
│  5. Record snapshot history                                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     POSTGRESQL DATABASE                          │
│  • grid_cells: x, y, owner, color, link, message, timestamp     │
│  • snapshot_history: cid, cid_hash, pixel_count, tx_hash        │
│  • Indexed for fast queries                                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   FRONTEND (Read Path)                           │
│  • Query backend API /grid                                      │
│  • Render canvas with optimized SVG borders                     │
│  • Real-time updates via polling                                │
└─────────────────────────────────────────────────────────────────┘
```

## ✨ Key Features

### 🎨 Smart Canvas
- **1000x1000 Pixel Grid**: One million pixels for endless creativity
- **Multi-Select Mode**: Hold `Ctrl/Cmd` + Click to select multiple pixels
- **Box-Select Mode**: Hold `Shift` + Drag to select rectangular areas
- **Smart Borders**: Connected selections show only outer borders with clean SVG rendering
- **Colored Pixel Visibility**: Already-colored pixels show continuous colors without borders

### 💎 IPFS Snapshot System
- **Efficient Storage**: Full pixel metadata (color, link, message, timestamp) stored on IPFS
- **On-Chain Minimalism**: Only `bytes32 cidHash` stored on-chain
- **Snapshot History**: Every paint action creates a new IPFS snapshot, enabling historical replay
- **Merkle Verification**: CID hash provides cryptographic integrity guarantees

### 💰 Fair Pricing Model
- **Pay for New Pixels Only**: Updating your own pixels is FREE
- **Market-Driven**: Acquiring others' pixels costs current market price (10% appreciation)
- **No Gas Waste**: Batch operations reduce per-pixel gas costs
- **Automatic Refunds**: Overpayment is automatically refunded

### 🔧 Developer Experience
- **Type-Safe**: Full TypeScript support with strict typing
- **Modular Architecture**: Clear separation between backend, indexer, and frontend
- **Hot Reload**: Vite HMR for rapid frontend development
- **SQL Compile-Time Checks**: SQLx ensures query correctness at compile time

## 🛠️ Technology Stack

### Smart Contracts
- **Solidity** `^0.8.20` with Foundry
- **OpenZeppelin Upgradeable** (UUPS pattern)
- **Gas Optimization**: Struct packing, lazy initialization
- **Security**: ReentrancyGuard, Ownable, upgradeable proxy

### Backend (Rust)
- **Framework**: Axum (Tokio-based async web framework)
- **Database**: SQLx with PostgreSQL (compile-time checked queries)
- **IPFS Integration**: HTTP API for adding/fetching snapshots
- **Caching**: DashMap for in-memory CID cache
- **CORS**: Full cross-origin support for frontend

### Indexer (Rust)
- **Blockchain**: Ethers-rs with WebSocket for real-time events
- **Event Processing**: AreaPainted event listener
- **IPFS Fetching**: Downloads and parses snapshots
- **Database Sync**: Batch inserts with upsert logic

### Frontend
- **Framework**: React 18 + TypeScript + Vite
- **Web3**: Wagmi + Viem for wallet and contract interaction
- **State**: @tanstack/react-query for data fetching and caching
- **Styling**: Tailwind CSS for rapid UI development
- **Rendering**: Optimized SVG border rendering with smart edge detection

### Infrastructure
- **Database**: PostgreSQL 14+
- **IPFS**: Kubo (go-ipfs) for distributed storage
- **Blockchain**: Monad (EVM-compatible L1)
- **DevOps**: Docker Compose for local development

## 📂 Project Structure

```
monad-grid-place/
├── backend/
│   ├── src/
│   │   ├── routes/          # API endpoints
│   │   │   ├── snapshot.rs  # /snapshot - Generate IPFS snapshot
│   │   │   ├── paint_area.rs # /paint-area - Record snapshot
│   │   │   ├── grid.rs      # /grid - Fetch pixel data
│   │   │   └── ...
│   │   ├── services/        # Business logic
│   │   │   ├── snapshot_service.rs
│   │   │   ├── ipfs_service.rs
│   │   │   └── grid_service.rs
│   │   └── models/          # Data models
│   ├── migrations/          # SQLx database migrations
│   └── Cargo.toml
│
├── contracts/
│   ├── src/
│   │   └── MonadAdWall.sol  # Main contract with paintArea
│   ├── script/              # Deployment scripts
│   └── test/                # Foundry tests
│
├── indexer/
│   ├── src/
│   │   ├── listener.rs      # Event listener
│   │   ├── storage.rs       # Database sync logic
│   │   └── abi.rs           # Contract ABI bindings
│   └── Cargo.toml
│
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── Grid.tsx     # Canvas rendering with smart borders
│   │   │   ├── PaintModal.tsx # Paint UI with IPFS flow
│   │   │   └── PixelInfo.tsx # Pixel detail display
│   │   ├── services/
│   │   │   └── api.ts       # Backend API client
│   │   └── types/
│   │       └── index.ts     # TypeScript types
│   └── package.json
│
└── ipfs/
    └── docker-compose.yml   # Local IPFS + PostgreSQL
```

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable) + `sqlx-cli`
- **Foundry** (for contract development)
- **Node.js** 18+ + `pnpm`
- **Docker** + Docker Compose
- **PostgreSQL** 14+ (or use Docker)

### 1. Setup Infrastructure

```bash
# Start PostgreSQL and IPFS
docker-compose -f ./ipfs/docker-compose.yml up -d

# Start local blockchain (Anvil)
anvil --host 0.0.0.0 --cors-origins "*"
```

### 2. Deploy Smart Contract

```bash
cd contracts
cp .env.example .env
# Edit .env with your Anvil private key

# Deploy
forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast

# Copy the proxy contract address
```

### 3. Configure Environment

Update `.env` files in `backend/`, `indexer/`, and `frontend/src/components/PaintModal.tsx` with:
- Database URL
- Contract address
- IPFS API URL
- RPC WebSocket URL

### 4. Run Database Migrations

```bash
cd backend
export DATABASE_URL="postgres://postgres:root@localhost:5432/gridplace"
sqlx migrate run
```

### 5. Start Services

Open **three terminals**:

```bash
# Terminal 1: Backend API
cd backend
cargo run
# Runs on http://127.0.0.1:3000

# Terminal 2: Indexer
cd indexer
cargo run
# Listens to blockchain events

# Terminal 3: Frontend
cd frontend
pnpm install
pnpm dev
# Runs on http://localhost:5173
```

### 6. Test the Application

1. Open `http://localhost:5173` in your browser
2. Connect MetaMask (import an Anvil account)
3. **Select pixels**:
   - Click for single selection
   - `Ctrl/Cmd + Click` for multi-select
   - `Shift + Drag` for box selection
4. Enter link, message, and color
5. Click **Paint** and confirm the transaction
6. Watch the logs in backend/indexer terminals
7. After confirmation, the canvas updates automatically

## 🎮 User Guide

### Selection Modes

| Action | Mode | Description |
|--------|------|-------------|
| **Click** | Single | Select one pixel |
| **Ctrl/Cmd + Click** | Multi | Add/remove individual pixels |
| **Shift + Drag** | Box | Select rectangular area |

### Visual Feedback

- **Selected Pixels**: Green border (only outer edges for connected selections)
- **Already Colored**: No border, continuous color display
- **Hover Effect**: Slight opacity change

### Pricing Rules

| Scenario | Cost |
|----------|------|
| Empty pixel (first paint) | 0.01 ETH |
| Your own pixel (update) | **FREE** |
| Others' pixel (acquire) | Current price (10% appreciation) |

## 📊 Database Schema

### `grid_cells` Table
```sql
CREATE TABLE grid_cells (
    id SERIAL PRIMARY KEY,
    x INT NOT NULL,
    y INT NOT NULL,
    owner VARCHAR(42) NOT NULL,
    ipfs_cid VARCHAR(255) NOT NULL,
    color VARCHAR(7) NOT NULL,
    link TEXT,
    message TEXT,
    pixel_index BIGINT GENERATED ALWAYS AS (y * 1000 + x) STORED,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(x, y)
);
```

### `snapshot_history` Table
```sql
CREATE TABLE snapshot_history (
    id SERIAL PRIMARY KEY,
    owner VARCHAR(42) NOT NULL,
    cid VARCHAR(255) NOT NULL UNIQUE,
    cid_hash VARCHAR(66) NOT NULL UNIQUE,
    pixel_count INTEGER NOT NULL,
    total_price VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    tx_hash VARCHAR(66),
    block_number BIGINT
);
```

## 🔐 Security Considerations

- **Reentrancy Protection**: All state-changing functions use `nonReentrant`
- **Access Control**: Owner-only functions for upgrades and withdrawals
- **Input Validation**: Bounds checking for pixel indices
- **IPFS Pinning**: Production should use a pinning service (Pinata, Infura)

## 🌐 Web3 Philosophy

GridPlace embodies core Web3 principles:

1. **Decentralization**: No single point of control or failure
2. **Transparency**: All transactions and ownership are publicly verifiable
3. **Composability**: Open data enables derivatives and integrations
4. **Censorship Resistance**: Art cannot be removed or altered by third parties
5. **User Sovereignty**: True digital ownership through blockchain

## 🚀 Roadmap & Vision

> **⚠️ This is just the beginning!**
>
> GridPlace is currently in early development and runs on a local blockchain for testing. We have ambitious plans to evolve this into a full-featured decentralized application on the **Monad testnet and mainnet**.

### Planned Features

- 🌐 **Testnet/Mainnet Deployment**: Migrate from local Anvil to Monad network
- 🏆 **Leaderboard System**: Rank top artists and collectors with on-chain achievements
- 💎 **Token Rewards**: Earn tokens for creative contributions and community participation
- 💬 **On-Chain Messaging**: Real-time chat and collaboration features for pixel neighbors
- 🎨 **Advanced Tools**: Layer support, templates, and collaborative painting modes
- 📊 **Analytics Dashboard**: Track pixel ownership, trading volume, and community stats
- 🔗 **Social Integration**: Link Twitter, Discord, and other Web3 identities
- 🎁 **NFT Integration**: Mint your pixel creations as NFTs

### Join Us!

We're looking for passionate contributors who share our vision of decentralized creativity:

- **Developers**: Help build features, optimize performance, and improve DX
- **Designers**: Create intuitive UI/UX and stunning visual effects
- **Community Builders**: Grow the GridPlace community and organize events
- **Enthusiasts**: Share ideas, report bugs, and spread the word

**Get in touch:**
- 📧 Email: [atsushinee@outlook.com](mailto:atsushinee@outlook.com)
- 💬 Issues: [Open an issue on GitHub](https://github.com/your-repo/issues)
- 🤝 Contributions: PRs are always welcome!

Come learn and build in Web3 with us. Let's create something amazing together! 🚀

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests (`cargo test`, `pnpm test`, `forge test`)
5. Submit a pull request

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Built on **Monad** for high-performance blockchain
- Inspired by **r/place** and decentralized art movements
- Powered by **IPFS** for distributed storage

---

**GridPlace** - Where blockchain meets creativity. One pixel at a time. 🎨
