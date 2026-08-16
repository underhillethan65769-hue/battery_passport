# Battery Passport Frontend

React, TypeScript and Vite frontend for Battery Passport on Stellar Mainnet.

## Production configuration

    Network: Stellar Mainnet
    Contract: CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5
    RPC: https://soroban-rpc.mainnet.stellar.gateway.fm
    Read account: GANXWLW37X7D6FHGHPOQZGYQRF5G5EYAALQX4R6FNS2XT3B4UPTYL63L

Runtime configuration is stored in src/contractConfig.ts.

No production .env file is required.

## Development

    npm ci
    npm run type-check
    npm run build
    npm run dev

Public battery lookup uses read-only RPC simulation and does not request a signature.

State-changing lifecycle actions use Freighter and require Stellar Mainnet.

Never place a private key, seed phrase or recovery phrase in frontend source or environment configuration.