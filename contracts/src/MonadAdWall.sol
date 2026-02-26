// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./deps/Dependencies.sol";

/**
 * @title MonadAdWall
 * @author Your Name
 * @notice A gas-optimized, indexer-friendly, and upgradeable pixel ad wall.
 *
 * Architecture:
 * - UUPS Upgradeable Proxy Pattern.
 * - On-chain storage is minimized to a packed struct per pixel.
 * - All descriptive data is stored off-chain on IPFS.
 * - Structured `Painted` events are emitted for indexer consumption.
 */
contract MonadAdWall is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    //================================================================
    // State & Constants
    //================================================================

    uint256 public constant GRID_SIZE = 1000 * 1000;
    uint96 public constant INITIAL_PRICE = 0.01 ether;
    uint256 public constant PRICE_INCREASE_PERCENT = 10;
    uint64 public constant AD_DURATION = 30 days;

    /**
     * @notice Represents a single pixel on the ad wall.
     * The struct is packed to optimize storage, fitting into 3 storage slots.
     */
    struct Pixel {
        // Slot 1: Packed Owner, Expiry, Color (160 + 64 + 32 = 256 bits)
        address owner;
        uint64 expiry;
        uint32 color;
        // Slot 2: The price for the *next* paint.
        uint96 price;
        // Slot 3: keccak256 hash of the IPFS CID.
        bytes32 cidHash;
    }

    mapping(uint256 => Pixel) public pixels;

    //================================================================
    // Events
    //================================================================

    event Painted(
        uint256 indexed index,
        address indexed owner,
        uint32 color,
        uint96 price,
        bytes32 cidHash
    );

    //================================================================
    // Initializer & Upgradeability
    //================================================================

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    function initialize() public initializer {
        __Ownable_init(msg.sender);
        __UUPSUpgradeable_init();
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    //================================================================
    // Core Functions
    //================================================================

    receive() external payable {}

    function paint(uint256 index, uint32 color, bytes32 cidHash) external payable {
        require(index < GRID_SIZE, "MGP: Index out of bounds");

        Pixel storage pixel = pixels[index];
        uint96 currentPrice = pixel.price;

        // If the pixel has never been painted or the ad has expired, its price is the initial price.
        if (currentPrice == 0 || block.timestamp >= pixel.expiry) {
            currentPrice = INITIAL_PRICE;
        }

        require(msg.value >= currentPrice, "MGP: Insufficient payment");

        // Update pixel state
        pixel.owner = msg.sender;
        pixel.color = color;
        pixel.cidHash = cidHash;
        pixel.expiry = uint64(block.timestamp + AD_DURATION);

        // Calculate and set the price for the *next* paint using an unchecked block for gas savings.
        unchecked {
            pixel.price = uint96(currentPrice * (100 + PRICE_INCREASE_PERCENT) / 100);
        }

        emit Painted(index, msg.sender, color, currentPrice, cidHash);

        // Refund any excess Ether sent.
        uint256 excess = msg.value - currentPrice;
        if (excess > 0) {
            (bool sent, ) = msg.sender.call{value: excess}("");
            require(sent, "MGP: Refund failed");
        }
    }

    function getPixel(uint256 index) external view returns (Pixel memory) {
        return pixels[index];
    }

    //================================================================
    // Utility & Admin Functions
    //================================================================

    function multicall(bytes[] calldata data) external payable returns (bytes[] memory results) {
        results = new bytes[](data.length);
        for (uint256 i = 0; i < data.length; i++) {
            (bool success, bytes memory result) = address(this).call(data[i]);
            require(success, "MGP: Multicall sub-call failed");
            results[i] = result;
        }
    }

    function withdraw() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "MGP: No balance to withdraw");
        (bool sent, ) = owner().call{value: balance}("");
        require(sent, "MGP: Withdraw failed");
    }
}
