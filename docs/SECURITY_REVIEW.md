# Security Review

This document describes the security boundary of the deployed Battery Passport Mainnet MVP. It is not an external audit report.

## Contract controls

- admin authority is stored on-chain and can be transferred;
- operational roles can be granted/revoked only through authorized admin actions;
- create_passport requires the manufacturer role;
- dd_inspection requires the inspector role;
- erify_passport requires the verifier role and valid inspection state;
- lag_recall requires recall authority;
- ownership transfer requires the current owner;
- recycling requires the owner/recycler lifecycle approval flow;
- recycler authorization is checked by contract logic;
- recycled batteries cannot re-enter normal lifecycle write paths;
- recall and verification state transitions are constrained;
- a new inspection can invalidate the previous verification state.

## Input/state validation

The contract tests cover invalid metadata, score bounds, unauthorized role actions, ownership failures, duplicate approvals, recycling authorization, recall behavior, verification requirements and TTL helper bounds.

## Frontend controls

- no private key or recovery phrase is requested;
- Freighter signs state-changing transactions;
- Freighter network is checked before write submission;
- read-only calls are simulated and do not request a user signature;
- transaction submission is followed by final-status polling;
- raw contract errors are mapped to user-facing messages;
- production contract ID, RPC URL, network and read account are public values.

## Production code identity

`	ext
Contract:
CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5

WASM SHA-256:
0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663
`

The release gate fails if the local contract build stops matching the deployed Mainnet WASM.

## Public data

Battery serials, lifecycle metadata, public wallet addresses, health scores, audit information and state changes stored on Stellar are public blockchain data.

Do not put confidential personal or commercial information in fields intended for on-chain storage.

## State lifetime

Persistent contract state is subject to Stellar state archival/TTL behavior.

The repository includes bounded Mainnet maintenance helpers for passport/audit state and role state. Operators must monitor long-lived production records and restore archived state when necessary.

## Admin operations

Admin authority controls role assignment and is security-sensitive. Use an appropriately secured operational account/policy and keep signing material outside this repository.

The current admin is not asserted in this document because it was not independently verified as part of this sync.