# Acceptance: Nimbus runtime triple refactor (Rust side) — and runtime-triple close-out

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance |
| id | acceptance-2026-09-30-nimbus-runtime-triple |
| status | **ACCEPTED** |
| author | lyra |
| date | 2026-09-30 |
| version | v1 |
| verdict | **ACCEPTED** — with one boundary exception ruled below |
| packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1.md` |
| **accepted commit** | **`92234bcfe8800abd139dc408254fa5ba3feda4fa`** |
| merge commit | `5da7b8b` into `track/infra-foundation` |
| verification | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-nimbus-verification-delivery-v1.md` (PASS) |
| companion | `docs/coordination/acceptance/2026-09-30-lyra-onyx-runtime-triple-migration-acceptance-v1.md` |
| tags | acceptance, nimbus, runtime-triple, close-out |

---

## 1. Verdict

**ACCEPTED** at `92234bc`, merged as `5da7b8b`. **The runtime triple is now complete on
`track/infra-foundation`** — schema and Rust both landed.

`Runtime` no longer exists. `grep -rn "enum Runtime"` returns nothing; the 7 remaining textual hits
are comments naming the former enum, which is what the packet allowed.

## 2. What Flux proved, beyond re-running the commands

### 2.1 The parse boundary is single, and its negative path actually works

One generic helper, parsing purely to validate. Flux investigated a construct that *"read in
isolation is exactly the convenience fallback you told me to look for"* and established it is not
one. Negative tests pass by name:

```
objects::seat::tests::unknown_role_is_a_typed_error_not_a_custom_fallback ... ok
objects::session::tests::unknown_agent_is_a_typed_error_not_a_custom_fallback ... ok
```

Typed error, no panic, no fallback, casing significant (`"stepcode"` is rejected).

### 2.2 Serde wire shape read directly, requests included

Flux read `src-tauri/src/dto.rs` itself rather than trusting Nimbus's excerpt. This is the rule the
project already learned the hard way: `tsc` passing proves the TS types agree with each other, not
that they agree with the Rust wire format.

### 2.3 Nimbus's corrections to *my* packet — all four independently confirmed

Flux re-measured each. All correct. See §4.

## 3. Boundary ruling — `ui/src/lib/dto-to-v1.ts`

Flux escalated rather than waving through, which was right.

The packet's TS allowance read *"type declarations under `ui/src/**/*.ts` … only where
`tsc --noEmit` forces a type-only fix."* `dto-to-v1.ts` carries a one-line **logic** change:

```diff
-    runtime: coerceRuntime(d.runtime),
+    runtime: coerceRuntime(d.agent),
```

**Ruled: ACCEPTED as a named exception.** Reasoning:

1. **Forced.** `d.runtime` no longer exists on `SessionDto`; `tsc` fails without it. The only
   alternative was leaving the build red.
2. **Semantically correct.** Migration `009` backfills `agent` from `runtime`, so `agent` is the
   field carrying the old meaning. Mapping anything else would have been wrong.
3. **It preserves behaviour rather than adding surface.** The V1 display model still shows a single
   `runtime`; this keeps it showing the same thing. Surfacing `harness`/`model` would have been the
   actual Option-B violation, and Nimbus explicitly deferred that to L2 with a comment saying so —
   the L1-first rule working as intended.
4. **The binding boundary held.** Option B's declared constraint was zero `.tsx` files, and
   `git diff --name-only d70a5e0..92234bc | grep '\.tsx$'` is empty across the whole delta.

**My packet wording was the defect here, not the change.** "Type-only" was too narrow for a
refactor that removes a field the UI reads. Future packets should say: *minimal forced changes that
preserve existing behaviour and add no new surface.*

## 4. Corrections to my own packet — confirmed, and now erratum'd

Nimbus re-measured every §1.3 row with `git show 9599136:<path>`. Two were wrong, both from reading
a **dirty working tree** — the same defect class as Onyx drift D-1. Flux confirmed all four.

| # | Claim in my §1.3 | Truth at `9599136` |
|---|---|---|
| 9.1 | `db/models.rs:52` is `runtime: String` | it is `:53`; `:52` is `seat_id` |
| 9.2 | seed `:40-45` → six role values incl. `data_engineer` | `:40-45` is the delegation block; role rows are `:33-37`, **five**, no `data_engineer` |
| 9.3 | `.seatloom/` holds only `bootstrap/` + `transcripts/` | also `seat-sessions.tsv` now; load-bearing part (no `seats/`, no `delegations/`) still holds |
| 9.4 | §2.6 construction-site list | incomplete |

**§9.2 is material** and I want how Nimbus handled it on the record. My §2.3.3 justified adding
`SeatRole::DataEngineer` *because* "the seed already writes `data_engineer` at `seed/001…:45`" — at
the declared base commit, it does not. Nimbus did not work around it and did not drop the variant.
It kept `DataEngineer` on the independent evidence (memory addendum decision 3; `18a69fb`, now an
ancestor, which does write it at `:45`), corrected the citation in code comments rather than
reproducing it, and added `every_seeded_role_value_parses` asserting all six parse. That is a seat
catching an error in its own dispatched packet and handling it on the merits — the behaviour the
rules want.

An erratum block is appended to both runtime-triple packets so no P0 packet inherits the bad table,
per Flux's explicit recommendation.

## 5. Runtime triple — close-out summary

| Half | Commit | Merge | Gates |
|---|---|---|---|
| Onyx — schema 009 | `d4f40b5` | `80be8b5` | HOLD → FAIL → PASS |
| Nimbus — Rust refactor | `92234bc` | `5da7b8b` | PASS |

Four gates, three of which caught something real: a working tree that could not be pinned, a CI-path
regression from a cross-packet collision, and an ordering heuristic that had to become a rule.

## 6. Open items carried forward

| # | Item | Owner |
|---|---|---|
| 1 | `project_id` on seed 001 session tuples — `apply-db.sh` is **not** idempotent today | Onyx |
| 2 | `verify-postgres-baseline.sh` to the `run`/`rm`/`volume rm` form (D-5 remainder; podman has no compose provider) | Nimbus + Flux |
| 3 | Host `psql` client, or an explicitly supported containerised path — `apply-db.sh`'s own invocation path remains unexercised | Mr. Zhang |
| 4 | Destructive baseline reset — **risk reduced**: pid 91929 is gone and nothing is attached to the database now | Mr. Zhang |
| 5 | Proper ordered-migration runner + separate idempotent seed | Onyx |
| 6 | `seat-dispatch.sh` misattributes every dispatch as "AEGIS dispatch" | Aegis |
| 7 | Packet wording: "type-only" → "minimal forced, behaviour-preserving, no new surface" (§3) | Lyra |

---

*Lyra · 2026-09-30 · runtime triple COMPLETE on track/infra-foundation · Onyx `80be8b5` + Nimbus
`5da7b8b` · boundary exception ruled in §3 · my own §1.3 errors erratum'd in §4*
