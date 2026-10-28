# Flux verdict — the project switcher interaction quality

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-project-switcher-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| tags | reverification, mutation-testing, ux-honesty, keyboard, seat-flux |

- **Verifying dispatch**: `disp-UODw2mn6k1XlpJC49FP2d` (Aegis → Flux, to open the missing acceptance card)
- **Subject dispatch**: `disp-gvVtOC1Z_uvVz8mq-lc45` (Mira)
- **Subject commit**: `4271e57` on `packet/mira/knowledge-panel` — **not merged to `track/infra-foundation`**
- **Verdict event**: **`ev-sMrl5_wXRSDsj_V4qe3gs`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**All five dispatch items are fixed, the sixth problem she found on her own is real, and her
test-quality self-criticism is the most valuable thing in the delivery.** Every mutation she
disclosed reproduces at exactly the kill count she claimed, including the two she reported as
*having been* false greens before she fixed the tests — and I reproduced both false greens to
confirm her diagnosis rather than taking it.

**The single most useful check I ran, which she did not:** I ran her 14 new tests against the
**pre-fix component**. **13 of 14 are red.** That is the negative control that makes the whole
file meaningful, and it is not something she reported.

---

## 1. The five items — each verified

**Item 1, the placeholder.** Her claim that nothing binds a shortcut to open the picker is
correct. I checked both Ctrl+K sites myself:

- `useGlobalShortcuts.ts:70` → `onOpenCommandBar()` with `preventDefault()`
- `App.tsx:366` → toggles the Supervisor IM with `preventDefault()`

Neither opens the ProjectSwitcher, and `ProjectSwitcher` is rendered unconditionally from
`NavRail.tsx` and `TopNav.tsx` with no shortcut path. So the old text invited the operator to
press a combination that did something else. **Her decision to delete the promise rather than
implement the shortcut is right**, and her reasoning is right: implementing it would mean
changing the established L1 Supervisor IM interaction for a search box, which is a trade she
should not make inside a UX dispatch. The placeholder is now "search projects" with no promise.

**Item 2, the visit-time column.** I verified the data claim independently rather than trusting
it:

- `ProjectDto` (`src-tauri/src/dto.rs:25-34`) has only id, name, created_at, two budgets, home_path.
- `grep last_accessed` across `infra/` and `crates/` → **nothing**.
- The live database: `projects` columns are id, name, created_at, worker_budget_tokens,
  supervisor_budget_tokens, parent_project_id, home_path. **No `last_accessed` anywhere in the
  database at all.**

So there was no real data to connect, and `dto-to-v1.ts:160` really did map `createdAt` into
`last_accessed`. **Deleting the column is the only honest option** that does not need a backend
field, and her refusal of the "rename the label" alternative is correct — a timestamp that
cannot answer the operator's question should not occupy that position. The two real counts
(`N inbox items`, `N active sessions`) are kept, which is the right scope.

**Item 3, the heading.** Now "pinned projects", which names only what the sort does. Same disease as
item 1 — the interface promising two things and delivering one.

**Item 4, keyboard navigation.** Present and correct: ArrowDown/ArrowUp with clamping at both
ends, Enter only when highlighted, Escape, Home/End, `scrollIntoView({block:'nearest'})`. Her
four design decisions are each defensible and she gave reasons:

- **nothing highlighted on open** — pre-selecting row 0 would make Enter pick a project the
  operator never looked at. This is the right default and the one most implementations get wrong.
- **clamping, not wrapping** — wrapping loses the operator's place in a filtered list in a way
  that is hard to notice.
- **the listener on the input, bubbling, with no per-row `tabIndex`** — one entry point, no focus
  trap to get wrong.
- **`scrollIntoView`** — arrowing out of the 420px viewport selects something invisible.

None of these is a new interaction paradigm, which respects the dispatch's boundary.

**Item 5, path search.** `contains(project.name, needle) || contains(project.path, needle)` —
verified in the source, and empty/whitespace queries show everything.

## 2. The sixth problem she found — real, and she was honest about its consequence for her own claim

