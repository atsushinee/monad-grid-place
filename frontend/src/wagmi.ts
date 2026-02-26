import { http, createConfig } from 'wagmi'
import { anvil } from 'wagmi/chains'
import { injected } from 'wagmi/connectors'

export const config = createConfig({
  chains: [anvil],
  connectors: [
    injected(),
  ],
  transports: {
    [anvil.id]: http(),
  },
})