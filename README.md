# Battery Passport

Battery Passport is a Stellar Mainnet application for tracking and verifying a battery lifecycle from manufacture through inspection, ownership changes, verification, recall and recycling.

Public users can look up a battery by serial number without signing a transaction. Authorized participants connect Freighter only when they need to write a lifecycle event.

## Mainnet status

`	ext
Network:
Stellar Mainnet

Contract ID:
CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5

WASM SHA-256:
0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663

RPC:
https://soroban-rpc.mainnet.stellar.gateway.fm

Public read account:
GANXWLW37X7D6FHGHPOQZGYQRF5G5EYAALQX4R6FNS2XT3B4UPTYL63L
`

The local release WASM and the deployed Mainnet contract have been verified to use the same WASM hash.

The deployed interface includes the production lifecycle functions:

- create_passport
- dd_inspection
- erify_passport
- 	ransfer_owner
- lag_recall
- equest_recycling
- pprove_recycling
- execute_recycling
- grant_role
- evoke_role
- 	ransfer_admin
- efresh_passport_ttl
- efresh_role_ttl

The complete public deployment record is in [deployments/mainnet.json](deployments/mainnet.json).

## Product flow

### Public verification

1. Enter a battery serial number.
2. Read lifecycle status and health score from the Mainnet contract.
3. Review origin, ownership and lifecycle audit history.
4. Open Stellar explorer details when deeper technical verification is useful.

Read-only queries are simulated through Stellar RPC. They do not request a wallet signature and do not submit a transaction.

### Authorized lifecycle participants

The contract uses role-based access control:

- **Manufacturer** â€” creates battery passports.
- **Inspector** â€” records health inspections.
- **Verifier** â€” verifies batteries that satisfy the inspection rules.
- **Recall authority** â€” flags a battery recall.
- **Recycler** â€” approves recycling requests.
- **Owner** â€” transfers ownership and initiates/completes the recycling lifecycle.
- **Admin** â€” manages operational roles and can transfer administrative authority.

Recycling requires both the current owner workflow and an authorized recycler approval.

## Why Stellar

Soroban provides a shared lifecycle state and wallet authorization layer across organizations that would otherwise maintain separate records.

Battery Passport uses Stellar for:

- wallet-authorized role actions;
- shared lifecycle state;
- public battery status;
- audit history;
- deterministic lifecycle rules;
- verifiable Mainnet contract state.

The application does not store user private keys.

## Repository

`	ext
contracts/battery_passport/   Soroban contract and tests
frontend/                     React + TypeScript production UI
scripts/                      Release verification and Mainnet TTL operations
deployments/                  Verified public Mainnet identifiers
docs/                         Architecture, security, deployment and user guide
.github/workflows/            CI
`

## Contract verification

`powershell
cargo fmt --all -- --check
cargo test --workspace
stellar contract build --locked

Get-FileHash 
  .\target\wasm32v1-none\release\battery_passport.wasm 
  -Algorithm SHA256
`

Expected SHA-256:

`	ext
0E7F3CE1012E76B877EF021C2D8DE5AEF0303A509AA3138DFAB4F8AE60347663
`

Verify the deployed contract:

`powershell
stellar contract info hash 
  --contract-id CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5 
  --rpc-url https://soroban-rpc.mainnet.stellar.gateway.fm 
  --network-passphrase "Public Global Stellar Network ; September 2015"
`

The on-chain result must be:

`	ext
0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663
`

## Frontend

Production network values are pinned in:

`	ext
frontend/src/contractConfig.ts
`

No production .env file is required.

Run locally:

`powershell
cd frontend
npm ci
npm run type-check
npm run build
npm run dev
`

Freighter must be connected to Stellar Mainnet before state-changing actions are signed.

The configured read account is a public G-address used only as the transaction source for read-only RPC simulation. It is not a signing secret.

## Release verification

From the repository root:

`powershell
powershell -ExecutionPolicy Bypass -File .\scripts\verify-release.ps1
`

The verifier checks:

- Rust formatting;
- all contract tests;
- locked production WASM build;
- exact local WASM SHA-256;
- exact deployed Mainnet WASM SHA-256;
- required deployed interface functions;
- frontend dependency install;
- TypeScript type-check;
- production frontend build;
- Mainnet runtime identifiers;
- stale Testnet/pre-deployment files and wording;
- tracked secrets/generated artifacts when Git is initialized.

## Mainnet operations

The contract contains bounded TTL maintenance functions for long-lived lifecycle records.

Operator helpers:

`	ext
scripts/maintain-passport-ttl.ps1
scripts/maintain-role-ttl.ps1
`

These scripts are Mainnet-only and require a local Stellar identity when they submit maintenance transactions. They never contain a secret key.

## Security boundary

- role-gated actions are enforced by the contract;
- ownership transfer requires the current owner;
- verification requires the required inspection state;
- recycling requires the lifecycle approval flow;
- recycled and recalled state transitions are constrained;
- state-changing frontend actions use Freighter signing;
- public reads do not request signatures;
- no private key or recovery phrase is required by the application.

See [docs/SECURITY_REVIEW.md](docs/SECURITY_REVIEW.md).

## Deployment policy

The current Mainnet contract is already deployed and its WASM matches this source tree.

Do **not** redeploy the contract for documentation, frontend copy, CI, hosting or repository changes.

A new deployment is appropriate only when the intended contract bytecode changes. A new WASM hash must be treated as a new release and all public deployment records must be updated together.

## Evidence policy

Do not fabricate a deployment transaction, current admin, user flow, transaction count or production metric.

The repository records only facts that have been independently verified.