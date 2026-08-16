# Security Review

This document describes the security boundary of the Battery Passport Mainnet MVP. It is not an external audit report.

## Contract controls

Admin authority is stored on-chain and can be transferred.

Operational roles are managed through authorized admin actions.

create_passport requires the manufacturer role.

add_inspection requires the inspector role.

verify_passport requires the verifier role and valid inspection state.

flag_recall requires recall authority.

Ownership transfer requires the current owner.

Recycling requires the owner and recycler lifecycle approval flow.

Recycler authorization is enforced by contract logic.

Recycled batteries cannot return to normal lifecycle write paths.

A new inspection can invalidate previous verification state.

## Frontend controls

No private key or recovery phrase is requested.

Freighter signs state-changing transactions.

The wallet network is checked before write submission.

Read-only contract calls do not request a user signature.

Production network identifiers are public configuration values.

## Production code identity

    Contract: CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5
    WASM SHA-256: 0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663

The release gate must fail if the verified production identity changes unexpectedly.

## Public data

Battery serials, lifecycle metadata, wallet addresses, health scores and audit history stored on Stellar are public blockchain data.

Do not store confidential personal or commercial information on-chain.

## State lifetime

Persistent Soroban state is subject to state archival and TTL behavior.

The project includes bounded maintenance helpers for passport, audit and role state.

## Admin operations

Admin authority is security-sensitive. Signing material must remain outside this repository.

The current admin is not asserted because it has not been independently verified during this release sync.