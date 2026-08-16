# Contributing

Battery Passport uses **Conventional Commits 1.0.0** for every commit in the production repository.

## Commit subject

```text
<type>[optional scope]: <description>
```

Supported project types:

- `feat` — product capability or user-facing feature
- `fix` — bug fix
- `refactor` — code restructuring without a behavior change
- `test` — test additions or corrections
- `docs` — documentation
- `style` — formatting/style-only change
- `perf` — performance improvement
- `build` — build system or dependency change
- `ci` — CI/workflow change
- `chore` — maintenance work

Use a scope when it improves clarity, for example `contract`, `frontend`, `wallet`, `mainnet`, `deployment` or `docs`.

Examples:

```text
feat(contract): add role-based lifecycle authorization
fix(frontend): wait for final transaction confirmation
refactor(frontend): simplify public verification flow
test(contract): cover recycling approval boundary
docs(mainnet): update production deployment record
```

Breaking changes use `!` and should explain the change in the commit body/footer when needed:

```text
feat(contract)!: replace legacy lifecycle state model
```

Commits should be atomic, meaningful and truthful. Do not create empty commits, fake history or large unrelated bulk commits merely to increase commit count.
