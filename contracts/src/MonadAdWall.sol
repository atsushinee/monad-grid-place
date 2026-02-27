// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./deps/Dependencies.sol";

/**
 * @title MonadAdWall V7 - 链上实时像素竞价对战
 * @notice 这是一个"广告竞价 + 实时抢占"的链上像素领地战游戏
 *
 * 核心规则：
 * 1. 网格大小：1000 x 1000 = 1,000,000 个像素
 * 2. 玩家调用 paint() 占领像素，支付当前价格
 * 3. 像素价格每次被占领后上涨 10%
 * 4. 玩家 3 秒内只能操作一次（防 spam）
 * 5. 像素被占领后 30 秒内不可被抢占（防抢跑）
 * 6. 单次最多占领 200 个像素
 *
 * @dev 使用 ReentrancyGuard 防止重入攻击
 */
contract MonadAdWall is Initializable, OwnableUpgradeable, UUPSUpgradeable, ReentrancyGuard {
    uint256 public constant GRID_SIZE = 1000 * 1000; // 总像素数：1,000,000
    uint256 public constant MAX_PIXELS_PER_TX = 200; // 单次最多占领像素数
    uint256 public constant PRICE_MULTIPLIER = 110; // 价格乘数：110%
    uint256 public constant PRICE_DIVISOR = 100; // 价格除数
    uint256 public constant PLAYER_COOLDOWN = 3 seconds; // 玩家冷却时间：3 秒
    uint256 public constant PIXEL_COOLDOWN = 30 seconds; // 像素冷却时间：30 秒
    uint256 public constant BASE_PRICE = 0.01 ether; // 基础价格

    /// @notice 像素所有者 mapping(pixelIndex => owner)
    mapping(uint256 => address) public pixelOwner;

    /// @notice 像素当前价格 mapping(pixelIndex => price)
    mapping(uint256 => uint256) public pixelPrice;

    /// @notice 像素最后更新时间 mapping(pixelIndex => timestamp)
    mapping(uint256 => uint256) public pixelLastUpdate;

    /// @notice 玩家最后操作时间 mapping(player => timestamp)
    mapping(address => uint256) public playerLastMove;

    /// @notice 玩家占领像素数量 mapping(player => count)
    mapping(address => uint256) public playerPixelCount;

    /// @notice 合约总收益
    uint256 public totalRevenue;

    /// @notice 像素更新事件
    /// @param player 玩家地址
    /// @param indices 被更新的像素索引数组
    event PixelsUpdated(address indexed player, uint256[] indices);

    /// @notice 像素被占领事件
    /// @param pixelIndex 像素索引
    /// @param previousOwner 前所有者
    /// @param newOwner 新所有者
    /// @param price 成交价格
    event PixelCaptured(
        uint256 indexed pixelIndex,
        address indexed previousOwner,
        address indexed newOwner,
        uint256 price
    );

    constructor() { _disableInitializers(); }

    /**
     * @notice 初始化合约
     * @dev 不再循环初始化所有像素价格，改为惰性初始化（首次访问时设置）
     */
    function initialize() public initializer {
        __Ownable_init(msg.sender);
        __UUPSUpgradeable_init();
        __ReentrancyGuard_init();
        // 不再初始化所有像素价格，改为在 paint 时惰性初始化
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    receive() external payable {}

    /**
     * @notice 占领像素（竞价对战核心函数）
     * @dev 支付当前价格，更新像素所有权和价格
     * @param indices 像素索引数组 (index = y * 1000 + x)
     */
    function paint(uint256[] calldata indices) external payable nonReentrant {
        // 检查：至少占领 1 个像素
        require(indices.length > 0, "MGP: Must paint at least one pixel");

        // 检查：不超过最大像素数
        require(indices.length <= MAX_PIXELS_PER_TX, "MGP: Too many pixels");

        // 检查：玩家冷却时间（3 秒内只能操作一次）
        require(
            block.timestamp >= playerLastMove[msg.sender] + PLAYER_COOLDOWN,
            "MGP: Player cooldown"
        );

        // 计算总价格并验证每个像素
        uint256 totalPrice = 0;
        for (uint256 i = 0; i < indices.length; i++) {
            uint256 index = indices[i];

            // 检查：索引边界
            require(index < GRID_SIZE, "MGP: Index out of bounds");

            // 惰性初始化：如果价格为 0，设置为基础价格
            if (pixelPrice[index] == 0) {
                pixelPrice[index] = BASE_PRICE;
            }

            // 检查：像素冷却时间（30 秒内不可被抢占）
            require(
                block.timestamp >= pixelLastUpdate[index] + PIXEL_COOLDOWN,
                "MGP: Pixel cooldown"
            );

            totalPrice += pixelPrice[index];
        }

        // 检查：支付金额足够
        require(msg.value >= totalPrice, "MGP: Insufficient payment");

        // 更新玩家最后操作时间
        playerLastMove[msg.sender] = block.timestamp;

        // 执行占领
        uint256 newCaptures = 0;
        for (uint256 i = 0; i < indices.length; i++) {
            uint256 index = indices[i];
            address previousOwner = pixelOwner[index];

            // 如果是首次占领或从其他玩家手中夺取，计数
            if (previousOwner != msg.sender) {
                if (previousOwner != address(0)) {
                    // 从其他玩家手中夺取，减少对方的计数
                    playerPixelCount[previousOwner]--;
                }
                newCaptures++;
            }

            // 更新像素所有者
            pixelOwner[index] = msg.sender;

            // 更新像素价格：oldPrice * 110% (向上取整)
            uint256 oldPrice = pixelPrice[index];
            uint256 newPrice = (oldPrice * PRICE_MULTIPLIER + PRICE_DIVISOR - 1) / PRICE_DIVISOR;
            pixelPrice[index] = newPrice;

            // 更新像素最后更新时间
            pixelLastUpdate[index] = block.timestamp;

            // 触发像素被占领事件
            emit PixelCaptured(index, previousOwner, msg.sender, oldPrice);
        }

        // 更新玩家占领像素数量
        playerPixelCount[msg.sender] += newCaptures;

        // 更新合约总收益
        totalRevenue += totalPrice;

        // 触发像素更新事件
        emit PixelsUpdated(msg.sender, indices);

        // 退款：退回超额支付的部分
        if (msg.value > totalPrice) {
            payable(msg.sender).transfer(msg.value - totalPrice);
        }
    }

    /**
     * @notice 涂色区域 - 链上所有权 + IPFS 状态快照
     * @dev 遍历 indices 更新像素所有权，记录 owner 快照 CID
     *      只对新占领的像素收费，自己的像素免费更新
     * @param indices 像素索引数组，index = y * 1000 + x
     * @param cidHash IPFS 快照 CID 的哈希
     */
    function paintArea(uint256[] calldata indices, bytes32 cidHash) external payable nonReentrant {
        require(indices.length > 0, "MGP: Must paint at least one pixel");

        // 计算需要支付的价格（只计算从他人手中夺取的像素）
        uint256 totalPrice = 0;
        for (uint256 i = 0; i < indices.length; i++) {
            uint256 index = indices[i];
            require(index < GRID_SIZE, "MGP: Index out of bounds");
            
            // 如果不是自己的像素，需要支付当前价格
            if (pixelOwner[index] != msg.sender) {
                uint256 price = pixelPrice[index];
                if (price == 0) {
                    price = BASE_PRICE; // 首次涂色
                }
                totalPrice += price;
            }
        }

        // 验证支付金额
        require(msg.value >= totalPrice, "MGP: Insufficient payment");

        // 遍历并更新每个像素的所有权
        for (uint256 i = 0; i < indices.length; i++) {
            uint256 index = indices[i];
            address previousOwner = pixelOwner[index];
            
            // 更新像素所有者
            pixelOwner[index] = msg.sender;

            // 如果是从他人手中夺取，更新价格和计数
            if (previousOwner != msg.sender && previousOwner != address(0)) {
                // 减少前所有者的计数
                playerPixelCount[previousOwner]--;
                
                // 更新像素价格：oldPrice * 110% (向上取整)
                uint256 oldPrice = pixelPrice[index];
                if (oldPrice == 0) {
                    oldPrice = BASE_PRICE;
                }
                uint256 newPrice = (oldPrice * PRICE_MULTIPLIER + PRICE_DIVISOR - 1) / PRICE_DIVISOR;
                pixelPrice[index] = newPrice;
                
                // 增加新所有者的计数
                playerPixelCount[msg.sender]++;
            } else if (previousOwner == address(0)) {
                // 首次涂色
                pixelPrice[index] = BASE_PRICE;
                playerPixelCount[msg.sender]++;
            }
            // 如果是自己的像素，价格不变，免费更新
        }

        // 更新 owner 的最新快照 CID
        ownerSnapshots[msg.sender] = cidHash;

        // 更新合约总收益
        totalRevenue += totalPrice;

        // 触发事件
        emit AreaPainted(msg.sender, cidHash, indices.length, totalPrice);

        // 退款：退回超额支付的部分
        if (msg.value > totalPrice) {
            payable(msg.sender).transfer(msg.value - totalPrice);
        }
    }

    /// @notice 保留原有的 ownerSnapshots 映射（兼容 V6）
    mapping(address => bytes32) public ownerSnapshots;

    /// @notice 保留原有的 AreaPainted 事件（兼容 V6）
    event AreaPainted(
        address indexed owner,
        bytes32 indexed cidHash,
        uint256 pixelCount,
        uint256 totalPrice
    );

    /**
     * @notice 批量查询像素所有者
     * @param indices 像素索引数组
     * @return owners 所有者地址数组
     */
    function getPixelOwners(uint256[] calldata indices) external view returns (address[] memory owners) {
        owners = new address[](indices.length);
        for (uint256 i = 0; i < indices.length; i++) {
            owners[i] = pixelOwner[indices[i]];
        }
    }

    /**
     * @notice 批量查询像素价格
     * @param indices 像素索引数组
     * @return prices 价格数组
     */
    function getPixelPrices(uint256[] calldata indices) external view returns (uint256[] memory prices) {
        prices = new uint256[](indices.length);
        for (uint256 i = 0; i < indices.length; i++) {
            // 如果价格为 0，返回基础价格（惰性初始化）
            uint256 price = pixelPrice[indices[i]];
            prices[i] = (price == 0) ? BASE_PRICE : price;
        }
    }

    /**
     * @notice 批量查询像素冷却时间
     * @param indices 像素索引数组
     * @return cooldowns 剩余冷却时间数组（秒）
     */
    function getPixelCooldowns(uint256[] calldata indices) external view returns (uint256[] memory cooldowns) {
        cooldowns = new uint256[](indices.length);
        for (uint256 i = 0; i < indices.length; i++) {
            uint256 endTime = pixelLastUpdate[indices[i]] + PIXEL_COOLDOWN;
            if (block.timestamp >= endTime) {
                cooldowns[i] = 0;
            } else {
                cooldowns[i] = endTime - block.timestamp;
            }
        }
    }

    /**
     * @notice 查询玩家冷却时间
     * @param player 玩家地址
     * @return 剩余冷却时间（秒）
     */
    function getPlayerCooldown(address player) external view returns (uint256) {
        uint256 endTime = playerLastMove[player] + PLAYER_COOLDOWN;
        if (block.timestamp >= endTime) {
            return 0;
        }
        return endTime - block.timestamp;
    }

    /**
     * @notice 计算占领指定像素所需的总价格
     * @param indices 像素索引数组
     * @return totalPrice 总价格
     */
    function calculateTotalPrice(uint256[] calldata indices) external view returns (uint256 totalPrice) {
        for (uint256 i = 0; i < indices.length; i++) {
            // 如果价格为 0，使用基础价格（惰性初始化）
            uint256 price = pixelPrice[indices[i]];
            totalPrice += (price == 0) ? BASE_PRICE : price;
        }
    }

    /**
     * @notice 提取合约中的 ETH（只有 Owner 可以）
     */
    function withdraw() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "MGP: No balance to withdraw");
        (bool sent, ) = owner().call{value: balance}("");
        require(sent, "MGP: Withdraw failed");
    }

    /**
     * @notice 紧急提款（带金额限制）
     * @param amount 提款金额
     */
    function withdrawPartial(uint256 amount) external onlyOwner {
        require(amount > 0, "MGP: Amount must be > 0");
        require(address(this).balance >= amount, "MGP: Insufficient balance");
        (bool sent, ) = owner().call{value: amount}("");
        require(sent, "MGP: Withdraw failed");
    }
}
