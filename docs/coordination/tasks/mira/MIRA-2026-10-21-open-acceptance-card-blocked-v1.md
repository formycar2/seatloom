# Delivery: no card could be opened — two blockers, both measured, neither mine to remove

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-21-open-acceptance-card-blocked-v1 |
| status | blocked |
| author | mira |
| date | 2026-10-21 |
| version | v1 |
| dispatch | disp-BOictOYRYxM64GEQgKEoX |
| depends_on | AEGIS-2026-10-21-mira-open-acceptance-card-v1 |
| tags | blocked, acceptance-card, merge-guard, verdict-missing, delivery, mira |

---

## 0. One paragraph

I could not open a card. Two separate things block it, and I want to be precise that
**neither is a missing step of mine**: (1) the merge-containment guard refuses every card
on this branch because the branch deletes one file that exists on track, and that
deletion is deliberate documented work rather than an accident; (2) four of the five
dispatches the packet wants cards for **have no independent verdict at all**, and the
guard requires one — so even without (1) four cards could not be opened. No card was
opened and nothing was weakened to get one through. This is the escalation, with the
evidence for each part measured rather than asserted.

## 1. Blocker 1 — the merge guard refuses `ui/src/app-v2/AppV2.tsx`

Attempting the one card I *could* legitimately support (the sanitisation dispatch, whose
verdict exists — see §2), the tool replied:

```
request_acceptance rejected: merging "packet/mira/knowledge-panel" into
"track/infra-foundation" would DELETE 1 file(s) tracked on
"track/infra-foundation": ui/src/app-v2/AppV2.tsx — the delivery must not remove
merged work
```

I reproduced it independently rather than trusting it, since the tool could be wrong:

```
$ git merge-tree --write-tree track/infra-foundation packet/mira/knowledge-panel
  -> clean preview tree (no conflicts)
$ git diff --name-status track/infra-foundation packet/mira/knowledge-panel | grep ^D
  D  ui/src/app-v2/AppV2.tsx
```

Exactly one file, and it is the one I deleted deliberately on `disp-DFeH6Ut_aNfzIbuuR21V5`
item 3b, whose packet instruction was to delete it and whose delivery note records why.
The guard is doing its job — it is a containment defence against a merge destroying
already-merged work, and it is correct to refuse rather than to reason about intent. What
it cannot do is tell a deliberate deletion of dead code from an accidental one, because
it looks only at paths.

**That the deletion is safe, measured three ways** (I checked before escalating, because
"the code is dead" is exactly the kind of claim that must not be asserted on looks):

1. No importer, any spelling, anywhere in the tree:
   `grep -rn "AppV2" ui/src src-tauri/src` returns only two **comments** that discuss its
   absence, no import.
2. No route/mount reference in the Rust entry point: `grep -rn "AppV2" src-tauri/src` is
   empty.
3. The same grep **on track itself** returns nothing outside its own file — so it was
   already unreachable before I deleted it, not made unreachable by the deletion.

And the tree is strictly healthy without it: `tsc --noEmit` clean, `vite build` green,
`vitest` 239/239 including the tests that cover the components it used to import.

So this is a **policy question for the operator, not a defect to route around**: either
the deletion is accepted and the branch merges, or it is not and the file stays. Both are
defensible; picking one silently from a seat shell is not.

## 2. Blocker 2 — four of the five dispatches have no verdict to cite

The packet asks for cards covering six dispatches. Guard 2 on `request_acceptance`
requires a `flux_verdict_ref` that exists, is a `ReviewVerdictIssued`, says **pass**, and
**names that dispatch** — through `payload.dispatch_id` or an `event_object_refs` row with
`ref_type='dispatch'`. That rule exists because of the incident this seat already lived
through: three pending cards once cited a **fail** verdict about a different dispatch,
because the check used to be existence-only. I am not going to be the person who tests
that guard a second time.

Measured across `ReviewVerdictIssued` for the five dispatches:

| dispatch | work | independent verdict |
|---|---|---|
| `disp-FzldOwJuxCfxc2TGq9dMw` | document sanitisation | **yes** — `ev-ucN0gvx2vIB_MmDMiZEgz`, `pass`, names this dispatch |
| `disp-ZqvAuQ1GANGLx8Y8FLz2T` | sandbox identity strip | **none** |
| `disp-DFeH6Ut_aNfzIbuuR21V5` | restart restore + event filter + legacy fakes | **none** |
| `disp-RWEf5ATPvICLkGtF__fEY` | V5 honesty fixes | **none** |
| `disp-bBdpBqTT9g5-kKU28xvIt` | the branch sync itself | **none** |

I read the one passing verdict's payload rather than trusting its id: it names
`disp-FzldOwJuxCfxc2TGq9dMw` in `payload.dispatch_id`, verdict `pass`. That one is real.
The other four dispatches have several events each, but all are `ArtifactCreated`
delivery announcements of my own — my own announcements are not independent verification,
and citing one as a verdict would be exactly the fabrication this project has been
burned by.

**So even if blocker 1 were resolved, four of the six cards could not be opened.** The
packet's premise ("six dispatches ready, only the card is missing") is wrong in one
specific way: the verdict step has not happened for four of them.

## 3. What I did not do

- Did not open a card with a fabricated, borrowed, or self-authored verdict reference.
- Did not edit code to make the branch non-deleting by restoring a file I deliberately
  removed, which would have undone documented work to satisfy a guard.
- Did not weaken, bypass, or try a different base/branch to get past the containment
  check.
- Did not merge.

## 4. What clears each blocker

**Blocker 1** (needs the operator, one decision): accept that deleting
`ui/src/app-v2/AppV2.tsx` is the intended outcome of the dead-code dispatch and allow this
branch to merge with that deletion — or decree the file stays, in which case I restore it
and the guard is satisfied. Nothing in between can be decided from this seat. Note the
consequence for the sandbox: while the branch cannot merge, the sandbox build keeps
showing the pre-filtering branch, which is the very confusion that produced the sync
dispatch.

**Blocker 2** (needs Flux, four dispatches): a `ReviewVerdictIssued` for each of the four
dispatches above, each naming its own dispatch. They are independent questions, not one
batch — the sanitisation verdict was issued on a different day from the identity-strip and
three-in-one work, and lumping them would produce one verdict claiming to cover four
things it did not look at.

## 5. What is ready, unchanged

Branch state is exactly as the packet found it and every gate is green, so the moment the
two blockers clear, opening the cards is mechanical: behind track **0**,
`boundMirrorNames` **2** hits, `tsc` clean, `vitest` **239/239** (19 files, including the
5 call-site tests that Flux asked for), `cargo` **283/283**, clippy clean, fmt clean,
`vite build` green. Working tree clean apart from the two pre-existing untracked scratch
files.

## 6. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
