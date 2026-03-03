# GridPlace

**GridPlace** 是一个构建在 Monad 区块链上的完全去中心化的链上像素艺术市场和协作画布。它结合了区块链技术和 IPFS 分布式存储，创建一个无需信任、透明且社区共有的数字艺术平台。

## 🎯 项目愿景

GridPlace 通过去中心化的视角重新构想协作数字艺术的概念：

- **真正的所有权**：每个像素都是你真正拥有的 NFT，不可篡改地记录在链上
- **抗审查**：没有中心化的权威机构可以修改或删除你的艺术作品
- **社区治理**：画布通过去中心化协作不断演进
- **可组合性**：开放的 API 和链上数据支持第三方集成和衍生品

## 🏗️ 架构设计：链上 + 链下混合模型

GridPlace 实现了一个复杂的混合架构，在去中心化、性能和成本之间取得平衡：

### 设计理念

1. **链上建立信任**：关键状态（所有权、来源）存储在链上以确保可验证性
2. **IPFS 实现持久性**：丰富的元数据存储在 IPFS 上，确保数据持久性而不阻塞链
3. **链下提升性能**：索引数据库为前端渲染提供快速查询
4. **事件驱动同步**：索引器监听链上事件并维护链下状态一致性

### 数据流向

```
┌─────────────────────────────────────────────────────────────────┐
│                        用户交互                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         前端                                     │
│  • React + TypeScript                                           │
│  • Wagmi + Viem Web3 交互                                       │
│  • 多选和框选 UI                                                 │
│  • 实时画布渲染                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    后端 API (Rust)                               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 1. 生成快照（合并旧像素 + 新像素）                         │   │
│  │ 2. 上传到 IPFS → 获取 CID                                │   │
│  │    - Pinata: POST /pinning/pinFileToIPFS                │   │
│  │    - 本地 IPFS: POST /api/v0/add                        │   │
│  │ 3. 计算价格（只为新像素付费）                             │   │
│  │ 4. 缓存 CID Hash → CID 映射 (内存 DashMap)               │   │
│  │ 5. 返回 cidHash 和 totalPrice                            │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   智能合约（链上）                                │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ • paintArea(indices, cidHash)                           │   │
│  │ • 验证支付（只为你不拥有的像素付费）                      │   │
│  │ • 更新链上所有权                                          │   │
│  │ • 触发 AreaPainted(owner, cidHash, pixelCount, price)   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    索引器 (Rust)                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ 1. 监听 AreaPainted 事件 (WebSocket)                     │   │
│  │ 2. 从后端获取 CID: GET /cache/{cidHash}                 │   │
│  │ 3. 从 IPFS Gateway 下载快照                              │   │
│  │    - Pinata: GET {gateway}/ipfs/{cid}                   │   │
│  │    - 本地 IPFS: GET {gateway}/ipfs/{cid}                │   │
│  │ 4. 解析像素数据并存储到 PostgreSQL                       │   │
│  │ 5. 记录快照历史                                          │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   PostgreSQL 数据库                              │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ • grid_cells: x, y, owner, color, link, message, ...    │   │
│  │ • snapshot_history: cid, cid_hash, pixel_count, ...     │   │
│  │ • 索引优化快速查询                                        │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  前端（读取路径）                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ • 查询后端 API /grid                                     │   │
│  │ • 使用优化的 SVG 边框渲染画布                              │   │
│  │ • 通过轮询实时更新                                        │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### IPFS 双模式架构

Backend 和 Indexer 都支持两种 IPFS 模式（通过 `.env` 配置）：

| 模式 | Backend | Indexer | 使用场景 |
|------|---------|---------|----------|
| **本地 IPFS** | 上传：`POST /api/v0/add`<br>Gateway: `http://127.0.0.1:8080` | Gateway: `http://127.0.0.1:8080` | 开发环境 |
| **Pinata** | 上传：`POST /pinning/pinFileToIPFS`<br>Gateway: `https://gateway.pinata.cloud` | Gateway: `https://gateway.pinata.cloud` | 生产环境 |

**为什么 Indexer 只需要 Gateway URL：**
- Backend 负责上传快照到 IPFS（需要 API URL）
- Indexer 只从 IPFS 读取快照（只需要 Gateway URL）
- 这种分离减少了耦合，提高了容错性

### 组件职责

| 组件 | IPFS API | IPFS Gateway | 数据库写入 | 数据库读取 | 缓存 |
|------|----------|--------------|------------|------------|------|
| **Backend** | ✅ 上传 | ✅ 读取 | ✅ snapshot_history | ✅ grid_cells | ✅ CID Hash→CID |
| **Indexer** | ❌ | ✅ 读取 | ✅ grid_cells, snapshot_history | ❌ | ❌ |
| **Frontend** | ❌ | ❌ | ❌ | ❌ | ❌ |

