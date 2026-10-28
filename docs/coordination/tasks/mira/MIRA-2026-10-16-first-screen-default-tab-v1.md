# Delivery: the dispatch page is now the default landing

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-first-screen-default-tab-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-first-screen-default-tab |
| depends_on | LYRA-2026-10-16-first-screen-contract-v1 |
| tags | first-screen, default-tab, delivery, mira |

---

## 0. The one-paragraph version

`ui/src/App.tsx`'s initial tab changed from `'live-seats'` to `'dispatches'` — one line of
code. The mirror keeps its NavRail entry and still attaches a seat, the ⌘K chord layer is
untouched, nothing else moved. Measured: zero new lint issues in a file that was already
red, and the error set is byte-for-byte the same as `track`'s.

## 1. The change

The code line, before and after (the surrounding comment lines are the replacement for the
old comment, quoted in §1's prose rather than reproduced here):

```diff
-  const [activeTab, setActiveTab] = useState<Tab>('live-seats');
+  const [activeTab, setActiveTab] = useState<Tab>('dispatches');
```

**One line of code.** The other five lines replace a comment that would otherwise have said
the opposite of what the code does — "open on the live mirror, the one surface that proves
itself" would have been false the moment the line changed. A stale comment that contradicts
its own code is the misleading-comment failure this project has already paid for, so it was
replaced rather than left. The replacement records the operator's own words for what the
first screen is, why the dispatch page already answers it, and that the mirror keeps its
place as a detail view. The contract forbids adding a partition, a column, a component, a
route or a visual treatment; a comment is none of those.

## 2. The three proofs, each a measurement

**Proof 1 — opening the app lands on the dispatch page.**
`useState<Tab>('dispatches')` at `App.tsx:96`, and nothing persists `activeTab`: the only
`localStorage` keys `App.tsx` touches are `seatloom.supervisor.open` (get and set). So the
default applies on every open, not just the first. `case 'dispatches'` in the render
switch returns `<DispatchInboxView />` (`App.tsx:434`), which is the real table — 177
dispatches, live statuses, real dependency edges, polled every 5 s.

**Proof 2 — the mirror tab is still in the NavRail and still attaches a seat.**
`NavRail.tsx:70` still lists the live-seats entry and `:75` the dispatch entry; both are
present and neither was renamed or reordered. The mirror's attach path is untouched by
this change — `git diff --stat` over `LiveSeatsView.tsx` and
`useLiveSeatsMirrorStore.ts` is empty. It keeps its entry as the "drop into a seat's raw
terminal" detail view, which is what the operator asked for when he said the mirror is
transitional.

**Proof 3 — the ⌘K panel still opens and still sends, and the chord layer is untouched.**
Zero lines mentioning `metaKey` / `ctrlKey` in the diff. `useGlobalShortcuts` and the
⌘K bindings are not in the changed file's diff at all. The known ⌘K double-binding is
still an open ruling with Aegis and was deliberately left alone.

**Nothing else moved.** `git diff --stat` is one file, 6 insertions, 2 deletions.

## 3. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest` | 163/163 |
| `vite build` | green |
| `scripts/lint-changed.sh` | names `App.tsx` — see below |
| `scripts/verify-docs.sh` | this note clean |

**The lint number, measured both ways rather than asserted.** In a `track` worktree at the
real paths, `src/App.tsx` has **15 eslint errors**; in my tree, the same file has **15**.
The error *sets* are identical (`sort | uniq -c` diff is empty), so zero new issues.
`lint-changed.sh` still names the file because its rule is "changed files must be clean"
and `App.tsx` has been red since before this dispatch. Closing 15 pre-existing errors in a
file this dispatch changes by one line is out of scope, and the contract's instruction is
to leave already-dirty files exactly as dirty.

## 4. What is not proven

Nobody has opened the built app. The proofs above are that the default is `'dispatches'`,
that nothing overrides it, that the mirror's entry and attach path are intact, and that the
chord layer is untouched. What is unverified is the operator's own first look — whether the
dispatch page reads as the front door he described. That is what the acceptance card asks
him, and it is the click-level check the contract assigns to him rather than to a gate.

## 5. Boundaries

No re-layout. No new navigation, component, route or visual treatment. The mirror tab was
not deleted or hidden. The ⌘K layer was not touched. `ui/src/lib/api.ts` and
`ui/src/lib/types-dto.ts` were not touched. No backend change, no live-database write, no
merge.

## 6. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: `ev-first-screen-default-tab-landed` (ArtifactCreated, announcement — no
  `verified` field, per §6.1).
- This delivery note.
