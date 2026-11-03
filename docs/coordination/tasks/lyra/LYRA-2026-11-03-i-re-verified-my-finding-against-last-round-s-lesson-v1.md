# Standing-duty verification: I re-verified my published finding against the lesson from last round's near-miss — there is no reconciling layer, the two surfaces are co-visible in the same shell, and the finding survives. The near-miss changed what I checked, not what I found.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-i-re-verified-my-finding-against-last-round-s-lesson-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, verification, confirmed |

## Why this round

Last round I caught myself one measurement from publishing a false defect, and the lesson I wrote was
specific: **I went looking for the defect at the point where two representations differ, which is exactly
where a correct system is supposed to differ.** A vocabulary of nine and a filter of three looked like a
discrepancy; a map that converts one to the other made it harmless.

**A lesson that only applies to the finding I retracted is not a lesson.** So this round I turned it on the
finding I *published* — the inbox badge and the supervisor panel counting different sets under one label.
**The question the lesson asks is: is there something between the two layers that reconciles them?**

## Measured, and there is not

**Layer 1 — the badge.** `NavRail.tsx:66` reads `currentData?.inboxItems.length`, where `currentData` is the
store's per-project data for `activeProjectId`. `inboxItems` is derived in `useDataStore.ts:1956-2010` from
workitems and handoffs.

**Layer 2 — the panel.** `SupervisorIntent.tsx:105,160` reads only `activeProjectId` from the store. It does
not read `inboxItems`. It fetches its own data — `api.listAcceptances`, `api.listDispatches`,
`api.listHandoffs`, `api.listEvents` — and holds the result in local state (`setDelta`, line 154).

**So the two surfaces do not share a source, a read model, or a store field.** There is no map, no
normalizer, no shared selector between them. **The reconciliation layer that made my retracted finding
harmless does not exist here.**

## And the two surfaces are co-visible, which is what makes it matter

I checked reachability rather than assuming it, because a divergence between two screens the operator never
sees together is a much smaller problem.

- `NavRail` is rendered by `layouts/AppShell.tsx:2,52`, inside `<AppShell>` (`App.tsx:810-846`).
- `SupervisorPanel` is rendered at `App.tsx:875`, **inside the same `AppShell`**, gated on
  `showSupervisorIM` — a floating panel opened by ⌘K.

**So the badge on the inbox tab and the panel are in the same shell at the same time.** The operator can see
a badge reading 0 on the inbox tab while a panel reading 1 pending acceptance floats beside it. **That is
not a hypothetical divergence between two routes; it is two numbers about "what needs my attention" on one
screen.**

**And the badge's own label is the Chinese for "inbox"** (NavRail.tsx:72) - the same word the panel uses
for its pending-acceptance section (SupervisorIntent.tsx:257). **The label collision is not my
inference; both surfaces call themselves the inbox.**

## What the near-miss changed

**It changed what I checked, not what I found.**

- Before last round I would have published the two-sets finding and moved on.
- This round I asked the question the near-miss taught me — *is there a reconciling layer?* — and measured
  the answer: **no shared source, no shared field, no map, two independent fetch paths.**

**The finding survives because the answer is "no," and I know that because I looked.** That is the difference
between a finding that was checked and a finding that was assumed, and the near-miss is what forced the
check.

**The general form, which is what I will carry:** when two layers disagree, the defect is not the
disagreement — it is the *absence of a reconciling mechanism*. **If a reconciler exists, the disagreement is
representation. If none exists, the disagreement is the defect.** Last round I found a reconciler and was
wrong to call it a defect. This round I looked for one, did not find it, and the finding stands.

## What I did not do

I did not edit any view, store, or the contract. I did not dispatch. My measurements were the store-field
reads in both components, the panel's own fetch calls and local state, the mount points of both surfaces
(`AppShell.tsx`, `App.tsx:810-846`, `App.tsx:875`), the ⌘K gating condition, and the badge's label string.
`seats` is 10, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
