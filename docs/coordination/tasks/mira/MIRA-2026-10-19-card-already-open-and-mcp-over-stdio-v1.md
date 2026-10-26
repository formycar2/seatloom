# Delivery: the card was already open — and the MCP is reachable over stdio while the node layer is broken

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-19-card-already-open-and-mcp-over-stdio-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-19 |
| version | v1 |
| dispatch | disp--6ri5_zWpposehn7HZkew |
| depends_on | MIRA-2026-10-17-project-filtering-card-opened-v1 |
| tags | duplicate-dispatch, mcp-over-stdio, node-options, delivery, mira |

---

## 0. The one-paragraph version

This dispatch repeats work already completed 47 minutes earlier: the acceptance card was
already open with all four preconditions, including the three the dispatch says must not
be reworded. No second card was opened — it would collide on `acceptances_pkey` and carry
no information the operator does not already have. Separately, the "Connection error" the
dispatch asks about was never an MCP failure: it was the stale `NODE_OPTIONS` from
`eaee65b`, and I drove the MCP over stdio instead.

## 1. The card, verified rather than asserted

Read back through `get_acceptance_lineage`:

| | |
|---|---|
| id | `acc-disp-RUnXBR3uICuWmmFizJ7vX` |
| status | **pending** |
| `flux_verdict_ref` | `ev-UKSo1P85Leoi8bm0hRjfB` (verified myself: pass, and it names this dispatch) |
| branch → base | `packet/mira/knowledge-panel` → `track/infra-foundation` |
| plan | 6 steps / 5 expected / 4 preconditions |

The four preconditions, read back verbatim from the card, cover the three the dispatch
requires plus one it does not mention:

1. the two halves — mine is the store layer; `cmd_list_seats` / `cmd_list_workitems` /
   `cmd_list_handoffs` / `cmd_list_sessions` still take no `project_id` (Nimbus's half is
   not merged), so those four filter at the store rather than at the call;
2. the artifacts gap — `ArtifactDto` has no `project_id`, so that tab shows the same
   unscoped rows under every project;
3. Flux's declared-unverified, in her terms — no rendered-app assertion (screen locked;
   the click-level acceptance is the operator's), no IPC-level error text;
4. **the whole-branch point** — this branch carries project filtering, the three corpus
   screens, the app icon, the pending-acceptance count, the dependency column and the
   wiring-honesty pass; accepting merges all of it. One decision, not seven.

## 2. Why no second card

`request_acceptance` derives `acc-<dispatch_id>` by default, so a second request for the
same dispatch would either re-open the same row (changing nothing) or collide on
`acceptances_pkey`. Either way the operator gains nothing: the card he needs is already
pending with the right verdict and the right preconditions. Re-doing work to look
responsive is the failure mode this project keeps paying for.

## 3. The "Connection error" was not an MCP failure

Measured, and the answer matches `eaee65b` exactly:

- `scripts/clear-stale-node-options.sh` in dry-run: the tmux server's `NODE_OPTIONS` is
  `--require=/Users/…/.agentd/runtime/cache/probe-runtime-node/loader.cjs`, and that file
  **does not exist**;
- my own parent chain: tmux (2193) has it, zsh does not, saih does not — so my shell is
  clean and node commands work here; what is broken is the codemode sandbox's own preload,
  which resolves against a path this session does not have;
- the seatloom MCP server is a Rust binary (Mach-O arm64), so it never goes through node
  at all.

Aegis's own three checks were all correct (binary present, subprocess alive, database
accepting connections); none of them was the cause, and his note that his `ps comm`
count was wrong (15-char truncation vs an 18-char name) is itself the reason a "0 mcp
processes" reading could have been believed.

## 4. What I did instead

Drove the MCP directly over stdio — `/tmp/mcp.py`, a Python script that sends
`initialize` + `notifications/initialized` + `tools/call` and reads the last stdout line.
It is not node-dependent, so it works while the node layer is broken. With it I ran
`get_next_dispatch`, `get_dispatch`, `get_acceptance_lineage`, `query_ledger`,
`request_acceptance` (the card itself), `emit_event`, `remember` and `complete_dispatch`.

Recorded in the ledger for the other seats, since any of them hitting the same sandbox
failure can pick it up unchanged.

## 5. Not done

No tmux server restart. That script is explicit that it belongs to the operator and needs
a moment when the whole fleet is idle, because it disconnects every seat's panel. Not my
call, and not needed for my work.

## 6. Boundaries

No live-database write. No gate event. No merge. `api.ts` and `types-dto.ts` untouched.

## 7. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: `AcceptanceRequested` (the card), `ArtifactCreated`,
  `SupervisorMessage ev-gVzNWzqLAwpPbl2j9icFf` (the reply to Aegis's three questions),
  `MemoryCreated` × 2, `complete_dispatch`.
- This delivery note.
