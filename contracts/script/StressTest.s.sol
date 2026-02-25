// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/MonadAdWall.sol";

contract StressTest is Script {
    function run() external {
        uint256 sk = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(sk);

        MonadAdWall wall = MonadAdWall(payable(0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512));

        // 构造 200 个点的数据
        bytes memory data = "";
        for (uint16 i = 0; i < 200; i++) {
            // 每个点占 5 字节: uint16 index + 3 字节 color
            data = abi.encodePacked(data, uint16(i * 10), uint8(0), uint8(255), uint8(0));
        }

        // 发送交易
        wall.compressedBatchPaint{value: 2 ether}(data, "https://monad.xyz");

        vm.stopBroadcast();
        console.log("Stress test transaction sent!");
    }
}