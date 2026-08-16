# Battery Passport

Battery Passport is a Stellar Mainnet application for tracking and verifying a battery lifecycle from manufacture through inspection, ownership transfer, verification, recall and recycling.

Public users can look up a battery by serial number without signing a transaction. Authorized participants connect Freighter when they need to write a lifecycle event.

## Mainnet

    Network: Stellar Mainnet
    Contract ID: CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5
    WASM SHA-256: 0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663
    RPC: https://soroban-rpc.mainnet.stellar.gateway.fm
    Public read account: GANXWLW37X7D6FHGHPOQZGYQRF5G5EYAALQX4R6FNS2XT3B4UPTYL63L

The local release WASM and deployed Mainnet contract have been verified to use the same WASM hash.

## Contract capabilities

Production lifecycle functions include:

- create_passport
- add_inspection
- verify_passport
- transfer_owner
- flag_recall
- request_recycling
- approve_recycling
- execute_recycling
- grant_role
- revoke_role
- transfer_admin
- refresh_passport_ttl
- refresh_role_ttl

## Product flow

A manufacturer creates a battery passport. Inspectors record battery health. Authorized verifiers confirm eligible passports. Ownership can move between wallets. Recall authorities can flag unsafe batteries. Owners and authorized recyclers complete the recycling lifecycle through a two-party approval flow.

Public battery lookup is read-only and does not require a wallet signature.

## Roles

Manufacturer creates passports.

Inspector records inspections.

Verifier verifies eligible batteries.

Recall authority flags recalls.

Recycler approves recycling.

Owner controls ownership transfer and the owner side of recycling.

Admin manages operational roles and administrative authority.

## Why Stellar

Soroban provides wallet authorization, shared lifecycle state, public verification, deterministic lifecycle rules and an on-chain audit trail across organizations that would otherwise maintain separate records.

## Repository

    contracts/battery_passport/   Soroban contract and tests
    frontend/                     React and TypeScript frontend
    scripts/                      Release and Mainnet operations
    deployments/                  Mainnet deployment record
    docs/                         Product and security documentation
    .github/workflows/            CI

## Verification

Run from the repository root:

    cargo fmt --all -- --check
    cargo test --workspace
    stellar contract build --locked

Expected WASM SHA-256:

    0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663

Full release verification:

    powershell -ExecutionPolicy Bypass -File .\scripts\verify-release.ps1

## Frontend

Production runtime values are stored in:

    frontend/src/contractConfig.ts

No production .env file is required.

For local development:

    cd frontend
    npm ci
    npm run type-check
    npm run build
    npm run dev

Freighter must be connected to Stellar Mainnet before a state-changing transaction is signed.

## Security

The frontend never requests a private key or recovery phrase.

Contract authorization controls privileged lifecycle actions. Public reads require no signature.

See docs/SECURITY_REVIEW.md for the security boundary.

## Deployment policy

The current Mainnet contract is already deployed and matches this source tree.

Do not redeploy for documentation, frontend, CI or hosting changes.

A new contract release is required only when intended contract bytecode changes and produces a new WASM hash.

## Evidence policy

Do not fabricate deployment transactions, admin identities, production users, transaction counts or product metrics.

Only independently verified production facts should be recorded.