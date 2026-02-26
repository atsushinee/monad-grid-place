import { useAccount, useConnect, useDisconnect } from 'wagmi'

export function ConnectButton() {
  const { address, isConnected } = useAccount()
  const { connectors, connect } = useConnect()
  const { disconnect } = useDisconnect()

  const buttonStyle = "bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg transition duration-300";

  if (isConnected) {
    return (
      <div className="flex items-center gap-4">
        <p className="text-sm font-mono bg-gray-800 px-3 py-1 rounded-md">{`${address?.slice(0, 6)}...${address?.slice(-4)}`}</p>
        <button onClick={() => disconnect()} className={buttonStyle}>
          Disconnect
        </button>
      </div>
    )
  }

  return (
    <div>
      {connectors.map((connector) => (
        <button key={connector.id} onClick={() => connect({ connector })} className={buttonStyle}>
          Connect with {connector.name}
        </button>
      ))}
    </div>
  )
}
