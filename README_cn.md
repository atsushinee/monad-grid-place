# GridPlace

GridPlace 是一个全栈、生产级的 Web3 像素广告墙项目，专为 Monad 测试网等高吞吐量环境设计。它可作为构建健壮、可扩展且易于维护的去中心化应用的参考架构。

## 架构概览

项目遵循清晰、解耦的架构原则，每个组件都有单一的职责。数据从区块链单向流向前端，确保了数据的完整性和一致性。

```
[ 链上: Monad ] <=> [ Indexer (Rust) ] <=> [ 数据库 (PostgreSQL) ] <=> [ 后端 API (Rust) ] <=> [ 前端 (React) ]
       ^                                                                                                  ^
       |                                                                                                  |
[ 用户钱包 ]                                                                                           [ IPFS (元数据) ]
```

- **智能合约**: 仅存储必要数据（哈希和所有权）并发出事件。
- **Indexer (索引器)**: 一个独立的 Rust 服务，通过 WebSocket 监听合约事件，解码事件数据，并将其写入 PostgreSQL 数据库。这是链下状态的唯一真实来源。
- **后端 API**: 一个无状态的 Rust (Axum) 服务，它从数据库中读取数据以服务于前端，同时处理 IPFS 上传。
- **前端**: 一个 React 应用，与用户钱包交互，并且只从后端 API 获取数据。

## 核心特性

- **模块化与可扩展**: 每个服务（后端、索引器）都是独立的，可以单独进行扩展。
- **索引器优先设计**: 应用状态通过事件溯源（Event Sourcing）模式构建，提供了极佳的健壮性和可回溯性。前端永远不直接查询区块链。
- **链上/链下分离**: 通过仅在链上存储 IPFS CID 的 `bytes32` 哈希，最大限度地减少了链上足迹和 Gas 成本。所有丰富的元数据都存放在 IPFS 上。
- **Gas 优化的合约**: 利用结构体打包（Struct Packing）、`unchecked` 数学运算和高效的价格模型来降低交易成本。
- **可升级与安全**: 实现了 UUPS 代理模式以支持合约升级，并遵循社区安全最佳实践。
- **开发者体验**: 利用 Foundry 进行合约开发，使用 `sqlx-cli` 进行编译时 SQL 查询检查，并提供 Docker 化的开发环境。

## 技术栈

- **智能合约**:
  - Solidity `^0.8.20`
  - Foundry (Forge, Anvil, Cast)
  - OpenZeppelin Upgradeable Contracts (UUPS)
- **后端 API**:
  - Rust, Tokio
  - Axum (Web 框架)
  - SQLx (PostgreSQL)
  - Reqwest (用于 IPFS 的 HTTP 客户端)
- **索引器 (Indexer)**:
  - Rust, Tokio
  - Ethers-rs (WebSocket 事件订阅)
  - SQLx (PostgreSQL)
- **数据库**:
  - PostgreSQL
- **DevOps**:
  - Docker & Docker Compose
  - `.env` 文件进行配置管理

## 项目结构

```
.
├── backend/         # Rust (Axum) RESTful API 服务
├── contracts/       # Solidity 智能合约 (Foundry)
├── frontend/        # React 前端 (占位)
├── indexer/         # Rust 独立事件索引器
├── ipfs/            # 用于本地 IPFS 节点的 Docker-compose
└── README.md
```

## 快速开始

### 环境准备

- [Rust](https://www.rust-lang.org/tools/install)
- [Foundry](https://getfoundry.sh/)
- [Docker](https://www.docker.com/get-started)
- [`sqlx-cli`](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli): `cargo install sqlx-cli --no-default-features --features rustls,postgres`

### 1. 运行基础设施

启动 PostgreSQL 数据库和本地 IPFS 节点。

```sh
docker-compose -f ./ipfs/docker-compose.yml up -d
```

### 2. 部署合约

首先，启动一个本地测试网节点。

```sh
anvil
```

在**新的终端**中，将 Anvil 输出的私钥设置到 `contracts/.env` 文件中，然后部署合约。

```sh
cd contracts
cp .env.example .env
# 编辑 .env 文件，填入 Anvil 提供的私钥
forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast
```
复制部署成功后输出的代理合约地址。

### 3. 配置并运行服务

`backend` 和 `indexer` 服务都需要一个 `.env` 文件。

```sh
# 在 /backend 目录下
cp .env.example .env
# 编辑 .env 文件，填入你的数据库和 IPFS 设置

# 在 /indexer 目录下
cp .env.example .env
# 编辑 .env 文件，填入你的 RPC、数据库和刚刚部署的合约地址
```

使用 `sqlx-cli` 运行数据库迁移。

```sh
# 在 /backend 目录下
export DATABASE_URL="postgres://postgres:root@localhost/gridplace" # 或你自己的 URL
sqlx migrate run
```

最后，在各自的终端中运行服务。

```sh
# 终端 1: Indexer
cd indexer
cargo run

# 终端 2: Backend
cd backend
cargo run
```

### 4. 测试流程

使用 `cast` 触发一个 `paint` 事件。

```sh
cast send 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 \
"paint(uint256,uint32,bytes32)" 123 0xFF00FF \
0x1234567890123456789012345678901234567890123456789012345678901234 \
--value 0.01ether \
--private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
--rpc-url http://127.0.0.1:8545
```

你应该能在 Indexer 的终端看到事件被处理的日志。然后，你可以通过 Backend API 查询数据：`curl http://127.0.0.1:3000/grid/123/0`。
