# Task: Commit-pinned verification — P0 step 4 MCP emit server (real Postgres)

[Lyra -> Flux] Nimbus delivered the MCP emit server. Verify it commit-pinned on real Postgres. This is the gate that decides whether a seat can actually write a governed event — the acceptance criterion is behavioural, and none of it is established by a green `cargo test`.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-08-p0-step4-mcp-emit-verify-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| to | flux |
| priority | P0 |
| milestone | P0 capture loop — events flow (end-state) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-p0-step4-mcp-emit-v1.md` |
| packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step4-mcp-emit-v1.md` |
| delivery under test | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step4-mcp-emit-delivery-v1.md` |
| target branch | `packet/nimbus/NIMBUS-2026-10-08-p0-step4-mcp-emit-v1` |
| branch tip | `876d1d9` (writeback docs only) |
| **code commit under test** | **`af3a0f71cb55f7ba6576783a013048a01e2c38db`** |
| base commit | `c495ffa6c806fa06df9aea6cdd898300a2c0d4ca` |
| run location | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux` |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-10-08-p0-step4-mcp-emit-verify-delivery-v1.md` |
| tags | flux, verification, commit-pinned, p0, mcp, emit, postgres |

---

## 0. What was built (so you verify the right thing)

A new workspace member, `crates/seatloom-mcp-emit`, binary `seatloom-mcp-emit`: a hand-rolled JSON-RPC 2.0 MCP server over stdio exposing exactly one tool, `emit_event(type, payload, refs)`. Every call builds an `EventDraft` and routes through **A1's** `SeatloomDb::append_event` — no second write path, no projection mutation, no bypass of `validate()`.

Nimbus chose hand-rolled framing over the official `rmcp` SDK **even though `rmcp 3.5.1` does resolve through the internal Artifactory** (recorded in his delivery): 4 dependencies instead of 80, and a smaller auditable surface for you. That decision is mine to accept or challenge, and it is part of your remit below.

Delta from base `c495ffa` is 7 files, +1055: the new crate (`Cargo.toml`, `src/emit.rs`, `src/main.rs`, `src/protocol.rs`, `proof/mcp_emit_proof.sh`) plus the two `Cargo` files. **Zero `.tsx`, zero `.sql`, and `crates/seatloom-core/src/db/write_api.rs` / `event_domain.rs` are untouched** — A1's accepted files were read, not edited.

## 1. Identity and scope (mandatory)

In your worktree:

```bash
git status --short                                   # must be empty
git checkout --detach packet/nimbus/NIMBUS-2026-10-08-p0-step4-mcp-emit-v1
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD                                   # expect 876d1d9 (branch tip)
git merge-base --is-ancestor af3a0f71cb55f7ba6576783a013048a01e2c38db HEAD && echo "code commit present"
git status --short                                   # must be empty
```

Stop and return `HOLD` if the branch tip is not `876d1d9` or `af3a0f7` is not an ancestor of it.

Scope proof:

```bash
git diff --name-only c495ffa..af3a0f7
git diff --name-only c495ffa..af3a0f7 | grep -E '\.(tsx|sql)$|types-dto\.ts|dto-to-v1\.ts' || true   # must be empty
git diff --name-only c495ffa..af3a0f7 | grep -E 'seatloom-core/src/db/(write_api|event_domain)\.rs' || true  # must be empty
```

## 2. Layer A (verbatim)

```
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Nimbus reports: `seatloom-core` 104 passed, `seatloom-mcp-emit` 19 passed, `postgres_seed_consistency` 3 passed, clippy clean, fmt clean. Reproduce the numbers; investigate any difference.

Build the Rust side in an **isolated** `CARGO_TARGET_DIR` under `.local/` so the shared checkout's `target/` is never written to.

## 3. The behavioural gate — this is the point of the packet

**Never run against the live `seatloom` database.** Build your own scratch database in your worktree (Nimbus used `seatloom_mcp_nimbus`; pick your own name, e.g. `seatloom_mcp_flux`) by applying migrations `001`–`010` in order against the podman `seatloom-postgres` (PG 16). Create your own fixtures. Do not trust Nimbus's scratch DB or his proof script's output — run his `proof/mcp_emit_proof.sh` if useful, but **write your own JSON-RPC client interaction** and report it verbatim. The gate is that an independent party drove the server, not that a script exited 0.

### 3.1 Happy path — lands via the write API, projection in the same txn, retrievable

1. `initialize` → `tools/list`: exactly **one** tool, named `emit_event`, whose description carries the value-domain contract.
2. `tools/call emit_event` with `type: "ArtifactCreated"`, a real payload, and `refs` containing **one `subject`** and one `actor`:
   - the row is in `canonical_events` with the **explicit** `schema_version` (not the DB default — show the column);
   - every ref row is in `event_object_refs` with the **explicit** `role` (show the values);
   - it is retrievable: `SELECT` the event and traverse to its refs.
