# Quality and Release Gate

Battery Passport is a deployed Stellar Mainnet product.

## Contract gate

Rust formatting must pass.

All contract tests must pass.

The locked production WASM build must succeed.

The local WASM must match the verified Mainnet release.

The deployed Mainnet contract must expose the required lifecycle interface.

## Frontend gate

npm ci must succeed from the committed lock file.

TypeScript type-check must pass.

Production build must pass.

Mainnet Contract ID, RPC and network passphrase must be correct.

Public reads must not request signatures.

State-changing actions must use Freighter.

## Security gate

No secret key, recovery phrase or signing XDR may be tracked.

Generated target, node_modules, dist and test snapshot artifacts must not be tracked.

No stale Testnet deployment workflow should remain.

## Release command

    powershell -ExecutionPolicy Bypass -File .\scripts\verify-release.ps1

Push only after the release verifier passes.