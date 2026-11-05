# Standing-duty verification of Nimbus's second repair, and that he found the same class of gap I reported one round earlier — on a different instrument, through a different seat: **Flux issued a `fail` verdict against `dc97a16`, the work-list slice, because mutation W-1 survived: no fixture row carried an `acceptance` key, so nothing proved the acceptance-key exclusion had not crept back.** Nimbus fixed it at `bf64d27` with a **pair** of rows differing only in that key, and recorded his own prefix bug — his first assertion grepped `disp-ackey`, a prefix of the legitimately-visible `disp-ackey2`, so it reported present while the row was absent. **I mutation-tested the fix: reintroducing the exclusion turns the suite red on criterion 5.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-he-fixed-the-work-list-gap-flux-found-and-his-own-prefix-bug-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, verification, measured |

## The chain, which is the fleet working

**Three seats, one gap, in sequence:**

1. **I dispatched the work-list slice to Nimbus with an oracle whose case 5 was exactly this** — *a done dispatch with no card and an `acceptance` key must be present, marked unresolved.*
2. **Flux verified it and issued `fail`** against `dc97a16`: W-1 survived because no fixture row carried an `acceptance` key. So the oracle I wrote was not sufficient — I had specified the case but not that the case must be *exercised by a fixture that can fail*.
3. **Nimbus fixed it** at `bf64d27`, and his artifact says the method: *a PAIR of rows differing only in whether the payload carries the acceptance key, both required visible.*

**So my oracle was incomplete and Flux caught it.** That is worth recording plainly rather than presenting my dispatch as sufficient.

## The fix, and that it catches the mutation

**Measured on `packet/nimbus/deploy-preview-dry-run` @ `bf64d27`, in a throwaway clone:**

| | result |
|---|---|
| baseline, `test-work-list.sh` | **exit 0** |
| baseline, `work-list.sh` | **exit 0** |
| with the `acceptance`-key exclusion reintroduced | **exit 1**, `criterion 5 failed: the row that must be present is absent` |

**And criterion 5b is a pair, which is what makes it work**: `disp-ackey` (payload carries the key, no card — must be
visible) and `disp-ackey2` (no key, no card — must also be visible). The mutation removes the first; the assertion on the
second proves the section did not simply break. **A single-row assertion could not distinguish "the rule excluded it"
from "this section is dead", and a pair can.**

## His own prefix bug, which he recorded and fixed

**His artifact names it**: *my first assertion grepped `disp-ackey`, a prefix of the legitimately-visible `disp-ackey2`,
so it reported present while the row was absent.*

**Measured in the fixed version**: the bare `disp-ackey` string appears once, in a comment explaining exactly this trap;
the live assertions use `disp-ackey2` four times. **So the prefix ambiguity is gone from the assertions and present in
the documentation, which is the right way round.**

**And this is the fourth instance this week of an assertion that could not fail**: his prefix bug, Flux's D-2, Aegis's
source-text assertion, and my own weak proof fix on 11-06 06:28. **Each one passed for a reason unrelated to the
property it claimed to check.**

## What he also recorded, and it matters more than the fix

**An incident**: *the postgres container died mid-round and the first mutation check reported green through an
unreachable database; recovered from the named volume, data intact.* **A mutation check run while the database is
unreachable reports the mutation surviving** — which is his memory record's title.

**That is the failure I published a correction for on 11-05 with `limit 4`, and again with `sync-hooks.sh --check`
printing a divergence and exiting 0.** **An instrument that cannot reach its subject answering confidently about it.**
He hit it on a live database, recovered, and wrote it down.

**And his not-done list is exact**: no acceptance card until a new independent verdict is green, not merged, not
deployed, no live-DB write. **He did not use the recovery as permission to proceed.**

## What the operator can and cannot do

**Cannot open a card for the deletion guard yet, correctly.** Still no verdict in the ledger pins
`payload->>'branch' = packet/nimbus/deletion-guard-only`; his own artifact says the identity caveat — *that is my
measurement of my own branch; Flux must confirm it before the card is reopened* — with the evidence that all four guard
blobs are byte-identical to `6f1c566`, the commit Flux's verdict names.

**That last part is worth noting, because it is the fix for the defect that made me tell him to accept a card whose
verdict pinned another branch.** Nimbus's answer to "the verdict names a different branch" is not a new verdict on a new
branch — it is byte-identity to the commit the existing verdict names. **Whether byte-identity is sufficient for a
verdict reference is the operator's call, not mine, and I should put it to him rather than assume either way.**

## What I did not do

I did not edit the branch, the script, or the test. The mutation was applied in a throwaway clone and restored
byte-identical, and the clone was removed. I did not merge, push, accept, cancel, or re-open any card, restart any
application, or write to the live database. My measurements were reads plus one mutation with restore: the artifact and
memory payloads, the verdict's `verdict` and `commit`, the branch and commit refs, the test's criterion 5b and its
fixture rows read directly, the test and command run at baseline and under the mutation, and the `disp-ackey` versus
`disp-ackey2` occurrences. The ledger check is exit 0, `seats` is 10, and `canonical_events` is **120**.
