// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./deps/Dependencies.sol";

/**
 * @title MonadAdWall
 * @dev 针对 Monad 优化的并行友好型像素墙
 * 增加了 multicall 以支持单次签名大面积涂色，并优化了事件索引。
 */
contract MonadAdWall is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    struct Pixel {
        address owner;
        uint32 color;
        uint256 price;
        uint256 expiry;
        string link;
    }

    uint256 public constant CANVAS_SIZE = 100;
    uint256 public constant MIN_PRICE = 0.01 ether;
    uint256 public constant DURATION = 1 days;

    mapping(uint256 => Pixel) public canvas;
    mapping(address => uint256) public pendingWithdrawals;

    // 优化：给 index 增加 indexed，方便 Go 后端高效过滤特定坐标的事件
    event Painted(uint256 indexed index, address indexed owner, uint32 color, string link, uint256 price);
    event Withdrawn(address indexed user, uint256 amount);

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() { _disableInitializers(); }

    function initialize() public initializer {
        __Ownable_init(msg.sender);
        __UUPSUpgradeable_init();
    }

    /**
     * @dev 批量涂色逻辑
     */
    function compressedBatchPaint(bytes calldata data, string calldata link) external payable {
        uint256 pixelCount = data.length / 5;
        require(pixelCount > 0, "No data");

        uint256 costPerPixel = msg.value / pixelCount;
        require(costPerPixel >= MIN_PRICE, "Price low");

        for (uint256 i = 0; i < pixelCount; ) {
            {
                uint256 offset = i * 5;
                uint16 idx = uint16(bytes2(data[offset : offset + 2]));

                if (idx < 10000) {
                    Pixel storage pixel = canvas[idx];

                    if (block.timestamp > pixel.expiry || costPerPixel >= (pixel.price * 110) / 100) {
                        address oldOwner = pixel.owner;
                        if (oldOwner != address(0)) {
                            pendingWithdrawals[oldOwner] += pixel.price;
                        }

                        pixel.owner = msg.sender;
                        pixel.price = costPerPixel;
                        pixel.expiry = block.timestamp + DURATION;
                        pixel.link = link;

                        uint32 newColor = (uint32(uint8(data[offset + 2])) << 16) |
                            (uint32(uint8(data[offset + 3])) << 8) |
                            (uint32(uint8(data[offset + 4])));

                        pixel.color = newColor;

                        emit Painted(idx, msg.sender, newColor, link, costPerPixel);
                    }
                }
            }
            unchecked { i++; }
        }
    }

    /**
     * @dev 新增：Multicall 聚合调用
     * 允许前端将 2500 个点拆分为多个 100 点的 compressedBatchPaint 并在一个交易中执行。
     * 这是解决 Gas 报错和提升用户体验的关键。
     */
    function multicall(bytes[] calldata data) external payable returns (bytes[] memory results) {
        results = new bytes[](data.length);
        for (uint256 i = 0; i < data.length; i++) {
            // 使用 delegatecall 保持 msg.sender 和存储上下文
            (bool success, bytes memory result) = address(this).delegatecall(data[i]);

            if (!success) {
                // 抛出底层原始错误信息
                if (result.length > 0) {
                    assembly {
                        let returndata_size := mload(result)
                        revert(add(32, result), returndata_size)
                    }
                } else {
                    revert("Multicall: call failed");
                }
            }
            results[i] = result;
        }
    }

    function paint(uint256 x, uint256 y, uint32 color, string memory link) public payable {
        require(x < CANVAS_SIZE && y < CANVAS_SIZE, "Outside bounds");
        uint256 index = x * CANVAS_SIZE + y;
        Pixel storage pixel = canvas[index];

        require(msg.value >= MIN_PRICE, "Insufficient payment");

        uint256 currentPrice = pixel.price;
        require(block.timestamp > pixel.expiry || msg.value >= (currentPrice * 110) / 100, "Must pay 10% more");

        address oldOwner = pixel.owner;
        if (oldOwner != address(0)) {
            pendingWithdrawals[oldOwner] += currentPrice;
        }

        canvas[index] = Pixel({
            owner: msg.sender,
            color: color,
            price: msg.value,
            expiry: block.timestamp + DURATION,
            link: link
        });

        emit Painted(index, msg.sender, color, link, msg.value);
    }

    function withdraw() public {
        uint256 amount = pendingWithdrawals[msg.sender];
        require(amount > 0, "Nothing to withdraw");
        pendingWithdrawals[msg.sender] = 0;
        (bool success, ) = payable(msg.sender).call{value: amount}("");
        require(success, "Transfer failed");
        emit Withdrawn(msg.sender, amount);
    }

    function getPixel(uint256 index) public view returns (Pixel memory) {
        return canvas[index];
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    // 升级保护槽
    uint256[50] private __gap;
}