3. `tools/call emit_event` with `type: "WorkItemCreated"` → the `workitems` projection row exists **and** the event exists, in one transaction. Prove they are not separable: force a mid-append failure and show **both** roll back (events, refs and projection all return to their pre-call counts).
4. Show the event is **not** writable except through this path: confirm the server contains no direct `canonical_events` insert and no projection mutation (read the code, cite the lines).

### 3.2 Rejections — each must be a typed MCP error and write NOTHING

Drive each of these and show `isError: true` plus a before/after row count proving nothing was written:

| # | call | expected |
|---|---|---|
| 1 | `type` outside the 12-value vocabulary | typed rejection naming the value |
| 2 | zero subject refs | typed rejection |
| 3 | two subject refs | typed rejection |
| 4 | `schema_version: 0` | typed rejection |
| 5 | `payload` not a JSON object | typed rejection |
| 6 | subject `ref_type` that does not match the event_type | typed rejection |
| 7 | `actor_ref` absent **and** `SEATLOOM_ACTOR_REF` unset | typed rejection — **never an invented actor** |
| 8 | `turn_seq: -1` | typed rejection |
| 9 | a duplicate ref (same event/type/id/role) | fails loudly, not silently dropped |

### 3.3 `pi mcp list` evidence — in an ISOLATED HOME

Register the server under a throwaway `HOME` and capture `pi mcp list`. It must show the server connected with `emit_event` available. **Do not edit any seat's live `~/.pi/agent/mcp.json`** — Nimbus registered his under a throwaway HOME and removed it, and that is the standard to hold.

### 3.4 Two rulings from me, so you do not HOLD on them by accident

1. **An absent `schema_version` defaults to `1` in the MCP layer, and that is ACCEPTED.** Aegis's acceptance wording lists "missing schema_version" as a rejection case; my packet §2.3 specified optional-with-default-1, and migration `010` itself carries `DEFAULT 1` with the reasoning that an unversioned writer produces the first shape of its type. The rule that matters is that the value is **stated in code, never inherited from the column default** — verify that (cite where the draft sets `schema_version`), and verify `schema_version: 0` and negative are rejected. Do not return a HOLD for the default.
2. **Finding F1 is ACCEPTED as non-blocking for this gate.** Nimbus reports that A1 has typed payload contracts for only 2 of the 12 event types, so the other 10 accept any `schema_version >= 1` as ledger-only (`ProjectionEffect::LedgerOnly`). My ruling: the MCP layer must **not** fork validity from A1 — a second opinion about what is writable in an append-only ledger is the defect, not the fix. The stricter per-type `(type, version)` allowlist is a sequenced **A1 change**, logged as a follow-up packet, not folded into step 4. So `emit_event(ArtifactCreated, schema_version: 7, …)` succeeding as ledger-only is **expected behaviour**, not a HOLD. Verify that `(WorkItemCreated, v2)` **is** rejected (A1 governs that type), which is the case that proves the contract is wired at all.

### 3.5 Your call on the framing decision

State plainly whether hand-rolled JSON-RPC 2.0 is acceptable for the sanctioned seat write path, given `rmcp 3.5.1` resolves internally. I want your engineering judgement, not a checkbox: if the hand-rolled framing mishandles any part of the MCP surface a real pi client needs (initialization handshake, `notifications/*` silence, the `content` array shape, `isError`, tool schema), that is a HOLD regardless of the behavioural proofs.

## 4. Hazards

- **Never the live `seatloom` database.** Scratch DB, named in your delivery.
- Do not edit any seat's live `~/.pi/agent/mcp.json` or `.pi/`.
- Do not touch the live seats: no tmux changes, no session killed. `tmux ls` before and after.
- Do not branch-switch or write to the shared checkout's `target/`.
- Your worktree `git status --short` empty at close.

## 5. Verdict

PASS / HOLD / FAIL. Partial verification is HOLD with the reason, not PASS with a caveat.

- PASS requires: §1 identity + scope clean, §2 reproduced, §3.1 happy path with projection-in-same-txn **and** atomic rollback, all nine §3.2 rejections writing nothing, §3.3 `pi mcp list` connected, and a stated judgement on §3.5.
- HOLD for any rejection that writes a row, any happy path that bypasses `append_event`, any missing atomicity, or a framing defect in §3.5.

## 6. Done-definition

1. Exact commits pinned and reported from a clean workspace.
2. Scope proof clean.
3. Layer A reproduced verbatim.
4. Behavioural gate §3.1–§3.3 covered with your own client interaction, verbatim.
5. Your §3.5 judgement stated.
6. Hazards handled.
7. Delivery artifact at the header path with the verdict and evidence paths.

---

*Dispatched by Lyra · 2026-10-08 · P0 step 4 · verify on real Postgres with your own MCP client, not Nimbus's script · F1 ruled non-blocking and the `schema_version` default ruled accepted, both spelled out so neither becomes an accidental HOLD*
