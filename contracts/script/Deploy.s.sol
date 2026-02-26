// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/MonadAdWall.sol";
import "../src/ERC1967Proxy.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        // 显式计算部署者地址
        address deployer = vm.addr(deployerPrivateKey);

        vm.startBroadcast(deployerPrivateKey);

        // 1. 部署逻辑合约
        MonadAdWall logic = new MonadAdWall();
        console.log("Implementation deployed at:", address(logic));

        // 2. 编码初始化数据
        bytes memory initData = abi.encodeWithSelector(logic.initialize.selector);

        // 3. 部署代理合约
        ERC1967Proxy proxy = new ERC1967Proxy(address(logic), initData);

        address proxyAddress = address(proxy);
        console.log("Proxy deployed at:", proxyAddress);

        // 4. 验证初始化是否成功 (在脚本中直接检查)
        // 使用 payable() 进行显式类型转换
        address currentOwner = MonadAdWall(payable(proxyAddress)).owner();
        console.log("Proxy Owner is:", currentOwner);

        vm.stopBroadcast();
    }
}
