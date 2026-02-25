// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

abstract contract Initializable {
    uint64 private _initialized;
    bool private _initializing;
    event Initialized(uint64 version);

    modifier initializer() {
        require(_initializing || _initialized < 1, "Initializable: is already initialized");
        bool isTopLevelCall = !_initializing;
        if (isTopLevelCall) { _initializing = true; _initialized = 1; }
        _;
        if (isTopLevelCall) { _initializing = false; emit Initialized(1); }
    }

    // 关键修正：把修饰符放在基类，避免子类冲突
    modifier onlyInitializing() {
        require(_initializing, "Initializable: is not initializing");
        _;
    }

    function _disableInitializers() internal virtual {
        require(!_initializing, "Initializable: is initializing");
        if (_initialized < type(uint64).max) {
            _initialized = type(uint64).max;
            emit Initialized(type(uint64).max);
        }
    }
}

abstract contract ContextUpgradeable is Initializable {
    function _msgSender() internal view virtual returns (address) { return msg.sender; }
}

abstract contract OwnableUpgradeable is ContextUpgradeable {
    address private _owner;
    error OwnableUnauthorizedAccount(address account);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    function __Ownable_init(address initialOwner) internal onlyInitializing {
        _transferOwnership(initialOwner);
    }

    modifier onlyOwner() {
        if (owner() != _msgSender()) revert OwnableUnauthorizedAccount(_msgSender());
        _;
    }

    function owner() public view virtual returns (address) { return _owner; }

    function _transferOwnership(address newOwner) internal virtual {
        address oldOwner = _owner;
        _owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
}

abstract contract UUPSUpgradeable is Initializable {
    bytes32 private constant _IMPLEMENTATION_SLOT = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;

    function __UUPSUpgradeable_init() internal onlyInitializing {}

    function upgradeToAndCall(address newImplementation, bytes memory data) public payable virtual {
        _authorizeUpgrade(newImplementation);
        assembly { sstore(_IMPLEMENTATION_SLOT, newImplementation) }
        if (data.length > 0) { (bool success,) = newImplementation.delegatecall(data); require(success); }
    }

    function _authorizeUpgrade(address newImplementation) internal virtual;
}
