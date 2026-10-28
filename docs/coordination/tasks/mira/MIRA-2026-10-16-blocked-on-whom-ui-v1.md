# Delivery: the dependency column's two cases — and the half that is blocked on a field not in this tree

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-blocked-on-whom-ui-v1 |
| status | blocked-environment |
| author | mira |
| date | 2026-10-17 |
| version | v1 |
| dispatch | disp-blocked-on-whom-ui |
| depends_on | LYRA-2026-10-16-first-screen-contract-v1 |
| tags | dependency-column, blocked, dto-fence, delivery, mira |

---

## 0. The one-paragraph version

The dependency column already distinguishes "this landed" from "this has not been read",
and this dispatch tightened the second case so it cannot be misread as "does not exist".
The half that is **not** delivered is the measured existence fact itself: the DTO field that
carries it is on `packet/nimbus/gate-vs-announcement` @ `942da99` and is **not on `track`
and not in this tree**, so rendering it would be rendering a fact nothing computed —
exactly what this dispatch forbids and what `disp-lineage-ui` refused to do this morning.

## 1. What the gate says, and what is actually in the tree

`ev-blocked-on-whom-dto-landed` exists (ArtifactCreated, `seat-nimbus-001`, 2026-10-17), and
Nimbus's own `does_not_assert` is explicit that it does not assert the UI renders the field —
that is this dispatch. Measured in my tree:

| | |
|---|---|
| `src-tauri/src/dto.rs` — `DispatchDependencyDto` | 3 fields: `dispatch_id`, `dep_kind`, `dep_value`. **No `referent`.** |
| `ui/src/lib/types-dto.ts` — same interface | 3 fields, same three. **No referent.** |
| `942da99` an ancestor of `track/infra-foundation` | **no** |
| `942da99` an ancestor of `packet/mira/knowledge-panel` | **no** |
| `942da99` an ancestor of `packet/nimbus/gate-vs-announcement` | yes |

So the gate is satisfied and the capability is absent — the same shape as
`disp-lineage-ui` earlier today. The dispatch anticipated it precisely: *"Here the field does
not exist at all, so a local seam would render a fact nothing computed."*

## 2. What was already there, and what this dispatch added

An earlier dispatch (`disp-qR65V3mi0m-4yYl4J8mmk`, item 2) had already replaced the raw
`dep_kind: dep_value` tooltip with one plain sentence per edge, resolved through
`api.listEvents` and rendered by the **same translator** the supervisor feed uses
(`eventToText`), so the two surfaces cannot drift.

This dispatch's contribution is the distinction the contract's slice 3 asks for:

| case | before | after |
|---|---|---|
| referent **in** the window | `eventToText(event)` — what it was, who, when | `already in the ledger: <sentence>` — the landed case now says it landed |
| referent **not** in the window | "not among the newest 500, cannot tell" | "have not read what it waits for … **this is not a claim that it does not exist** — this screen simply cannot read that far; do not take 'not found' as 'not there'" |
| `dispatch`-kind edge | `waiting on another dispatch (id)` | unchanged |

The middle row is the point. A bounded read (`ORDER BY occurred_at DESC LIMIT $1`) is a positive detector, not a negative one, so absence from the window is absence of evidence. Saying "does not exist" would be the lie shape this project keeps paying for; and saying "cannot tell" without saying WHICH direction it cannot tell in invites the reader to supply the wrong one. So it now says both.

## 3. What is NOT delivered, and why I did not fake it

**The measured existence fact.** Nimbus's `referent: DependencyReferentDto` is what makes
"exists / does not exist" a fact rather than an inference, and it is not in this tree. The
alternatives were:

- **a local `invoke` seam** — forbidden by this dispatch, and correctly: the field does not
  exist, so the seam would compute nothing;
- **asserting non-existence from the bounded window** — a guess wearing a measurement's
  clothes, which is the exact defect the dispatch exists to remove;
- **requiring `backendHydrated`** — wrong fact; the dispatches read is independent of the
  store's hydration, and gating on it would under-claim.

So the column renders what it can measure, names what it cannot, and the missing half is
reported rather than papered over. When `942da99` reaches `track`, the change is additive:
read `dep.referent`, render its three states, and delete the window-based fallback.

## 4. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` | clean |
| `vitest` | **194/194** (2 new) |
| `vite build` | green |
| `cargo test --workspace` | 271 passed / 0 failed (no Rust changed) |
| `lint-changed.sh` | `DispatchInboxView.tsx` **0 → 0**, measured in a `track` worktree at the real path |
| `verify-docs.sh` | this note clean |

The two new tests are the ones that matter: a referent outside the window is **not**
reported as non-existent (and says so in both directions), and a referent inside the window
reads differently from one outside — the distinction the contract's slice 3 asks for.

## 5. Boundaries

No new column, no new partition, no new affordance, no re-layout. `ui/src/lib/api.ts` and
`ui/src/lib/types-dto.ts` untouched — the missing field is Nimbus's fence and was not
worked around. No backend change, no live-database write, no gate event, no merge. No gate
id invented: the dispatch required none from me, so the ledger entry is an `ArtifactCreated`
announcement with no `verified` field.

## 6. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: `emit_event(ArtifactCreated)` — announcement, no `verified`.
- This delivery note.