`dto-to-v1.ts:159` is `isPinned: true,` hardcoded. I verified the consequences she listed:

- every project renders as pinned, so the Pin icons are all solid;
- the sort is therefore an identity transform, which means **item 3's fix only corrected the
  heading — the ordering half was and remains impossible under current data**;
- the Pin button writes to memory (`useDataStore`) but the mapper overwrites it on every hydrate.

She wrote that explicitly — "for the dispatch's item 3 I said 'fixed', but really I only corrected the heading" — rather than letting a
partial fix read as a complete one. **That is exactly the disclosure this seat exists to
elicit**, and it is why the verdict is still a pass: the dispatch asked for the heading to say
only what the sort does, and it now does. The pin data gap is a separate, correctly-escalated
problem.

**She was right not to fix it.** A persistent pin field needs backend + schema + migration, which
is another seat's lane, and she said so instead of reaching across.

## 3. Mutations — all seven reproduce at her exact counts

| # | Mutation | Her claim | My measurement |
|---|---|---|---|
| M1 | Ctrl+K back in the placeholder | RED 11 | **RED 11** |
| M2 | visit-time column restored via the real formatter | RED 1 | **RED 1** |
| M3 | heading back to "recently visited and pinned" | RED 2 | **RED 2** |
| M4 | path matching dropped | RED 1 | **RED 1** |
| M5 | arrow navigation dropped | RED 2 | **RED 2** |
| M6 | the `highlighted < 0` guard removed | RED 1 | **RED 1** |
| M7 | empty-result guard removed | RED 1 | **RED 1** |

Two notes on how I ran these:

- For **M2** I restored the column using the *real* `formatShortDateTimeZh` from
  `utils/display`, so the mutation was a genuine reintroduction of the column rather than a
  literal string. The format-based assertion caught it, which is the point.
- For **M7** her description ("empty-result Enter not guarded") is ambiguous about which guard.
  Removing only the empty-list early return left the tests green, because `moveHighlight` still
  clamped. I had to remove **both** the early return and the upper clamp to produce the
  out-of-range read her test targets. Worth her knowing: **her M7 description understates the
  mutation** — the guard that actually prevents the crash is the clamp, and the early return is
  a second layer. Both are needed, and her test does catch the combination.

## 4. I reproduced both of her false greens, and her diagnosis is exactly right

Her most valuable section is the two mutations that were green on her first test version. She
diagnosed it as: `filteredProjects[-1]` is `undefined`, `chosen.id` throws, **React swallows the
exception inside the handler**, so `onSelect` is not called either way — the guard working and
the guard crashing look identical to an assertion that only observes one side effect.

**I verified this rather than accepting it.** For M6 I removed the guard *and* removed only the
`expect(thrown).toEqual([])` assertion: **the test stayed green.** Same for M7. So her diagnosis
is correct, and her fix — assert the absence of an unhandled error via a `window` error listener,
plus assert the picker is still open — is the right one and not a loosening of the assertion.

This is her second encounter with the same shape (the earlier one was the IME test picking the
wrong target). Her generalisation is the right one: **an assertion that observes a single side
effect whose value is identical on the correct path and the silent-failure path cannot tell a fix
from a crash.** I am recording it as a forward record because it generalises well beyond this
file.

## 5. The negative control she did not run

I checked out the **pre-fix** `ProjectSwitcher.tsx` and ran her new test file against it:

```
13 failed | 1 passed (14)
```

Thirteen of her fourteen tests are red on the code that had the bug. The one that stays green is
`keeps the counts that ARE real`, which is correct and intentional — those counts existed before
and after, so it is a non-regression guard rather than a fix detector.

**This matters** because it is the difference between "the tests pass" and "the tests are about
something". A file where every test passes on both the broken and the fixed code proves nothing;
here the whole file except one deliberately-unchanged assertion collapses on the old component.

## 6. Gates — every figure exact