## ✨ 核心特性

### 🎨 智能画布
- **1000x1000 像素网格**：一百万个像素，无限创意
- **多选模式**：按住 `Ctrl/Cmd` + 点击选择多个像素
- **框选模式**：按住 `Shift` + 拖拽选择矩形区域
- **智能边框**：相连的选择区域只显示外边框，使用干净的 SVG 渲染
- **已涂色像素可见性**：已上色的像素显示连续颜色，无边框

### 💎 IPFS 快照系统
- **高效存储**：完整的像素元数据（颜色、链接、消息、时间戳）存储在 IPFS 上
- **链上最小化**：只在链上存储 `bytes32 cidHash`
- **快照历史**：每次涂色创建新的 IPFS 快照，支持历史回放
- **默克尔验证**：CID 哈希提供密码学完整性保证

### 💰 公平定价模型
- **只为新像素付费**：更新自己的像素**免费**
- **市场驱动**：获取他人像素按当前市场价（10% 增值）
- **无 Gas 浪费**：批量操作降低每个像素的 Gas 成本
- **自动退款**：超额支付自动退还

### 🔧 开发者体验
- **类型安全**：完整的 TypeScript 严格类型支持
- **模块化架构**：后端、索引器和前端清晰分离
- **热重载**：Vite HMR 快速前端开发
- **SQL 编译时检查**：SQLx 确保查询在编译时正确

## 🛠️ 技术栈

### 智能合约
- **Solidity** `^0.8.20` + Foundry
- **OpenZeppelin Upgradeable**（UUPS 模式）
- **Gas 优化**：结构体打包、惰性初始化
- **安全性**：ReentrancyGuard、Ownable、可升级代理

### 后端（Rust）
- **框架**：Axum（基于 Tokio 的异步 Web 框架）
- **数据库**：SQLx + PostgreSQL（编译时检查查询）
- **IPFS 集成**：双模式支持（本地 IPFS + Pinata）
  - 本地 IPFS: `POST /api/v0/add`, `GET /ipfs/{cid}`
  - Pinata: `POST /pinning/pinFileToIPFS`, `GET {gateway}/ipfs/{cid}`
- **缓存**：DashMap 内存 CID Hash → CID 映射
- **日志**：`env_logger` 通过 `RUST_LOG` 配置日志级别
- **CORS**：完整的跨域支持

### 索引器（Rust）
- **区块链**：Ethers-rs + WebSocket 实时事件
- **事件处理**：AreaPainted 事件监听器
- **IPFS 获取**：双模式 Gateway 支持（本地 + Pinata）
- **数据库同步**：批量插入 + upsert 逻辑
- **日志**：`env_logger` 详细的事件处理步骤日志

### 前端
- **框架**：React 18 + TypeScript + Vite
- **Web3**：Wagmi + Viem 钱包和合约交互
- **状态**：@tanstack/react-query 数据获取和缓存
- **样式**：Tailwind CSS 快速 UI 开发
- **渲染**：优化的 SVG 边框渲染，智能边缘检测

### 基础设施
- **数据库**：PostgreSQL 14+
- **IPFS**：Kubo (go-ipfs) 本地开发，Pinata 生产环境
- **区块链**：Monad（EVM 兼容 L1）
- **DevOps**：Docker Compose 本地开发
- **日志**：集中式日志系统 `log` + `env_logger`

## 📂 项目结构

