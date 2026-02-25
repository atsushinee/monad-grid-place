// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
abstract contract Proxy {
    function _delegate(address implementation) internal virtual {
        assembly {
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), implementation, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch result case 0 { revert(0, returndatasize()) } default { return(0, returndatasize()) }
        }
    }
    fallback() external payable virtual { _delegate(_getImplementation()); }
    receive() external payable virtual { _delegate(_getImplementation()); }
    function _getImplementation() internal view virtual returns (address);
}
