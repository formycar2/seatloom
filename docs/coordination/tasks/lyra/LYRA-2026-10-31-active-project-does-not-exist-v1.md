# Product finding: the operator's active project does not exist, and nothing in the product notices — so every surface reads empty

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-active-project-does-not-exist-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, product-finding, blocking, state, ui |

## The finding

**His active project is `project_delivery`. That project does not exist.** I read it from his live persisted
store and then checked the `projects` table:

```
activeProjectId = project_delivery
```

```
projects:
  seatloom                         SeatLoom
  ...
  project_delivery/QL              project_delivery/QL
  project_delivery/WX              project_delivery/WX
  project_delivery/networkOrchestrator  networkOrchestrator   (2026-10-26)
  project_delivery/stepaicloud     stepaicloud              (2026-10-27)
```

**There is no row whose id is `project_delivery`.** There are four rows that *start* with it, which is almost
certainly how the id arose — a directory-style prefix that was once a project and is now a namespace.

And what that project contains, per table:

| table | rows under `project_delivery` |
|---|---|
| `workitems` | **0** |
| `handoffs` | **0** |
| `documents` | **0** |
| `canonical_events` | **0** |

**Every project-scoped read returns nothing.** So the operator opens the app, and the Work Items tab is
empty, the inbox is empty, the artifacts are empty, the timeline is empty — and the empty state he reads on
the inbox is *"status synced / all tasks processed"*, which is a **claim
about his work being finished** when the truth is **the project he is looking at does not exist.**

## Why the empty state makes it worse, not better

I checked whether the empty state is honest. It is not, and the reason is precise:

`hydrateProjectData` builds `inboxItems` **fresh** from real data — blocked workitems, in-review workitems,
handoffs — and assigns it, replacing the seed fixtures. So an empty list genuinely means "nothing to show."
**The code is correct.** The bug is that the product never asks *whether the project the empty list came
from exists.*

**So the operator's read is: "my work is all done." The actual state is: "I am looking at a project that
is not in the database."** Those are opposite conclusions, and the second is the one that matters — the work
is not done, it is in `seatloom`, which has 7 workitems, 3 handoffs and all 3 of his pending acceptance
cards.

## The compounding, and it is the whole picture now

This is the **fourth** independent reason he cannot act, and it sits underneath the other three:

| # | Cause | Effect |
|---|---|---|
| 1 | `supervisor.open = false` | the panel listing his cards is closed |
| 2 | panel scopes to `activeProjectId` | it would show "nothing to accept" — because there is genuinely nothing under this project |
| 3 | installed app predates the fix by 43h | the card's own steps would fail |
| 4 | **`activeProjectId` names a project that does not exist** | **every surface is empty, and the empty states assert his work is finished** |

**Cause 4 explains cause 2's empty list, and it explains something I have been reporting as a separate
problem.** I wrote last round that the panel shows him nothing because his active project has zero
acceptances. That is true and it is not the real defect: **the real defect is that the active project should
not be active.**

## Why nobody catches it

The store has the discipline elsewhere — I found comments about not publishing stale rows, about a failed
read not marking a different project failed, about rendering an empty state rather than stale rows. **What
is missing is the one check that would catch this: after `listProjects`, if the persisted `activeProjectId`
is not in the returned list, it must be dropped** — and the product must say so, rather than silently
rendering an empty project that claims to be caught up.

## The ask

**Validate `activeProjectId` against the fetched project list, and when it does not resolve, fall back to a
project that does** — the natural choice being `seatloom`, the one holding all the real work and all three
pending cards. **And surface the correction**: *"the project you were last in (`project_delivery`) no longer
exists; switched to SeatLoom."* A silent fallback would fix the data and hide the event, which is the same
failure one level down.

This is Mira's surface, so this is an ask and not an edit.

## What I did not do

I did not edit `useDataStore.ts` or `App.tsx`. **I did not write to his persisted localStorage** — changing
his active project from a seat would be editing his session state to make my measurement look better, which
is the same overreach as flipping `supervisor.open`. I did not grant, cancel, or re-open any card.

## A note on how I found it

This came from the empty-state audit, not from the acceptance-card chase. **I had been asking "why has he not
clicked the cards" for several rounds and producing causes; the cause was one level below all of them** — he
is not looking at a project that has cards in it, and nothing told him.
