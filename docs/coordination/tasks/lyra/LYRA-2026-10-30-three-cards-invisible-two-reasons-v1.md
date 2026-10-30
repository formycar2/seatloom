# Product finding: the operator's three pending acceptance cards are invisible to him, for two independent reasons — and neither is a bug anyone would file

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-three-cards-invisible-two-reasons-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, product-finding, blocking, acceptance, ui |

## The question I asked

He has three pending acceptance cards, all verified clickable, and has not clicked in about five hours.
Earlier rounds I concluded "that is his to do." **This round I asked the product-audit question instead: what
does he actually see when he opens the app?** The answer is: **nothing about those cards, for two
independent reasons, and either one alone would hide them.**

## Reason 1 — the panel that shows them is closed, and defaults closed

Pending acceptance cards are rendered by `ui/src/app-v2/panel/SupervisorIntent.tsx`:

```ts
pendingAcceptances: acceptances.filter(
  (a) => a.status === 'awaiting_acceptance' || a.status === 'pending',
),
```

That component is rendered inside `SupervisorPanel`, and the panel's open state is
(`ui/src/App.tsx:155-157`):

```ts
const [showSupervisorIM, setShowSupervisorIM] = useState(
  () => localStorage.getItem('seatloom.supervisor.open') === 'true',
);
```

**Default closed, restored from localStorage.** And his live persisted store, read from the app's own
WebKit localstorage:

```
seatloom.supervisor.open = false
```

So the only surface in the product that lists pending acceptance cards is **closed in his session right
now.** I grepped for a pending-card count or badge anywhere outside that panel — `NavRail`, `layouts/`,
`components/` — **zero matches.** There is no badge on the landing tab, no count in the nav, nothing.

## Reason 2 — even if he opened it, it would show him nothing, because of the active project

The panel scopes its reads to the active project (`SupervisorIntent.tsx:176`):

```ts
api.listAcceptances(activeProjectId),
```

His persisted active project, read from the same store:

```
activeTab       = dispatches
activeProjectId = project_delivery
```

And the ownership of every acceptance in the database:

| project | statuses | count |
|---|---|---|
| `seatloom` | accepted 68, cancelled 9, **pending 3** | 80 |
| `project_delivery` | — | **0** |

**So `api.listAcceptances('project_delivery')` returns an empty list.** All three of his pending cards
belong to `seatloom`, and his active project is a different one that has never had an acceptance. **Opening
the panel would show him an empty "no pending cards" state — which is worse than not showing him, because
it actively tells him there is nothing to accept.**

## Why this is the most expensive finding of the week

Everything else I reported was a record that was wrong or a badge that lied. **This is the operator's only
job in the product being structurally unreachable**, and the failure is silent in both directions:

- He cannot see that 3 cards are waiting.
- The panel, if opened, tells him none are.

**And it explains a thing I have been mis-reading for days.** I have repeatedly measured "he granted two
cards at 08:41 and has not clicked since" and treated it as his pace. **It is not his pace. It is that the
cards he granted were visible at that moment and the ones since have not been** — the active project
changed, or the panel closed, or both, and nothing in the product told him anything was waiting.

The work is done. The verdicts exist. The cards are clickable — I verified `check-cards-clickable.sh` says
`OK: all 3 pending cards are clickable`. **And he has no way to know they exist.**

## What I am not claiming

I am **not** claiming he is ignoring the work. He has been active on Aegis's heartbeat all evening. **I am
claiming the product does not surface the one action only he can take**, which is a different and much more
serious statement.

I am also **not** claiming this is anyone's bug in the "someone wrote it wrong" sense. Both halves are
locally reasonable: a panel that defaults closed is a normal preference, and scoping reads to the active
project is correct everywhere else in the app. **The defect is that no surface anywhere tells him the
count, so the two reasonable decisions compose into invisibility.**

## The ask

**A pending-acceptance count that is visible regardless of panel state and regardless of active project.**
The natural place is the nav, next to the existing `inbox` badge (`NavRail.tsx:72` already renders
`badge: inboxCount > 0 ? inboxCount : undefined`), so the pattern exists and needs a sibling.

**And the panel must not scope acceptances to the active project**, or must say explicitly that it is
showing `project_delivery`'s zero while three wait under `seatloom`. An empty state that reads as "nothing
to accept" when three things are waiting is the specific thing to kill.

This is Mira's surface, so this is an ask and not an edit.

## What I did not do

I did not edit `App.tsx`, `SupervisorIntent.tsx`, or `NavRail.tsx`. I did not change his persisted
localStorage — his session state is his, and flipping `supervisor.open` from a seat would be the same
overreach as editing his code. I did not grant, cancel, or re-open any card. I did not dispatch anyone: this
is a finding with a measured cause, and the fix is a badge plus a scoping correction.
