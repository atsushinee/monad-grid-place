# GridPlace

GridPlace 是一个全栈、生产级的 Web3 像素广告墙项目，专为 Monad 测试网等高吞吐量环境设计。它可作为构建健壮、可扩展且易于维护的去中心化应用的参考架构，其核心是一个端到端的、事件驱动的数据管道。

## 架构设计：工业级数据流

项目遵循严格的解耦架构，以确保高性能、低 Gas 成本和清晰的职责分离。数据流是单向的，旨在提供响应迅速的用户体验，同时保持链上数据的完整性。

**核心原则**：区块链被视为一个“只写”的命令总线。前端只从快速、已索引的链下数据库中读取数据，从不直接查询链。

**数据流转过程：**

```
1. 前端 (涂色操作)
   - POST /upload -> 后端 API (上传元数据至 IPFS, 返回 CID 和哈希)
   - POST /cache  -> 后端 API (为 Indexer 缓存 CID/哈希映射)
   - writeContract('paint') -> 区块链 (仅发送必要数据上链)
     |
     v
2. 区块链 (Monad)
   - 执行 `paint` 交易.
   - 发出 `Painted` 事件 (仅包含 `cidHash` 等索引数据).
     |
     v
3. Indexer (索引器, Rust)
   - 通过 WebSocket 监听 `Painted` 事件.
   - GET /cache/:cidHash -> 后端 API (用哈希换回原始 CID).
   - 将包含原始 CID 的完整像素数据写入 PostgreSQL 数据库.
     |
     v
4. 后端 API (Rust)
   - GET /grid -> 前端 (为前端提供像素数据).
   - 从 PostgreSQL 读取数据.
   - 使用存储的 CID 从 IPFS 获取完整的元数据.
   - 将丰富后的数据返回给前端.
     |
     v
5. 前端 (视图)
   - 基于从后端 API 获取的数据，渲染像素网格.
```

## 核心特性

- **工业级数据管道**: 实现了一个精巧的前端、后端和索引器之间的“缓存-查询”模式，以保持链上交易的极简和高速。
- **索引器优先设计**: 应用状态通过事件溯源（Event Sourcing）构建。前端是索引数据库状态的纯粹展示。
- **链上/链下分离**: 仅在链上存储 IPFS CID 的 `bytes32` 哈希，最大限度地减少了链上足迹。所有丰富的元数据（链接、描述等）都存放在 IPFS 上，由后端获取。
- **Gas 优化与可升级合约**: 利用了结构体打包（Struct Packing）和 UUPS 代理模式。
- **高性能前端**:
  - 基于 **Vite + React + TypeScript** 构建。
  - 使用 **Wagmi + Viem** 提供顶级的钱包和区块链交互体验。
  - **缩放与拖拽**: 实现了画布式的导航，提供流畅的用户体验。
  - **虚拟化就绪**: 网格渲染的架构已为支持虚拟化渲染做好准备，确保在百万级像素下仍能保持性能。
- **健壮的后端服务**: API 和索引器均使用 Rust 构建，以保证性能和安全性，具有异步操作、清晰的服务分离和通过 `sqlx` 实现的编译时 SQL 查询检查等特性。

## 技术栈

- **智能合约**:
  - Solidity `^0.8.20`, Foundry (Forge, Anvil, Cast)
  - OpenZeppelin Upgradeable Contracts (UUPS)
- **后端 API**:
  - Rust, Tokio, Axum, SQLx (PostgreSQL)
  - `DashMap` 用于内存缓存。
- **索引器 (Indexer)**:
  - Rust, Tokio, Ethers-rs (WebSocket), SQLx
  - `Reqwest` 用于向后端 API 进行缓存查询。
- **前端**:
  - Vite, React, TypeScript, Tailwind CSS
  - Wagmi & Viem 用于 Web3 状态管理。
  - `@tanstack/react-query` 用于数据请求。
- **数据库**: PostgreSQL
- **DevOps**: Docker & Docker Compose, `.env` 配置管理。

## 项目结构

```
.
├── backend/         # Rust (Axum) API: 负责 IPFS 上传、缓存和数据服务
├── contracts/       # Solidity 智能合约 (Foundry)
├── frontend/        # React DApp (Vite, Wagmi, Tailwind)
├── indexer/         # Rust 独立事件索引器
├── ipfs/            # 用于本地 IPFS 和 PostgreSQL 的 Docker-compose
└── README.md
```

## 快速开始

### 环境准备

- [Rust](https://www.rust-lang.org/tools/install) & `sqlx-cli`
- [Foundry](https://getfoundry.sh/)
- [Docker](https://www.docker.com/get-started)
- [Node.js](https://nodejs.org/) & `pnpm` (或 `npm`/`yarn`)

### 1. 运行基础设施

启动 PostgreSQL、IPFS 和本地测试网节点。

```sh
# 启动数据库和 IPFS
docker-compose -f ./ipfs/docker-compose.yml up -d

# 启动启用了 CORS 的区块链节点
anvil --host 0.0.0.0 --cors-origins "*"
```

### 2. 部署合约

在**新的终端**中，将 Anvil 输出的私钥设置到 `contracts/.env` 文件中，然后部署。

```sh
cd contracts
cp .env.example .env
# 编辑 .env 文件，填入 Anvil 提供的私钥
forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast
```
**复制新部署的代理合约地址。**

### 3. 配置并运行服务

使用新的部署信息，更新 `backend`、`indexer` 目录下的 `.env` 文件，以及 `frontend/src/components/PaintModal.tsx` 中的合约地址。

运行数据库迁移：
```sh
# 在 /backend 目录下
export DATABASE_URL="postgres://postgres:root@localhost/gridplace" # 或你自己的 URL
sqlx migrate run
```

在各自的终端中运行每个服务：
```sh
# 终端 1: Backend
cd backend
cargo run

# 终端 2: Indexer
cd indexer
cargo run

# 终端 3: Frontend
cd frontend
pnpm install
pnpm dev
```

### 4. 测试完整流程
1. 在浏览器中打开前端 URL。
2. 连接你的 MetaMask 钱包（确保网络是 Anvil，并已导入一个有资金的账户）。
3. 点击一个像素，输入链接，选择颜色，然后点击 "Paint"。
4. 观察 `backend` 和 `indexer` 的终端日志，查看“缓存-查询”流程是否按预期工作。
5. 交易确认后，网格会自动更新。再次点击你涂过的像素，应该能看到你输入的链接。
