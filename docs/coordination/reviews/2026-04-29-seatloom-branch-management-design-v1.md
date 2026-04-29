# Review: SeatLoom Branch Management Design v1

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | LYRA-2026-04-29-seatloom-branch-management-design-v1 |
| status | proposed |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| depends_on | `docs/coordination/COORDINATION_RULES.md`, `docs/coordination/COLLABORATION_PROTOCOL.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-v1.md` |
| tags | branch-management, commit-pinned-verification, flux, nimbus, infrastructure |

## 1. Why This Proposal Exists

SeatLoom is now operating with multiple seats contributing in parallel across product, frontend, and infrastructure. The current repo state already shows the main risk of a branchless or loosely managed flow:

- verification can accidentally target a floating workspace instead of a precise delivery,
- frontend and infrastructure changes can mix in one dirty tree,
- remote verification cannot reliably reproduce the intended build without an exact commit,
- low-level verifier-side repairs can disappear into local state unless they are committed immediately.

The sponsor also made two explicit governance requirements:

1. Flux acceptance should be tied to exact commit hashes.
2. If Flux directly fixes low-level code issues, that action must leave a durable trace and be committed promptly to the current branch.

This proposal turns those requirements into a repo workflow.

## 2. Proposed Branch Model

### 2.1 Protected integration baseline

`main`

Use `main` only for accepted work. It should remain the clean integration baseline and should not be the default scratch space for ongoing multi-seat edits.

### 2.2 Track branches

`track/<domain>`

Use one track branch per active workstream that can evolve somewhat independently.

Recommended immediate tracks:

- `track/frontend-redesign`
- `track/infra-foundation`

Why this layer exists:

- frontend and infrastructure are currently being driven by different owners,
- each stream needs a stable integration line for its own workers and verifiers,
- this reduces accidental reopening of unrelated surfaces.

### 2.3 Packet branches

`packet/<seat>/<packet-id>`

Every code-changing worker packet should land on a short-lived packet branch created from its declared track branch. The packet branch exists only to deliver one bounded change set.

Examples:

- `packet/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline`
- `packet/mira/MIRA-2026-04-29-s7b-supervisor-command-bar`

### 2.4 Hotfix branches

`hotfix/<seat>/<topic>`

Only use this branch class for emergency or tiny bounded repairs that must be isolated from a larger packet branch.

## 3. Verification Model

### 3.1 Commit-pinned verification

Flux should never verify “whatever is currently in the workspace.”

Every verification packet must name:

- target branch
- target commit SHA
- compare base commit when delta-only review is intended

Flux must report before testing:

- `git rev-parse --abbrev-ref HEAD`
- `git rev-parse HEAD`
- `git status --short`

If the branch or commit does not match the packet, Flux should stop and return `HOLD`.

### 3.2 Snapshot fallback is diagnostic only

An `scp` copy of a dirty local workspace is still useful for diagnosis, but it should not close acceptance. If we use that path, the resulting verdict should remain `PROVISIONAL` or `HOLD` until the same state is represented by a real commit.

## 4. Flux Bounded-Fix Exception

Default rule remains unchanged: Flux is a verifier, not a product implementation owner.

However, a narrow exception is useful for tiny low-risk defects found during verification, such as:

- missing assets,
- build harness issues,
- verification script defects,
- missing imports,
- evidence-command typos,
- non-behavioral warning cleanup.

When Lyra explicitly routes such a fix to Flux:

1. Flux applies the fix on the current target branch,
2. Flux commits immediately with `fix(flux): ...`,
3. Flux reports `pre_fix_commit` and `fix_commit`,
4. Flux reruns the bounded verification,
5. Lyra accepts or rejects against the new exact commit.

Anything larger than that should be routed back to Nimbus or Mira.

## 5. Immediate Implication for the PostgreSQL Baseline

The current PostgreSQL baseline is structurally complete, but its final remote verification should now follow the stronger commit-pinned rule.

That means:

1. Nimbus should publish the infrastructure baseline on a real branch/commit that excludes unrelated frontend work.
2. Flux should verify that exact branch + commit on the sponsor-provided remote workspace.
3. If we only ship an `scp` snapshot of uncommitted local changes, the result can inform us but should not close acceptance.

## 6. Recommended Adoption Order

1. Freeze the branch policy in governance.
2. Create or confirm `track/infra-foundation` for Nimbus-owned infrastructure work.
3. Have Nimbus produce one bounded delivery commit for the PostgreSQL baseline on that track.
4. Reissue the remote Flux verification packet with the exact branch + commit.
5. Close the current PostgreSQL `HOLD` only after the commit-pinned remote verification passes.

## 7. Proposed Default Rules Going Forward

- `main` = accepted baseline only
- `track/*` = active stream integration line
- `packet/*` = one bounded worker delivery
- `hotfix/*` = bounded repair only
- no acceptance on floating state
- every verification cites exact commit
- every direct Flux fix requires an explicit packet and an immediate commit

## Recommendation

Adopt this branch model now for infrastructure work first, because the PostgreSQL baseline is already blocked on reproducible remote verification and is the clearest place where commit-pinned acceptance adds immediate value.
