# Standing-duty correction to my own finding, and the measurement that makes it dispatchable: I wrote last round that a dispatch created outside `seatloom` "appears on neither side." That is wrong on one side. It is invisible on the project the operator was viewing — which is the side he acts on, and the only side that matters — but it *is* listed under `seatloom`, grouped under its raw seat id. The defect is real and the correction makes it narrower and more precise.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-correction-the-dispatch-is-not-invisible-on-both-sides-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, correction, measured, live |

## The claim I am correcting

In `LYRA-2026-11-04-new-dispatch-lands-in-a-hardcoded-project-v1.md` I wrote:

> **And then it is invisible from both sides:**
> - **On the project he was on** — the page reads `cmd_list_dispatches({ projectId: 'project_delivery/networkOrchestrator' })`. The new dispatch's `project_id` is `seatloom`, so it is not returned. The form closes, the list refreshes, and nothing is there.
> - **On `seatloom`** — the dispatch *is* returned, but its target seat has no binding under `seatloom`, so the seat filter cannot name it. `seatName` falls back to the raw seat id at `DispatchInboxView.tsx:752`.

**The second bullet is wrong as I worded it.** I wrote "so the seat filter cannot name it" and left "invisible from both sides" standing above it, which reads as: it cannot be seen under `seatloom`. **It can.**

## What is actually true, measured

`DispatchInboxView.tsx:751-753`:

```ts
const seatName = useCallback(
  (seatId: string): string => seatsDb.find((s) => s.id === seatId)?.name ?? seatId,
  [seatsDb],
);
```

**The fallback is the seat id, not a blank.** And the group header renders both the label and the id
(`DispatchInboxView.tsx:1291-1304`):

```tsx
const label = seatName(seatId);
...
<SeatAvatar label={label} id={seatId} size="md" />
<h3 ...>{label}</h3>
<span className="font-mono ...">{seatId}</span>
```

**So under `seatloom`, the dispatch is listed and grouped under `seat-networkorchestrator-001` — an ugly
identifier instead of `networkorchestrator`, but visible, and the raw id is printed right next to it.** An operator
browsing `seatloom` would find it.

**What is true, and is the whole defect:**

| side | returned by the page? | how it appears |
|---|---|---|
| the project he was viewing (`project_delivery/networkOrchestrator`) | **no** — `project_id` mismatch | nothing at all |
| `seatloom` | yes | grouped under the raw seat id, not the seat name |

**So the dispatch is not invisible. It is in the wrong project, and the operator is never told.** He clicks, the
form closes, the list he is looking at refreshes, and it is empty. Nothing in the application says "your dispatch
was filed under `seatloom`." **That is the defect, and it is the same defect with a more honest description.**

## The measured blast radius, which is what makes this dispatchable

| measurement | value |
|---|---|
| projects in live | 40 |
| projects with any seat binding | **4** |
| of those, not `seatloom` | **3** — `project_delivery/stepaicloud`, `project_delivery/QL`, `project_delivery/networkOrchestrator` |
| seats bound to each of those 3 | exactly **1** each |
| is any of those 3 seats also bound under `seatloom`? | **no** — each is bound to its own project only |
| running tmux sessions for them | `Stepaicloud-seatloom`, `QL-seatloom`, `NetworkOrchestrator-seatloom` — all three alive |

**So the affected set is exactly three projects, each with exactly one seat, and none of those three seats is
reachable from `seatloom`'s dropdown.** Every dispatch the operator creates from any of those three projects is
misfiled. `seatloom` itself — where he spends most of his time, and where the persisted landing project points —
is unaffected, which is why nobody has hit this.

**That is a narrow, precisely bounded defect, and it is now a dispatch with a measured scope rather than a
worry.**

## Why I am correcting rather than editing the first document

**The first document is already delivered and pushed to both remotes.** Rewriting it would leave a record that
never contained the error. **A correction that is a separate document keeps the error and its fix both visible,
which is the only version of the record that is worth anything.** This is the same rule I applied to my
`seatloom in sandbox: 0` measurement this morning.

## The failure, stated plainly

**I wrote a two-bullet list whose heading claimed "both sides," then wrote a second bullet that contradicted the
heading, and did not notice.** The mechanism of the error is the one I keep recording: I had a strong conclusion
(the dispatch vanishes) and I wrote supporting detail that fit it, including a clause — "the seat filter cannot
name it" — that is not the same statement as "it is not listed," and is not even true of the search path.

**The honest version is less dramatic and more useful:** the dispatch lands in the wrong project, the operator
is never told, and the affected set is three projects. **"It vanishes" would have been a better story and a worse
document.**

## What I did not do

I did not modify `src-tauri`, `ui/src`, or any script. I did not create a dispatch from the UI, did not run
`--apply`, did not merge, did not restart the operator's application, and did not write to the live database. My
measurements were all reads: `DispatchInboxView.tsx:751-753, 796, 1174, 1290-1304`, and `psql` against live for
projects, all `project_role_bindings` grouped by project, the three seats' binding sets, and `tmux list-sessions`.
The ledger check is exit 0, `seats` is 10, `canonical_events` is 4.
