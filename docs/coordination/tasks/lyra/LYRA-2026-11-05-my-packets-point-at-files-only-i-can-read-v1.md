# Standing-duty finding that every packet I have ever dispatched points at a file that is not on `track`, and that the work got done anyway — which is the mechanism working, not the packets working: **223 of the 258 documents in `docs/coordination/tasks/lyra/` on my branch are not on `track/infra-foundation`, and all three packets I dispatched to Aegis this week are among them.** He completed two of those dispatches without ever citing a packet, because the dispatch's `note` carried the content. **So the packet field is a pointer to a file only I can read, and the note is what actually delivers.** That is a finding about my own practice, and it means every dispatch I have sent this week would have worked identically with the packet field removed.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-my-packets-point-at-files-only-i-can-read-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, dispatch-channel |

## What I measured, and it is worse than a one-off

**Every packet path I have ever put in a dispatch to Aegis:**

```
disp-ZZEb6fpwXPR0zqMQjiHwV  ->  LYRA-2026-11-05-the-push-channel-covers-five-things-and-not-this-v1.md
disp-bvCX0jjTVC6rEMD251ODj  ->  LYRA-2026-11-05-the-lifecycle-has-no-state-for-finished-and-unsaid-v1.md
disp-aNMhDN7ICyR_2_qPf50iL  ->  LYRA-2026-11-05-the-operator-resolved-my-ruling-and-one-piece-is-outstanding-v1.md
```

**None of the three is on `track/infra-foundation`.** Measured with `git cat-file -e HEAD:<path>`.

**And it is not three files, it is the practice.** On my branch `packet/lyra/milestone-summary`, `docs/coordination/tasks/lyra/`
holds 258 documents. **223 of them are not on `track`.** So a packet path is a pointer into a tree the recipient's
seat does not have.

**The scale of it, measured rather than estimated:**

| | count |
|---|---|
| documents in `docs/coordination/tasks/lyra/` on my branch | 258 |
| of those, not on `track/infra-foundation` | **223** |
| packets I dispatched this week | 3 |
| packets whose path resolves on the recipient's tree | **0** |

## Why the work got done anyway, and that is the part worth naming

**Aegis completed two of these dispatches, and neither delivery note cites a packet** — measured, `grep -c` for
`LYRA-2026-11-05` in both notes returns **0**. He did not read them. **He could not.**

**What he did read is the dispatch's `note`**, which carries the measured facts, the requirements, and the
acceptance step inline. So the delivery mechanism is the note, and the packet is decoration that points at a file
only I can open.

**This is not a complaint about Aegis — he did the right thing with what reached him.** It is a finding about my own
practice: **I have been writing a pointer to an unreachable file and calling it a delivery, and the only reason the
work happened is that I also wrote the content into the note.**

## What this means for the outstanding dispatch

**`disp-aNMhDN7ICyR_2_qPf50iL` is still `ready` and its packet is unreachable.** But its note carries the ruling's
terms, the three replacement criteria, and the mutation discipline — so it will work when he takes it, the same way
the previous two did.

**What it does not carry is the second obligation I identified last round**: that the deletion check must be real at
click time, not present on a branch. I wrote that in my document and said I would fold it into the dispatch, and
**I cannot — the dispatch tools have no update operation.** `enqueue_dispatch`, `complete_dispatch`, `cancel_dispatch`,
`get_dispatch`, `get_next_dispatch` are all there is; nothing modifies a payload.

**So the honest options are two, and I am recording which I take:**

1. **Cancel and re-issue with both obligations in the note.** Costs the ruling's terms being restated, and the
   cancelled dispatch stays in the ledger.
2. **Leave it and dispatch the second obligation separately.** Two dispatches where one would do.

**I take the second.** The ruling dispatch is already correct on its own terms and I do not want to cancel work that
is accurately specified; the deletion-guard-at-click-time obligation is separable and Aegis already measured it
himself this morning, so he has the evidence in front of him. **A cancellation here would be tidiness, not accuracy.**

## What I did not do

I did not edit any script or product code, cancel or re-issue the dispatch, merge, push, accept, cancel, or re-open
any card, restart any application, or write to the live database. My measurements were reads: `git cat-file -e
HEAD:<path>` for each of the three packets, the recursive `ls-tree` on my branch with a per-file `cat-file -e` against
`track`, the dispatch rows' `payload->>'packet'`, `grep -c` for packet citations in his two delivery notes, and a
`searchTools` sweep confirming no dispatch-update tool exists. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 35.
