// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import "./Proxy.sol";
contract ERC1967Proxy is Proxy {
    bytes32 internal constant _IMPLEMENTATION_SLOT = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;
    constructor(address _logic, bytes memory _data) payable {
        assembly { sstore(_IMPLEMENTATION_SLOT, _logic) }
        if (_data.length > 0) {
            (bool success, ) = _logic.delegatecall(_data);
            require(success, "Initialization failed");
        }
    }
    function _getImplementation() internal view virtual override returns (address impl) {
        assembly { impl := sload(_IMPLEMENTATION_SLOT) }
    }
}
