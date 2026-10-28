# Delivery: acceptance-card closeout — two branches synced to track + the containment defence

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-10-acceptance-closeout-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-10 |
| version | v1 |
| tags | nimbus, acceptance-closeout, sync, containment, request-acceptance, d2 |
| governing dispatch | `disp-acclose-nimbus` (Aegis blocking ruling, highest priority) |
| governing packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-acceptance-closeout-v1.md` (on `packet/lyra/milestone-summary`; not on this branch) |
| base branch | `track/infra-foundation` @ `b6751a1` |
| delivery branches | `packet/nimbus/acceptance-lineage-readmodel` @ `dfa6ec4`; `packet/nimbus/d5-mcp-recall` @ `e4047f4` |
| scratch database | `seatloom_sync3` (001–012 + seed + 013/014 from track + 015 + 016 + scratch 017; NEVER live `seatloom`) |

---

## 0. Summary

Aegis's blocking ruling had two parts; both are done on the Nimbus lane.

**PART 1 — the two Flux-green branches were STALE** (merge-base `b84684a2` vs
track tip `b6751a1`) and an unsynchronized branch may not request acceptance.
Both are now synchronized:

| Branch | Sync commit | Contains track tip `b6751a1` |
|---|---|---|
| `packet/nimbus/acceptance-lineage-readmodel` | `37c9603` (merge of track) | yes |
| `packet/nimbus/d5-mcp-recall` | `e4047f4` (one-direction merge of the synced lineage branch) | yes |

**PART 2 — the structural defect is now PRODUCT behaviour.** `request_acceptance`
gained two containment checks, in the SAME place as the base check (one
implementation, no second one), both before any DB contact:

- **(b) the delivery branch CONTAINS the base** — `git merge-base --is-ancestor
  base branch`. Refused with `BranchNotSynced`, whose message says to merge the
  base in first.
- **(c) the merge would not DELETE files tracked on the base** — previewed with
  `git merge-tree --write-tree base branch` (writes nothing, moves no ref, leaves
  the working tree exactly as found — unlike a `merge --no-commit` rehearsal) and
  diffing `base..<resulting tree>` for `D` entries. Refused with
  `MergeWouldDelete`, listing every path that would be lost.

This is the guarantee the ruling asked for: onyx's branch would have deleted 6
tracked files (`acceptance_cmds.rs` itself), mira's 8, the old base 12. From now
on that is caught when the card is created, not when the operator clicks Accept.

**PART 3 — the two cards are pending Flux's post-sync re-verification**, which
was in flight when this doc was written (see §4).

---

## 1. PART 1 — the sync, and the conflict rule

### The rule, applied

"Never delete a file already tracked on track." Both syncs were resolved under
that rule, and the result was verified rather than assumed:

```
$ git diff --name-status track/infra-foundation <branch> | grep '^D'
(empty for both branches — nothing tracked on track is deleted)
```

### How the conflicts were resolved

Every conflicted file was a Nimbus-lane file whose track copy came from the
earlier **path-qualified D2 install** — i.e. track imported an ANCESTOR of this
branch chain. Per file it was verified that my version is a strict superset of
track's (`git diff track..mine` shows additions only; the few "deletions" are
newer versions of lines track added — the startup banner, the `tools/call` doc
comment, the acceptance row indices). Mine therefore won, and nothing track
carries was lost.

**The one exception found and handled** — and it matters, because taking mine
wholesale would have committed exactly the sin the ruling forbids:

`ui/src/lib/types-dto.ts` and `ui/src/lib/api.ts` were NOT supersets. This
branch chain's copies predate Mira's D2 acceptance surface
(`AcceptanceCardDto` / `FluxVerdictDto` / `AcceptanceDecisionDto` /
`listAcceptances` / `acceptDelivery` / `rejectDelivery` /
`ACCEPTANCE_CMDS_REGISTERED`). Resolving with `--ours` would have DELETED Mira's
merged work. Track's versions were restored and this lane's lineage DTO +
`getAcceptanceLineage` wrapper re-added on top — a union, not a replacement.

Verified after the sync: `ui/src/lib/acceptance.ts`,
`ui/src/components/AcceptanceSection.tsx`, `ui/src/views/DispatchInboxView.tsx`
and the acceptance DTOs are byte-identical to track.

### Branch strategy (declared, as the packet asks)

```
waker-hardening
  -> packet/nimbus/acceptance-lineage-readmodel   (track merged: 37c9603;
                                                   defence landed: dfa6ec4)
  -> packet/nimbus/d5-mcp-recall                  (one-direction merge of the
                                                   synced lineage branch:
                                                   0989f8d + e4047f4)
