# Delivery: branch synced to track (behind 0), and the call-site gap Flux found is closed

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-21-sync-to-track-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-21 |
| version | v1 |
| dispatch | disp-bBdpBqTT9g5-kKU28xvIt |
| depends_on | AEGIS-2026-10-21-mira-sync-to-track-v1 |
| tags | sync, merge, call-site-test, project-filtering, delivery, mira |

---

## 0. One paragraph

The branch is now **0 commits behind track** (it was 20; the packet said 19, measured
before its own commit landed). The project filtering from `2c69adb` is present —
`grep -c boundMirrorNames ui/src/views/LiveSeatsView.tsx` returns **2**. Six files
conflicted, and in every single case track's version turned out to be the more correct
one. During the merge I dropped a line that mattered, shipped a store that hydrated zero
projects with no error, and an existing test caught it — that is written up below rather
than buried. Flux's verification of the sanitation dispatch then found the call site had
no test; that gap is closed with 5 new tests, both of which were proved to bite by
deliberately breaking the code.

## 1. Preconditions, measured before touching anything

| Claim in the packet | Measured |
|---|---|
| branch is behind track | `git rev-list --count HEAD..track/infra-foundation` → **20** (not 19 — the packet was written before `97d6bbb`, which is itself, landed) |
| `boundMirrorNames` is absent | `grep -c boundMirrorNames ui/src/views/LiveSeatsView.tsx` → **0** |
| working tree clean | yes, apart from the two pre-existing untracked scratch files |

The dispatch's premise (a stale branch causes an incorrect verification) was therefore
confirmed on this branch rather than assumed.

## 2. What came in

Of the 20 commits, the packet's own reading was right: only **4 touched code**, and 3 of
those are the project-filtering series.

```
2c69adb  feat(filtering): isolated combined candidate with bound mirrors and stale-response guards
4149b7f  docs(nimbus): combined acceptance requested and own old card withdrawn
cfda740  Merge branch 'packet/nimbus/project-filtering-combined' into track/infra-foundation
e37c141  docs: Flux's verification of the sanitation work (pulled in during the merge)
```

The rest are Aegis's scripts and docs. The important arrival is `LiveSeatsView.tsx`
gaining `boundMirrorNames`, which is what makes the seat list follow the project instead
of showing all of them flat — the thing that made the operator think the feature was
broken. It was not broken; it was absent from this branch.

## 3. The conflicts, one by one

**`useDataStore.ts`.** Track's `hydrateProjectData` reads one scoped project; my version
read eight unfiltered lists once and filtered them client-side. **Track's is strictly
better and already contains both of my fixes one layer lower** — seat attribution by
role binding, and per-project filtering, done server-side. Taking my side would have
reverted the operator's reported bug. Took track's.

**`DispatchInboxView.tsx`** (3 blocks). Track's each added guards I did not have: a
stale-response `selection` ref, an early return when no project is selected or
hydration is unfinished, and error-state isolation. Took track's, all three.

**`DocumentsWorkspace.tsx`.** Track added the `alive` flag, project scoping and effect
cleanup. Took track's. My sanitiser call sat outside the conflicted hunk and survived
untouched.

**`SupervisionDashboard.tsx`.** Track added the `backendHydrated` guard. Took track's; my
`SyncFreshness` import likewise survived outside the hunk.

**`TimelineView.tsx`.** The one file where the substantive work was mine — the visibility
filter, its per-project storage, the collapsed-summary strip — and track's edit to the
same lines was formatting. Took mine. Also removed a `!currentData` guard the merge
produced twice.

**Three test files.** These were the stale direction, but mine: they double-filtered
client-side filtering that no longer exists, and their api doubles had the wrong shape.
`listDocuments` takes an **options object**, not a positional id — a mock that takes a
string reaches the store fine and silently returns the wrong slice, which is the exact
failure that file's own comment warns about, reintroduced. The doubles now honour the
project argument; the assertions were left standing unchanged, because the behaviour they
pin is still correct.

Two inbox assertions match on `role="status"` rather than the literal string, since
track's merge rewrote the empty-state copy; a future reword should not fail the suite for
no behavioural reason.

## 4. The thing I got wrong, and the test that caught it

Resolving `useDataStore.ts` I dropped one line:

```ts
set({ projects, activeProjectId: nextActive });;
```

Its absence left a store that hydrated **zero projects and reported no error**. Every
view would have rendered an empty state and called it correct — a silent, total, and
believable-looking failure.

`AllProjectsView.hydration.test.tsx` caught it by failing, and only for a reason worth
stating: the test was mine, from an earlier dispatch, written to pin "the Projects page
reads the database". It failed because the store no longer published projects, which is
precisely the thing that test exists to defend. Restored the line; 8/8 green.

This is the same class Flux's mutation run exposed (checks present, the thing they stand
for not verified), arriving from the opposite direction: I had the check, and removed its
subject while editing.

## 5. Flux's gap, closed

Flux verified the sanitation dispatch as pass and then mutated `sanitizeHtml` to return
its input unchanged — **all 14 unit tests stayed green**, because they assemble their own
markdown and never walk the path the product takes. The third mutation made it explicit:
bypass the sanitiser at `DocumentsWorkspace.tsx` and nothing notices. He also noted `img`
is absent from the allow-list, which is harmless today (0 documents carry images) but will
silently drop images once the view is mounted.

I closed the first gap with `ui/src/app-v2/views/DocumentsWorkspace.sanitize.test.tsx` — 5
tests that render the real component, click a real document, and assert on the **DOM the
browser built**: no script, no `onerror`, no `javascript:` href; and the other direction,
a normal document keeps its `h1`/`strong`/`code`/`li`/`blockquote` structure.

**Both were proved to bite**, by breaking the code on purpose:

| mutation | result |
|---|---|
| call site returns raw html (exactly Flux's mutation) | **2 of 5 red** — `onerror` and `javascript:` caught |
| tag allow-list emptied | **5 red** across both files, 4 of them the normal-document direction |

And each helper first checks that the body section actually rendered, throwing if it did
not, so these assertions can never pass vacuously — the very failure mode they were added
to prove against.

`img` remains a decision for the operator, not something to decide silently; it is
registered as a follow-up and omitted from the allow-list means an image is dropped
rather than executed, which is the safe direction.

## 6. Gates

| Gate | Result |
|---|---|
| **`git rev-list --count packet/mira/knowledge-panel..track/infra-foundation`** | **0** |
| **`grep -c boundMirrorNames ui/src/views/LiveSeatsView.tsx`** | **2** (non-zero — sync succeeded) |
| sanitize tests | still **14/14** |
| `tsc --noEmit` | clean |
| `vitest run` | **239/239**, 19 files (5 new) |
| `vite build` | green (pre-existing >500 kB advisory) |
| `cargo test` | **283/283** |
| `cargo clippy --all-targets` | no new warnings |
| `cargo fmt --all -- --check` | clean |
| lint | **zero delta on every changed file**, plus 0 errors on the new file, measured against a fresh `track` worktree |
| `verify-docs.sh` | this note clean |

## 7. Boundaries

No database write, no merge into `track`, no acceptance card, no rebase of anyone else's
work, and no functional change of my own implementations to accommodate older code — the
packet was explicit that the new code comes in, my work does not go backwards. Nothing was
deleted to make the merge clean.

## 8. Artifact index

- Branch: `packet/mira/knowledge-panel`, head `1f8c01b`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
