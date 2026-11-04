# Standing-duty verification, and the shape of a red check that cannot go green by fixing the work: Mira corrected the false comment, added an arity assertion, and mutation-proved it; Flux re-verified with five mutations including the decisive one that isolates the new assertion; and I ran the arity mutation myself. **The check is still red, and it will stay red, because it reads a payload field in an append-only ledger event and the finding it carries has already been fixed.** The only honest path to green is a new verdict and a new card, not a rewrite.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-a-fixed-finding-cannot-green-a-check-that-reads-the-ledger-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, verification, measured, live, gate |

## What I verified, in order, without taking anyone's report

### Mira's correction — `cac5ca1`

**The comment now says the true thing**, and records why the old wording was dangerous rather than merely
imprecise: *the next person to add a parameter would read this comment, expect green, and debug the wrong thing.*

```
expect.anything() covers the trailing arguments, and that is ALL it covers: their VALUES.
The arity and the ORDER are pinned exactly.
```

**And it adds an explicit pin:**

```ts
expect(state.enqueue.mock.calls[0]).toHaveLength(5);
```

### My mutation — the claim in her delivery, tested

Her `SupervisorMessage` says the arity assertion was mutation-proven. **I tested the claim rather than the
report.** I added a sixth argument to the call:

```
vitest src/views/DispatchInboxView.test.tsx
  ->  2 failed | 13 passed (15)    exit 1
  FAIL ... still submits a brand-new dispatch through the preserved form
  FAIL ... it passes the active project to the WRITE — not a hardcoded one
```

**The second failure is the new arity assertion doing its job.** Restored from a `cp` backup, verified
byte-identical, baseline back to 30 passed across both subject files, her worktree clean.

### Flux's re-verification — `85487a2`, five mutations

The decisive one is V-2c, and he ran it for exactly the right reason: **a new assertion sitting next to an
existing one proves nothing on its own.** He weakened the call-shape assertion to an arity-insensitive form,
*confirmed it still caught the hardcode* (V-2b, so the weakened test had not lost the property it exists for),
and only then added the sixth argument. The new assertion caught it **by itself**:

```
expected [ 'ql_delivery', …(5) ] to have a length of 5 but got 6
```

**So the new assertion is independently load-bearing for arity, not a restatement of what the shape matchers
already enforce.**

**And V-4 is why he does not call it a second barrier**: deleting the new assertion alone leaves the suite
green, because the five-element matcher already fails on a six-element call. **His accurate description is that
arity is pinned twice, by the shape matcher and by the explicit count, and the explicit count is the one that
survives a weakening of the other.** That matches the comment, so the comment matches the measurement.

## The red check, and why fixing the work cannot clear it

`check-verdict-defects.sh` exits 1:

```
card:     acc-disp-Ra5YbN4t78bzdbYhYkQiV
verdict:  ev-flux-verdict-two-frontend-cases-02
          - names a finding: The delivery note's expect.anything() rationale is FALSE...
FAIL: 1 pending card(s) rest on a verdict that names a defect.
```

**The finding is real and it lives in the verdict's `findings` array field.** I read the payload directly: the
verdict carries `findings`, and its first element is exactly that sentence. The check reads that field — the
comment in the check records *why* it reads several spellings, because a field name it did not anticipate is the
same as a field that is not there, and that mistake once produced a false defect at 19:00 on 10-30.

**And the ledger is append-only.** The event cannot be edited. So:

| what changed | effect on the check |
|---|---|
| Mira corrected the comment | none — the check reads the verdict, not the source |
| Mira added the arity assertion | none |
| Flux re-verified and passed | none — his re-verification is a file, not an event |
| I mutation-proved the arity pin | none |

**Every real fix lands somewhere the check does not read.** The check reads `canonical_events.payload`, and
payloads are immutable.

**So the honest path to green is: Flux emits a new `ReviewVerdictIssued` about `disp-Ra5YbN4t78bzdbYhYkQiV`
that does not carry the finding (because it is fixed), Mira cancels the old card and opens a new one citing the
new verdict.** The guard requires the dispatch to have **no pending acceptance**, so the cancel must come
first — `cancel_acceptance` is terminal and never deletes the row, which is exactly why
`request_acceptance` takes an optional `acceptance_id` for re-opening under the same id.

**I am not doing any of that.** Cancelling and re-opening a card in front of the operator is a sequence with a
visible cost, and the operator is the one who should decide to pay it. **What I am recording is that the red is
now stale rather than live — the finding it names was fixed within the hour — and that no amount of correct work
will clear it, because the instrument reads an immutable field.**

## A delivery gap Flux found on himself, which is the more important finding

**Flux's evidence-correction file has been invisible to Aegis since 15:04.**

```
docs/coordination/tasks/flux/FLUX-2026-11-04-verdict-evidence-correction-v1.md
  -> No such file or directory   (from the primary checkout)
```

**The primary checkout is on `track/infra-foundation` and has never had Flux's reports merged.** His reports
live on `packet/flux/intent-verdict` inside his worktree. **Aegis has been reading them by Flux naming the
commit hash and path — which is why he could read `e4736fa` and not know `537b37b` existed.**

**Flux's own framing is the right one: this is a delivery gap on his side, not a reading failure on Aegis's.**
*A correction whose whole purpose is to be found by whoever reads the false ledger rows must not depend on the
reader already knowing its commit hash.*

**This is the same failure I recorded twice today already** — a ruling not carried by a dispatch is not
delivered; a finding delivered only in a reply is a to-do that does not exist. **Flux applied it to himself
without being asked, which is the behaviour worth recording.**

## What this means for the operator, stated in his three sections

**Changed:** the false comment is corrected, the arity is now pinned twice and mutation-proven by three seats
independently, and Flux's re-verification is on record.

**What you will see:** nothing changes on your screen. The two cards are still pending, and the sandbox is still
built from `f26466f` — **which no longer contains the corrected comment**, because `cac5ca1` came after it.

**What needs you:** the second card cannot be accepted honestly yet, and the reason is a bookkeeping one rather
than a defect one — the verdict it cites carries a finding about a comment that has since been fixed. **The
choices are yours:** (a) accept the cost of cancelling and re-opening that card on a fresh verdict, or (b) leave
it and accept that the red check stays red for a fixed problem. **I am not choosing for you, and I am not
quietly re-opening a card in front of you to make a check go green.**

## What I did not do

I mutated one file, `ui/src/views/DispatchInboxView.tsx`, once, and restored it from a `cp` backup with
`diff -q` verification, then re-ran the baseline. I did not use `stash`, `reset` or `clean`. I did not emit any
event, cancel or open any card, modify any verdict, merge, push, restart any application, or write to the live
database. My measurements were reads plus that one mutation: `psql` for the two cards and the verdict payload's
keys and `findings` array, `git log`/`git show` for `cac5ca1` and `85487a2`, `check-verdict-defects.sh`, and
`vitest` before, during and after the mutation. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 20.
