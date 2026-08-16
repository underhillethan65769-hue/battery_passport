# Quality and Release Gate

Battery Passport is a deployed Mainnet product. A repository release must continue to match the verified production contract.

## Contract gate

- Rust formatting passes.
- All contract tests pass.
- stellar contract build --locked succeeds.
- Local WASM SHA-256 equals $ExpectedWasmHash.
- The deployed Mainnet contract reports the same WASM hash.
- Required production interface functions remain present.

## Frontend gate

- 
pm ci succeeds from the committed lock file.
- TypeScript type-check passes.
- production build passes.
- Mainnet contract ID, RPC and network passphrase are pinned correctly.
- public reads do not request signatures.
- state-changing actions use Freighter and wait for final transaction status.

## Security gate

- no secret key, recovery phrase or signing XDR is tracked;
- no stale Testnet deployment workflow remains in the production repository;
- generated 	arget, 
ode_modules, dist and test snapshot files are not tracked;
- role, ownership, verification, recall and recycling rules remain covered by contract tests.

## Release command

`powershell
powershell -ExecutionPolicy Bypass -File .\scripts\verify-release.ps1
`

Push only after the verifier passes.