```
monad-grid-place/
├── backend/
│   ├── src/
│   │   ├── routes/          # API 端点
│   │   │   ├── snapshot.rs  # /snapshot - 生成 IPFS 快照
│   │   │   ├── paint_area.rs # /paint-area - 记录快照
│   │   │   ├── cache.rs     # /cache/:cidHash - Indexer CID 查询
│   │   │   ├── grid.rs      # /grid - 获取像素数据
│   │   │   └── ...
│   │   ├── services/        # 业务逻辑
│   │   │   ├── snapshot_service.rs
│   │   │   ├── ipfs_service.rs    # 双模式 IPFS (本地 + Pinata)
│   │   │   ├── cache_service.rs
│   │   │   └── grid_service.rs
│   │   ├── models/          # 数据模型
│   │   ├── config.rs        # IPFS 双模式配置
│   │   └── main.rs          # 入口点，日志初始化
│   ├── .env.example         # 配置模板（含 USE_PINATA）
│   ├── migrations/          # SQLx 数据库迁移
│   └── Cargo.toml
│
├── contracts/
│   ├── src/
│   │   └── MonadAdWall.sol  # 主合约，包含 paintArea
│   ├── script/              # 部署脚本
│   └── test/                # Foundry 测试
│
├── indexer/
│   ├── src/
│   │   ├── listener.rs      # 事件监听器，详细日志
│   │   ├── storage.rs       # 数据库同步 + IPFS 获取（双模式）
│   │   ├── config.rs        # IPFS 双模式配置
│   │   └── abi.rs           # 合约 ABI 绑定
│   ├── .env.example         # 配置模板（含 USE_PINATA）
│   ├── IPFS_CONFIG.md       # IPFS 配置详细指南
│   └── Cargo.toml
│
├── frontend/
│   ├── src/
│   │   ├── components/
│   │   │   ├── Grid.tsx     # 画布渲染，智能边框
│   │   │   ├── PaintModal.tsx # 涂色 UI，IPFS 流程
│   │   │   └── PixelInfo.tsx # 像素详情展示
│   │   ├── services/
│   │   │   └── api.ts       # 后端 API 客户端
│   │   └── types/
│   │       └── index.ts     # TypeScript 类型定义
│   └── package.json
│
├── ipfs/
│   └── docker-compose.yml   # 本地 IPFS + PostgreSQL
│
└── README.md                # 本文件
```

## 🚀 快速开始

### 环境准备

- **Rust**（最新稳定版）+ `sqlx-cli`
- **Foundry**（合约开发）
- **Node.js** 18+ + `pnpm`
- **Docker** + Docker Compose
- **PostgreSQL** 14+（或使用 Docker）

### 1. 设置基础设施

```bash
# 启动 PostgreSQL 和 IPFS
docker-compose -f ./ipfs/docker-compose.yml up -d

# 启动本地区块链（Anvil）
anvil --host 0.0.0.0 --cors-origins "*"
```

### 2. 部署智能合约

```bash
cd contracts
cp .env.example .env
# 编辑 .env，填入 Anvil 私钥

# 部署
forge script script/Deploy.s.sol --rpc-url http://127.0.0.1:8545 --broadcast

# 复制代理合约地址
```

### 3. 配置环境变量

#### Backend 配置

复制 `backend/` 目录中的 `.env.example` 到 `.env`：

```bash
# 本地开发（本地 IPFS）
USE_PINATA=false
IPFS_API_URL=http://127.0.0.1:5001
IPFS_GATEWAY_URL=http://127.0.0.1:8080

# 生产环境（Pinata）
USE_PINATA=true
PINATA_API_KEY=your_pinata_api_key
PINATA_SECRET_KEY=your_pinata_secret_key
PINATA_GATEWAY_URL=https://gateway.pinata.cloud
```

#### Indexer 配置

复制 `indexer/` 目录中的 `.env.example` 到 `.env`：

```bash
# 必须与 Backend 的 IPFS 模式一致！

# 本地开发
USE_PINATA=false
IPFS_GATEWAY_URL=http://127.0.0.1:8080

# 生产环境（Pinata）
USE_PINATA=true
PINATA_GATEWAY_URL=https://gateway.pinata.cloud
```

#### Frontend 配置

更新 `frontend/src/components/PaintModal.tsx`：
- 合约地址
- 后端 API URL

### 4. 运行数据库迁移

```bash
cd backend
export DATABASE_URL="postgres://postgres:root@localhost:5432/gridplace"
sqlx migrate run
```

### 5. 启动服务

打开**三个终端**：

```bash
# 终端 1：后端 API
cd backend
RUST_LOG=info cargo run
# 运行在 http://127.0.0.1:3000

# 终端 2：索引器
cd indexer
RUST_LOG=info cargo run
# 监听区块链事件

# 终端 3：前端
cd frontend
pnpm install
pnpm dev
# 运行在 http://localhost:5173
```

### 6. 测试应用

1. 在浏览器中打开 `http://localhost:5173`
2. 连接 MetaMask（导入 Anvil 账户）
3. **选择像素**：
   - 单击：单选
   - `Ctrl/Cmd + 单击`：多选
   - `Shift + 拖拽`：框选
4. 输入链接、消息和颜色
5. 点击 **Paint** 并确认交易
6. 观察后端/索引器终端的日志
7. 交易确认后，画布自动更新

### 7. 日志与调试

Backend 和 Indexer 都使用 `env_logger`，可配置日志级别：

