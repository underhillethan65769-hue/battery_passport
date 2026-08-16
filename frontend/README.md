# Battery Passport Frontend

React + TypeScript + Vite frontend for the deployed Battery Passport application on Stellar Mainnet.

## Production configuration

The application is pinned to:

`	ext
Network:
Stellar Mainnet

Contract:
CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5

RPC:
https://soroban-rpc.mainnet.stellar.gateway.fm

Read account:
GANXWLW37X7D6FHGHPOQZGYQRF5G5EYAALQX4R6FNS2XT3B4UPTYL63L
`

These are public runtime identifiers stored in src/contractConfig.ts.

No production .env file is required.

## Local development

`powershell
npm ci
npm run type-check
npm run build
npm run dev
`

## Wallet behavior

Public battery lookup uses read-only RPC simulation and does not request a signature.

Lifecycle writes use Freighter. The wallet network is checked against Stellar Mainnet before a write transaction is prepared and signed.

Never put a private key, seed phrase or recovery phrase in frontend source, environment variables or deployment settings.