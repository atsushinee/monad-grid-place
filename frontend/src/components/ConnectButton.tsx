import { useAccount, useConnect, useDisconnect } from 'wagmi';

export function ConnectButton() {
  const { address, isConnected } = useAccount();
  const { connectors, connect } = useConnect();
  const { disconnect } = useDisconnect();

  const buttonStyle = "bg-[#00FFAA] hover:bg-green-400 text-black font-bold py-2 px-4 rounded-lg transition duration-300 ease-in-out";

  if (isConnected) {
    return (
      <div className="flex items-center gap-4">
        <p className="text-sm font-mono bg-gray-800 px-3 py-1 rounded-md">{`${address?.slice(0, 6)}...${address?.slice(-4)}`}</p>
        <button onClick={() => disconnect()} className={buttonStyle}>
          Disconnect
        </button>
      </div>
    );
  }

  const injectedConnector = connectors.find(c => c.type === 'injected');

  if (!injectedConnector) {
    return (
      <a href="https://metamask.io/download/" target="_blank" rel="noopener noreferrer" className={buttonStyle}>
        Install Wallet
      </a>
    );
  }

  return (
    <button onClick={() => connect({ connector: injectedConnector })} className={buttonStyle}>
      Connect Wallet
    </button>
  );
}
