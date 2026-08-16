# Mainnet Deployment Record

Battery Passport is deployed on Stellar Mainnet.

## Verified identifiers

    Contract ID: CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5
    WASM SHA-256: 0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663
    RPC: https://soroban-rpc.mainnet.stellar.gateway.fm
    Network passphrase: Public Global Stellar Network ; September 2015

The local release build and deployed Mainnet contract return the same WASM hash.

## Local verification

    stellar contract build --locked

Then verify:

    Get-FileHash .\target\wasm32v1-none\release\battery_passport.wasm -Algorithm SHA256

Expected:

    0E7F3CE1012E76B877EF021C2D8DE5AEF0303A509AA3138DFAB4F8AE60347663

## Mainnet verification

Use Stellar CLI contract info against:

    CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5

The deployed hash must equal the verified production hash above.

## Production frontend

Production configuration is stored in:

    frontend/src/contractConfig.ts

No production .env file is required.

## Unknown metadata

The deployment transaction and current contract admin are not asserted because they have not been independently verified in this repository.

Do not invent them.

## Redeployment policy

Do not redeploy for frontend, documentation, CI or hosting changes.

If contract bytecode intentionally changes, treat it as a new release and update the deployment record and frontend together.