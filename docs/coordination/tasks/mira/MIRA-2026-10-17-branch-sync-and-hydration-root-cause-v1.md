# Delivery: branch synced to `track/infra-foundation` @ `af0275a` — and the root cause of the unreadable database, verified end to end

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-17-branch-sync-and-hydration-root-cause-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-17 |
| version | v1 |
| dispatch | disp-rDZk_yTbmdQMY8HeFc0Hg |
| tags | branch-sync, hydration, root-cause, delivery, mira |

---

## 0. The one-paragraph version

`packet/mira/knowledge-panel` is synced to `track/infra-foundation` @ `af0275a` — a clean
merge, no conflicts, all gates green. While doing it I verified, end to end and with
evidence, the root cause the previous dispatch could not reach: **`seats.default_agent =
'Pi'` on two rows fails `Agent::from_str`, so `cmd_list_seats` errors, so the eight-read
`Promise.all` in `hydrateFromBackend` rejects** — which is why the store silently kept its
three fake project cards. One half of that is already fixed on `track`; the other half
(`1547219`, adding `Pi` to `enum Agent`) is on Nimbus's branch and **not on track**, so the
acceptance card's precondition is still unmet.

## 1. The sync

```
git switch packet/mira/knowledge-panel   (already on it)
git merge --no-edit track/infra-foundation
```

Result: `43eed6b Merge branch 'track/infra-foundation' into packet/mira/knowledge-panel`,
`track/infra-foundation` @ `af0275a` is now an ancestor. **No conflicts.** The merge
brought in a large amount of other seats' work, so every gate was re-run rather than
assumed.

| Gate | Result |
|---|---|
| `pnpm tsc --noEmit` | clean |
| `pnpm vitest run` | **187/187** |
| `pnpm vite build` | green |
| `cargo test --workspace` | **271 passed / 0 failed** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 issues |
| `cargo fmt --all -- --check` | 0 diffs |
| `scripts/lint-changed.sh` | 8 files named, **all zero-delta** (below) |
| `scripts/verify-docs.sh` | 2 violations, both the pre-existing untracked scratch files, not this dispatch's |

**Lint delta, measured both ways in a `track` worktree at the real paths** — because the
merge changed the baselines, every number was re-measured rather than carried over:

| file | track | mine | delta |
|---|---|---|---|
| `src/App.tsx` | 15 | 15 | +0 |
| `src/app-v2/dashboard/ProjectDashboard.tsx` | 33 | 33 | +0 |
| `src/components/AddSeatDialog.tsx` | 1 | 1 | +0 |
| `src/components/DelegationOverlay.tsx` | 2 | 2 | +0 |
| `src/components/HandoffForm.tsx` | 1 | 1 | +0 |
| `src/stores/useDataStore.ts` | 2 | 2 | +0 |
| `src/types/index.ts` | 2 | 2 | +0 |
| `src/views/SeatsView.tsx` | 3 | 3 | +0 |

## 2. The root cause, verified — the thing the last dispatch could not reach

The previous dispatch asked me to find out why hydration failed and I could not observe
the running app. Aegis's dispatch named the cause; I verified every link rather than
taking it on trust, and it holds completely:

| # | Link | Evidence |
|---|---|---|
| 1 | two seat rows spell `default_agent` as `Pi` | `SELECT … FROM seats` → `seat-ql-001` and `seat-vela-001` both `Pi`; the other six are `ClaudeCode` |
| 2 | `enum Agent` has no `Pi` variant, and `Pi` does not match the `Custom(...)` tagged form | `objects/session.rs:63` — six variants; `parse_custom_tagged("Pi", "Custom")` returns `None` because it does not start with `Custom(` |
| 3 | so `Agent::from_str("Pi")` returns `Err` | `impl FromStr for Agent`, `session.rs:186` |
| 4 | and the seat read validates that column against the enum | `repositories.rs:2879 check_seat_domains` → `check_domain::<Agent>("seats", id, "default_agent", …)`, called from `row_to_seat` at `:2910` |
| 5 | so `cmd_list_seats` returns `Err` for the **whole table** | `row_to_seat` returns `Result`, and one bad row fails the query |
| 6 | so `hydrateFromBackend`'s `Promise.all` of eight reads rejects | `useDataStore.ts` |
| 7 | and the error was stored in `backendHydrateError` — which **nothing rendered** | grep of the whole UI: `backendHydrated` feeds the provenance badge; `backendHydrateError` has no reader |

Link 7 is why this survived from the day Vela was registered. The three fake project cards
were not a rendering bug; they were a silently-swallowed database error standing next to a
hardcoded fallback.

**This is also the proof that last dispatch's fix was the right one.** `HydrationFailure`
renders exactly this error verbatim, so the next occurrence cannot hide. It could not have
been found by reading the app from a shell — only by making the error visible.

## 3. What is still blocking the acceptance card, stated precisely

`track` already carries `367007a`, which fixed **one** of the two failing columns:
`default_harness` is now `Stepcode` on all eight rows (measured). `default_agent` is still
`Pi` on two rows, and `check_domain::<Agent>` still rejects it — so **`cmd_list_seats`
still fails on my branch even after this sync.**

The remaining half is `1547219 feat(objects): Pi joins enum Agent`. Measured: it exists and
is an ancestor of `packet/nimbus/gate-vs-announcement`, and is **not** an ancestor of
`track/infra-foundation` or of this branch. Until it reaches `track`, the app on this
branch still cannot read the database, and the operator clicking the card would see the red
notice rather than 39 projects.

That matches what Aegis said, and I am recording it with the evidence so nobody has to
take either of our words for it. Handling it is Aegis's and Nimbus's, not mine.

## 4. Boundaries

No live-database write. No gate event. **No merge into `track`** — the sync is the other
direction, as instructed. `ui/src/lib/api.ts` and `ui/src/lib/types-dto.ts` untouched by
me.

## 5. Artifact index

- Branch: `packet/mira/knowledge-panel` @ `43eed6b`, containing `track/infra-foundation` @
  `af0275a`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
