// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/MonadAdWall.sol";

contract UpgradeScript is Script {
    // 你的 Proxy 地址（永远不变的那个）
    address public constant PROXY_ADDR = 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512;

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        // 1. 部署新的逻辑合约 (Implementation)
        MonadAdWall newImpl = new MonadAdWall();
        console.log("New Implementation deployed at:", address(newImpl));

        // 2. 获取 Proxy 实例并指向新逻辑
        MonadAdWall proxy = MonadAdWall(payable(PROXY_ADDR));

        // 执行升级
        proxy.upgradeToAndCall(address(newImpl), "");

        vm.stopBroadcast();
        console.log("Upgrade complete! Proxy at:", PROXY_ADDR, "now uses new logic.");
    }
}