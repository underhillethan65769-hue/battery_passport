# Mainnet Deployment Record

Battery Passport is already deployed on Stellar Mainnet.

## Verified production identifiers

`	ext
Contract ID:
CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5

WASM SHA-256:
0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663

RPC:
https://soroban-rpc.mainnet.stellar.gateway.fm

Network passphrase:
Public Global Stellar Network ; September 2015

Public read account:
GANXWLW37X7D6FHGHPOQZGYQRF5G5EYAALQX4R6FNS2XT3B4UPTYL63L
`

The local release build and the deployed contract return the same WASM hash.

## Verify local source identity

`powershell
stellar contract build --locked

Get-FileHash 
  .\target\wasm32v1-none\release\battery_passport.wasm 
  -Algorithm SHA256
`

Expected:

`	ext
0E7F3CE1012E76B877EF021C2D8DE5AEF0303A509AA3138DFAB4F8AE60347663
`

## Verify Mainnet identity

`powershell
stellar contract info hash 
  --contract-id CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5 
  --rpc-url https://soroban-rpc.mainnet.stellar.gateway.fm 
  --network-passphrase "Public Global Stellar Network ; September 2015"
`

Expected:

`	ext
0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663
`

Inspect the deployed interface:

`powershell
stellar contract info interface 
  --contract-id CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5 
  --rpc-url https://soroban-rpc.mainnet.stellar.gateway.fm 
  --network-passphrase "Public Global Stellar Network ; September 2015"
`

The interface must include the lifecycle functions used by the production frontend.

## Production frontend

The frontend is configured directly in:

`	ext
frontend/src/contractConfig.ts
`

No production .env file is required.

The configured public read account is used only to obtain a valid source account/sequence for read-only transaction simulation. It does not sign public lookup requests.

## Unknown metadata

The deployment transaction and current contract admin have not been independently verified in this repository snapshot.

They are intentionally not invented. If they are later verified, add them to deployments/mainnet.json.

## State lifetime operations

Long-lived lifecycle records use persistent Soroban storage and require operational TTL awareness.

The Mainnet-only helpers are:

`	ext
scripts/maintain-passport-ttl.ps1
scripts/maintain-role-ttl.ps1
`

They submit real Mainnet transactions from a locally configured Stellar identity. Never embed the identity secret in repository files.

## Redeployment policy

Do not redeploy this contract for frontend, documentation, CI, hosting or repository-only changes.

If Rust contract code changes and the resulting WASM hash differs from the recorded production hash, treat it as a new contract release and update the deployment record, frontend configuration and release verification together.