```

One direction only. An earlier attempt merged track directly into the recall
branch and produced 11 conflicted files; it was **aborted** in favour of merging
the already-synced lineage branch, which resolved with **0 conflicts** because
that branch is a superset of both sides. `dispatch.rs` therefore never diverged.

---

## 2. PART 2 — the containment defence

### Design

The two checks live in `crates/seatloom-core/src/git/ops.rs` next to
`local_branch_exists` (the helper family this lane added), so the request-time
guard and the merge path share one vocabulary and one wording:

| Helper | Question | Refusal |
|---|---|---|
| `local_branch_exists` | does the ref exist? | `UnknownBase` (pre-existing) |
| `branch_contains` | `merge-base --is-ancestor base branch` | `BranchNotSynced` |
| `merge_would_delete` | which base-tracked files would the merge remove? | `MergeWouldDelete { paths }` |

All three run in `request_acceptance` **before the transaction opens**, together
with the base resolution and the acceptance-plan validation. The delivery branch
must also exist (the merge path checked it; the request path did not).

`merge_would_delete` previews with `merge-tree --write-tree` rather than a
`merge --no-commit` rehearsal: the rehearsal would touch the index and the
working tree of whatever repository the server runs in, which is unacceptable for
a validation that must be side-effect-free.

### Negative matrix (all typed, all before any DB contact)

| Case | Refusal |
|---|---|
| base names no branch | `UnknownBase` (pre-existing) |
| delivery branch names no branch | `UnknownBase` |
| branch does not contain base | `BranchNotSynced` — "merge track into branch first" |
| merge would delete base-tracked files | `MergeWouldDelete` — lists `keep.txt` |
| contained, synced, non-destructive | passes every check and reaches the DB |

Tests: `git::ops::tests::containment_helpers_answer_synced_and_would_delete`
(a scratch repo with synced / unsynced / destructive branches; asserts the
preview moves no ref and changes no working-tree byte) and
`dispatch::tests::request_acceptance_refuses_unsynced_and_destructive_branches`
(all four rows of the matrix above). The existing DB integration fixtures now
create a real delivery branch, so the DB guards they test are reached rather than
the new branch check.

### The defence run against the real repository

```
$ git merge-tree --write-tree track/infra-foundation packet/nimbus/d5-mcp-recall
$ git diff --name-status track/infra-foundation..<tree> | grep '^D'   # empty
$ git merge-base --is-ancestor track/infra-foundation packet/nimbus/d5-mcp-recall  # exit 0
```
Same for the lineage branch. Both branches pass their own defence.

---

## 3. Validation

Commands run on BOTH branch tips (Rust 1.95.0; scratch DB `seatloom_sync3` =
001–012 + seed + 013/014 from track + 015 from `packet/onyx/d5-knowledge-schema` +
016 from `packet/onyx/acceptance-plan-schema` + `.scratch/017_acceptance_cancel.sql`,
in `apply-db.sh`'s canonical order; `seatloom-cli reconcile` run first, as CI does):

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 findings) |
| compile | `cargo check --workspace --all-targets` | PASS |
| core tests | `cargo test -p seatloom-core -- --include-ignored --test-threads=1` | PASS (115 + 3 lineage + 6 D2 + 27 baseline + 3 seed) |
| mcp-emit tests | `cargo test -p seatloom-mcp-emit -- --include-ignored --test-threads=1` | PASS (63 on the recall tip) |
| UI | `pnpm typecheck` + `pnpm test` | PASS (46 tests) |
| sync evidence | `git merge-base --is-ancestor b6751a1 <branch>` | PASS (both) |
| containment | `merge-tree` preview + `diff --name-status` | PASS (no `D` entries, both) |

The 3 `db_baseline` failures seen on a REUSED scratch DB are that suite's
fixed-id isolation artifacts (duplicate `car-test-001` etc.); a fresh-DB run is
green, as the table above records.

---

## 4. PART 3 — the two cards: pending Flux's post-sync verdict

Self-requested: `disp-dXXcgn-OnWkQFGLISoRKa` (commit-pinned, both new tips,
base `track/infra-foundation` @ `b6751a1`). It was **in_flight by
`seat-flux-001`** when this doc was written — not yet green, so the cards were
NOT created (the ruling is explicit: the cards follow the re-verification).

The exact calls, ready to make the moment the verdict lands:

```
request_acceptance(dispatch_id="disp-accplan-lane-b-nimbus",
  branch="packet/nimbus/acceptance-lineage-readmodel",   # tip dfa6ec4
  base="track/infra-foundation",
  how_to_verify=<the packet §3 text, verbatim (Chinese, as Lyra authored it:
  "call the lineage command on acc-disp-me6Nm8e9D6ZRe4fKyVokY and get, in one
  response, the upstream dispatch / event chain / Flux verdict / products /
  dependency edges; a request_acceptance without a plan is a typed refusal with
  zero writes") — reproduced in the packet, not re-worded here, because this
  document must be English per §10 while the tool argument must not be
  paraphrased>,
  flux_verdict_ref=<the post-sync ReviewVerdictIssued event id>,
  acceptance_plan=<docs/coordination/acceptance/2026-10-10-lyra-nimbus-lineage-readmodel-acceptance-plan-v1.md §2, verbatim>)

request_acceptance(dispatch_id="disp-d5-lane-b-nimbus",
  branch="packet/nimbus/d5-mcp-recall",                  # tip e4047f4
  base="track/infra-foundation",
  how_to_verify=<the packet §7 text, verbatim>,
  flux_verdict_ref=<the post-sync ReviewVerdictIssued event id>,
  acceptance_plan=<docs/coordination/acceptance/2026-10-10-lyra-nimbus-d5-recall-acceptance-plan-v1.md §2, verbatim>)
```

Both plans are PO-authored (Lyra) and are carried verbatim from the acceptance
plan docs; the seat does not author product meaning.

**Superseded verdicts (do NOT use these for the cards):** the pre-sync
verifications `disp-accplan-lane-b-nimbus` (Flux delivery
`docs/coordination/tasks/flux/disp-jp7UZhC5oSvosTkwYFKG--delivery.md`, verified
against tip `32a3369`) and `disp-d5-lane-b-nimbus`
(`disp-1dwtHxGOk11gOnU7qWUaA-delivery.md`, verified against tip `db7e1f3`) were
green for the PRE-sync tips. The ruling requires the post-sync re-verification.

---

## 5. Escalations and notes

1. **The dispatch is left in flight.** Everything in the Nimbus lane is done and
   evidenced; the last step (the two cards) is blocked on another seat's verdict,
   and claiming completion without it would violate the done-definition. The
   dispatch therefore stays `in_flight` with this doc as the durable state.
2. **Onyx's 017 may have landed** (Flux verified `disp-acclose-onyx` at
   2026-10-10T12:03Z). When it is on track, `.scratch/017_acceptance_cancel.sql`
   becomes redundant — and the acceptance-cancel path it enables becomes live,
   which is also what lets Aegis settle the dangerous bootstrap card
   `acc-disp-me6Nm8e9D6ZRe4fKyVokY` through the product instead of by hand.
3. **The defence will apply to other seats' cards.** If Onyx's or Mira's card is
   created while their branch does not contain the base, or their merge would
   delete tracked files, the new checks will refuse it. That is the intended
   behaviour; an anomaly there is an escalation, not a reason to weaken the gate.
4. **Pre-existing docs-gate debt on this branch** (unchanged): D2-era docs not in
   `.gitlab/docs-grandfather.txt`. This delivery's doc conforms.
5. `infra/**` untouched — 017 remains Onyx's file; the scratch copy in
   `.scratch/` is where this lane's DDL lives until then.
