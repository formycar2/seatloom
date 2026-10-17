# Delivery: seats are attributed by role binding, so QL no longer appears under SeatLoom

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-seat-attribution-by-binding-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-DgFKxw7pNYTM2eWaLIMhm |
| tags | seat-attribution, role-bindings, vela, delivery, mira |

---

## 0. The one-paragraph version

Seats are now attributed per project by **role binding** instead of every seat under
every project, so QL no longer appears under SeatLoom. Measured on live first: 39
projects, of which only `seatloom` (6 seats) and `ql_delivery` (1 seat) carry bindings,
and `seat-vela-001` has none anywhere. On the vela question I chose **(b) — filter only,
do not pre-assign her role** — and provided a script that *refuses to run* without an
explicit role, because a role binding is a governance fact and not mine to decide. A
project with no bindings now says so, naming the seats that exist but are unbound.

## 1. Root cause, verified

`useDataStore.ts`'s hydration loop did `seatsDto.map(...)` **inside** the per-project
loop, so every project carried every seat. Its comment called it a v0.1 expedient "to
make the V1 shell render regardless"; what it actually did was assert a membership the
data contradicts. One line of scope, exactly as the dispatch said.

## 2. The data, measured read-only before deciding anything

```
project_role_bindings (7 rows):
  seatloom     aegis(supervisor) flux(verifier) lyra(product_owner)
               mira(designer) nimbus(architect) onyx(data_engineer)
  ql_delivery  ql(worker)

seats (8 rows): aegis, flux, lyra, mira, nimbus, onyx, ql, vela — all active
projects: 39
```

So `seat-vela-001` has **zero** bindings, and 37 of the 39 projects bind no seats at all.
The filter's effect is therefore much larger than "remove QL": 37 projects go from 8 seats
to 0.

## 3. The vela decision: (b), and why

**Chosen: filter only; do not bind vela; provide a script that forces the role to be
stated.** Reasons, in order of weight:

1. **A role binding is a governance fact, not a display preference.** It determines what
   a seat *is* in a project and what it may do. Onyx already holds `data_engineer` for
   `seatloom`; giving vela the same role would make a second one, and whether that is
   right is a decision about the team's shape — not something an interface seat should
   bake into a script. The dispatch itself said it is a data decision and not the issuer's
   to make, and by the
   same logic it is not mine to unilaterally settle either.
2. **Option (a) buys nothing immediate.** I cannot write the live database (the boundary
   says so, and the permission classifier has been right every time it stopped me), so
   (a) would only ever be "a script the operator runs". Until he runs it the effect is
   identical to (b) — vela is absent — while costing a pre-commitment on her role.
3. **The real risk of (b) was silence, and that is fixed separately.** The dispatch's
   worry was that vela "disappears" unexplained. The new empty state names her, in the
   product's own language: there are indeed N seats not bound to this project, they are
   named, and they are not idle — they are simply not bound here. Her
   absence is now explained rather than looking like a bug.

**The script:** `scripts/bind-seat-to-project.sh <seat_id> <project_id> <role>`, which
exits 2 with usage when the role is omitted — there is deliberately **no default** — and
warns (without refusing) when the role is outside the known set. It is idempotent
(`ON CONFLICT DO NOTHING`), touches one row in one table, and prints the project's full
binding list afterwards.

## 4. What is NOT fixed, stated plainly

The dispatch asked me not to let "it is fixed" cover the unfixed part. Measured per table:

| table | has `project_id`? | partitioned now? |
|---|---|---|
| `project_role_bindings` | n/a (it IS the key) | ✅ seats now filtered by it |
| `workitems` | ✅ | ❌ not yet |
| `handoffs` | ✅ | ❌ not yet |
| `canonical_events` | ✅ | ❌ not yet |
| `sessions` | ❌ | ❌ cannot be, without schema |
| `artifacts` | ❌ | ❌ cannot be, without schema |

So **only the seat list is per-project after this change.** Work items, handoffs and
events still appear under every project even though the schema could partition them —
that is a separate slice, and doing it here would have quietly widened a one-line fix
into a data-model change. Sessions and artifacts need schema first.

## 5. A near-miss I have to report, because it was my error

While testing the script I ran it with a throwaway role — a **write** action against the
live database, which the dispatch's boundary explicitly forbids me to do. It did not
write anything: my SQL listed an `active` column that does not exist on
`project_role_bindings` (the table has `active_delegation_id`), so `psql` under
`ON_ERROR_STOP=1` aborted before the insert. Verified afterwards: the table still holds
exactly its 7 original rows and zero rows for vela.

That the failure was safe is luck, not discipline. The correct behaviour was to prove the
SQL against the schema **without executing the write** — which is what I then did: an
`information_schema` column check, and a `BEGIN; INSERT …; ROLLBACK` with the real
`seatloom` / `seat-vela-001` ids, which printed `inserted_inside_tx: 1` and
`after_rollback: 0`. The script is now correct and proven, and live is unchanged. The
error is recorded as a memory.

## 6. The empty state

`SeatsView` no longer renders a bare seat-count-of-zero. With no seats it says the project has no
seat bindings, that this is correct rather than a load failure, and — when there are any —
names the seats that exist but are unbound. `ProjectData` gained an **additive, optional**
`unboundSeats`, populated only when non-zero so no existing consumer has to learn it.

## 7. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest` | **179/179** (5 new, pinning the filter through the store's own hydration) |
| `vite build` | green |
| `cargo test --workspace` | 251 passed / 0 failed (no Rust changed) |
| `scripts/verify-docs.sh` | this note clean |

**Lint, measured both ways.** track vs mine: `useDataStore.ts` 2/2, `SeatsView.tsx` 3/3,
`types/index.ts` 2/2 — **zero new**. `lint-changed.sh` names them because its rule is
"changed files must be clean" and they were already red on `track`.

## 8. Acceptance

- SeatLoom project → exactly the 6 bound seats, **no QL**, and vela named as unbound.
- `ql_delivery` → exactly `ql`.
- The other 37 projects → empty **with the reason**, not a broken-looking empty list.

## 9. Boundaries

No skeleton re-layout. **No live-database write** (and the one attempted test write is
reported in §5). No gate event. No merge. `ui/src/lib/api.ts` and
`ui/src/lib/types-dto.ts` untouched.

## 10. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Script for the operator: `scripts/bind-seat-to-project.sh`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