- Her file: **14 passed**.
- Full UI suite: **314 passed / 25 files / 0 errors**. Baseline (`4271e57~1`) is **300 passed /
  24 files**. The delta is exactly her 14 — the arithmetic matches her claim.
- `tsc --noEmit` clean.
- **eslint: 143 problems before AND after**, diffed against `4271e57~1` on the whole `src` tree.
  Her "identical to baseline, 0 on my two files" claim holds.
- **prettier**: both her files are clean *after* the change — and the pre-fix
  `ProjectSwitcher.tsx` was itself prettier-dirty, so she also cleaned up pre-existing debt in a
  file she was already rewriting. Her note that she diffed line-by-line after `--write` to
  confirm it only touched her own code is the right discipline.
- `vite build` succeeds.

## 7. What I did not do

- Did **not** change the M7 mutation description, the clamp, or anything else in her files. The
  observation in §3 is reported, not applied.
- Did **not** fix `isPinned: true`. It needs a backend field and is another lane; she escalated
  it correctly and I am escalating it again rather than acting.
- Did **not** merge. `4271e57` is not on `track`.
- Did **not** touch Mira's worktree beyond a detached worktree at `4271e57`, removed afterwards;
  her `git status` is empty.
- Did **not** verify by eye in a running app — the dispatch's criteria are all DOM-observable and
  I verified them through the real component. Real-machine click-through remains the operator's.
- Did **not** write to the live database (read-only `information_schema` queries only).

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/sw 4271e57
ln -s <mira>/ui/node_modules /private/tmp/sw/ui/node_modules
cd /private/tmp/sw/ui

./node_modules/.bin/vitest run src/components/ProjectSwitcher.test.tsx            # 14 passed
./node_modules/.bin/vitest run                                                  # 314 / 25 files
./node_modules/.bin/tsc --noEmit                                                # clean
./node_modules/.bin/eslint src                                                  # 143 (same as 4271e57~1)
./node_modules/.bin/prettier --check src/components/ProjectSwitcher.tsx \
    src/components/ProjectSwitcher.test.tsx                                     # clean
./node_modules/.bin/vite build                                                  # ok

# THE DATA CLAIMS, VERIFIED:
#   ProjectDto fields      -> id, name, created_at, 2 budgets, home_path (no last_accessed)
#   grep last_accessed infra/ crates/  -> nothing
#   live DB projects columns -> id, name, created_at, worker_budget_tokens,
#                               supervisor_budget_tokens, parent_project_id, home_path
#   dto-to-v1.ts:159      -> isPinned: true   (hardcoded)
#   dto-to-v1.ts:160      -> last_accessed: d.createdAt

# THE NEGATIVE CONTROL SHE DID NOT RUN:
git checkout 4271e57~1 -- ui/src/components/ProjectSwitcher.tsx
./node_modules/.bin/vitest run src/components/ProjectSwitcher.test.tsx   # 13 failed | 1 passed
git checkout HEAD -- ui/src

# MUTATIONS (each reverted, tree verified identical afterwards) — all match her claims:
#   Ctrl+K back in the placeholder            -> RED 11
#   visit-time column restored (real fmt)     -> RED 1
#   heading back to recently-visited-and-pinned -> RED 2
#   path matching dropped                     -> RED 1
#   arrow navigation dropped                  -> RED 2
#   the highlighted<0 guard removed           -> RED 1
#   empty-result guard (early return + clamp) -> RED 1

# HER TWO FALSE GREENS, REPRODUCED:
#   M6 with the guard removed AND expect(thrown).toEqual([]) removed -> GREEN
#   M7 likewise                                                      -> GREEN
#   i.e. React swallows the in-handler throw, so onSelect is uncalled either way.
```

## Verdict event

`ev-sMrl5_wXRSDsj_V4qe3gs`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-gvVtOC1Z_uvVz8mq-lc45` in the payload.

## Files I did not change

No source file. Seven mutations were applied and each reverted, with the tree confirmed
byte-identical to `4271e57` after each. One detached worktree was created at Mira's packet branch
and removed afterwards. My probe files were deleted.
