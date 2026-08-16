# Architecture

## Product boundary

Battery Passport is intentionally small: a Soroban lifecycle registry plus a browser frontend. There is no application backend in the current MVP because the product does not require private off-chain state to perform its core job.

## Read flow

```text
Serial search
  -> frontend builds a read-only contract invocation
  -> Stellar RPC simulates the invocation
  -> ScVal result is converted to a user-facing passport/timeline
```

A configured public G-address is used as the transaction source for simulation. The corresponding secret key is not needed.

## Write flow

```text
Role/ownership check in UI
  -> frontend builds and prepares transaction
  -> Freighter asks the user to sign
  -> transaction is submitted to Stellar RPC
  -> frontend polls until SUCCESS or failure
  -> product refreshes the passport state
```

The UI role check is convenience only. Contract authorization remains the security boundary.

## Contract state

Instance storage:
- platform admin/config;
- aggregate registry statistics.

Persistent storage:
- role bitmasks by address;
- battery passports by serial;
- audit records;
- recycling approvals.

Battery lifecycle uses a single status field: Active, Verified, Under review, Recalled or Recycled. Verification and recall are not maintained as independent booleans, preventing contradictory states. Every new inspection invalidates the previous verification so a verifier always attests to the latest inspection state.

## Authorization

The admin grants operational roles. Each privileged function both requires a wallet authorization and checks the stored role. Ownership actions independently verify the current passport owner.

Recycling uses three steps: owner request, authorized recycler approval, owner execution. The final execution verifies that the recycler still holds the recycler role.


## State lifetime

Battery passports, roles, audit records and recycling approvals use persistent storage. The contract extends persistent-entry TTL when those entries are touched by a submitted contract transaction. Contract instance/code TTL is also renewed when contract state is accessed in a submitted transaction.

Public verification in the browser uses RPC simulation, so a public read does **not** count as an on-chain maintenance transaction. Production operations therefore include periodic TTL maintenance. `refresh_passport_ttl` is intentionally not exposed in the user interface; an operator can call it in batches of at most 50 audit records using `scripts/maintain-passport-ttl.ps1`. Dormant participant role entries can be renewed with `refresh_role_ttl` through `scripts/maintain-role-ttl.ps1`. Active roles are naturally renewed when they authorize submitted lifecycle transactions.

If an entry has already become archived, it must be restored using Stellar's state-restoration tooling before the contract can access and renew it again.