```bash
# 显示所有日志（info 及以上）
RUST_LOG=info cargo run

# 显示调试日志
RUST_LOG=debug cargo run

# 只显示错误
RUST_LOG=error cargo run

# 按模块设置日志级别
RUST_LOG=backend::services::ipfs_service=debug,backend=info cargo run
RUST_LOG=indexer::storage=debug,indexer=info cargo run
```

**日志输出示例：**

```
═══════════════════════════════════════════════════════════
🎨 [Backend] Generating snapshot for owner: 0xabc...
   - New pixels count: 10
═══════════════════════════════════════════════════════════
📤 [IPFS] Uploading snapshot to IPFS...
   - Mode: Pinata
   - JSON size: 2048 bytes
📡 [Pinata] Sending request to: https://api.pinata.cloud/...
✅ [IPFS] Upload successful! CID: QmXyz...
💾 [Cache] CID mapping stored:
   - CID Hash: 0x789...
   - CID: QmXyz...
```

## 🎮 用户指南

### 选择模式

| 操作 | 模式 | 说明 |
|------|------|------|
| **单击** | 单选 | 选择一个像素 |
| **Ctrl/Cmd + 单击** | 多选 | 添加/移除单个像素 |
| **Shift + 拖拽** | 框选 | 选择矩形区域 |

### 视觉反馈

- **选中的像素**：绿色边框（相连区域只显示外边缘）
- **已涂色像素**：无边框，连续颜色显示
- **悬停效果**：轻微透明度变化

### 定价规则

| 场景 | 费用 |
|------|------|
| 空像素（首次涂色） | 0.01 ETH |
| 自己的像素（更新） | **免费** |
| 他人的像素（获取） | 当前价格（10% 增值） |

## 📊 数据库 Schema

### `grid_cells` 表
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

### `snapshot_history` 表
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

## 🔐 安全考虑

- **重入保护**：所有状态变更函数使用 `nonReentrant`
- **访问控制**：升级和提款仅限 Owner
- **输入验证**：像素索引边界检查
- **IPFS 固定**：生产环境应使用固定服务（Pinata、Infura）

## 🌐 Web3 理念

GridPlace 体现了核心的 Web3 原则：

1. **去中心化**：没有单一控制点或故障点
2. **透明度**：所有交易和所有权公开可验证
3. **可组合性**：开放数据支持衍生品和集成
4. **抗审查**：艺术作品不能被第三方删除或篡改
5. **用户主权**：通过区块链实现真正的数字所有权

## 🚀 路线图与愿景

> **⚠️ 这只是开始！**
>
> GridPlace 目前处于早期开发阶段，运行在本地区块链上进行测试。我们有雄心勃勃的计划，将其发展成为一个在 **Monad 测试网和主网**上运行的全功能去中心化应用。

### 计划特性

- 🌐 **测试网/主网部署**：从本地 Anvil 迁移到 Monad 网络
- 🏆 **排行榜系统**：排名顶级艺术家和收藏家，链上成就系统
- 💎 **代币奖励**：为创意贡献和社区参与赚取代币
- 💬 **链上通讯**：像素邻居之间的实时聊天和协作功能
- 🎨 **高级工具**：图层支持、模板和协作绘画模式
- 📊 **数据分析仪表板**：追踪像素所有权、交易量和社区统计
- 🔗 **社交集成**：绑定 Twitter、Discord 和其他 Web3 身份
- 🎁 **NFT 集成**：将你的像素创作铸造为 NFT

### 加入我们！

我们正在寻找与我们共同愿景的热情贡献者：

- **开发者**：帮助构建功能、优化性能、改善开发者体验
- **设计师**：创造直观的 UI/UX 和精美的视觉效果
- **社区建设者**：发展 GridPlace 社区并组织活动
- **爱好者**：分享想法、报告问题、传播口碑

**联系方式：**
- 📧 邮箱：[atsushinee@outlook.com](mailto:atsushinee@outlook.com)
- 💬 Issues：[在 GitHub 上提交 issue](https://github.com/your-repo/issues)
- 🤝 贡献：欢迎提交 PR！

来和我们一起在 Web3 中学习和构建吧！让我们共同创造惊人的作品！🚀

## 🤝 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 仓库
2. 创建功能分支
3. 进行修改
4. 运行测试（`cargo test`、`pnpm test`、`forge test`）
5. 提交 Pull Request

## 📄 许可证

MIT 许可证 - 详见 [LICENSE](LICENSE)

## 🙏 致谢

- 构建在 **Monad** 高性能区块链上
- 灵感来自 **r/place** 和去中心化艺术运动
- 由 **IPFS** 提供分布式存储支持

---

**GridPlace** - 区块链与创意的交汇。一次一个像素。🎨
