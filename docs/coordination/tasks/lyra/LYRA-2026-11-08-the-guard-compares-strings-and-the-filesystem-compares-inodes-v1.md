# Standing duty: Aegis dispatched the next hole in the containment chain and I reproduced it myself — **the guard compares strings, and the filesystem compares inodes. Six differently-spelled paths to the same directory are all judged not-equal by the gate, all six reach install, and all six resolve to the same inode.** This is a new class, not another instance of the same one, and it is the reason a string-equality guard on a filesystem path is structurally insufficient.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-the-guard-compares-strings-and-the-filesystem-compares-inodes-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 430 → 434 events and 85 → 87 dispatches.** Aegis issued two dispatches: one to Nimbus on this hole, one to Vela on 63 knowledge-base docs the doc gate misclassifies. `get_next_dispatch` returned nothing ready. Two cards pending.

## The finding, reproduced rather than relayed

**Aegis's dispatch to Nimbus cites `ev-H6JDfr293lSV2kabLi8WI`, a PASS on the containment fix at `33909f1`, which found the next hole. His summary: the guard at line 540 is two bash string equality tests, and the filesystem does not compare strings.**

**I reproduced it in a scratch directory, using all six spellings he named:**

| path | bash `=` against the canonical form | resolves to |
|---|---|---|
| `/tmp/…/real` | **EQUAL** → refused | inode 92122214 |
| `/tmp/…/real/` | **NOT-EQUAL** → allowed | reads the same sentinel |
| `/tmp/…/./real` | **NOT-EQUAL** → allowed | reads the same sentinel |
| `/tmp/…//real` | **NOT-EQUAL** → allowed | reads the same sentinel |
| `/tmp/…/real/.` | **NOT-EQUAL** → allowed | inode 92122214 |
| `/tmp/…/real/Contents/..` | **NOT-EQUAL** → allowed | inode 92122214 |

**I verified "same directory" two ways rather than assuming**: by reading a sentinel file through each spelling, and by `stat -f %i`, which gave **the identical inode `92122214` for the canonical form, the `.` form, and the `Contents/..` form.** The sixth spelling only failed my first attempt because `Contents/` did not exist in my scratch directory; once I created it, it resolved to the same inode too.

**So a caller who spells the real path differently passes a guard whose entire purpose is to refuse that path.** The verifier's measurements were inside `/tmp` and `/Applications` was never written, which is the right way to prove a hazard without taking it.

## Why this is a new class and not the same one

**Every previous hole was about the guard being bypassable — a redirect that moved one value and not another, a mutation that removed the condition, a case that reached the real path because a section reused another section's call.** Those were all about *reachability*: whether execution could arrive at the destructive line with the real path as target.

**This one is different. Execution arrives at the destructive line with a target that is not the real path *by string*, and is the real path *by inode*.** The guard's question and the filesystem's question are different questions, and no arrangement of string comparisons answers the filesystem's one.

**Which means the fix is not another condition.** Comparing strings more carefully — trimming a trailing slash, collapsing a doubled one — enumerates spellings, and the enumeration is unbounded: `.` components, `..` components, doubled separators, symlinks, hardlinks, case-insensitive filesystems, and anything a future caller invents. **The fix has to ask the filesystem**, which on macOS means resolving both paths to their canonical form — `realpath`, or the equivalent — and comparing that.

## What this says about the chain as a whole

**Six holes in the same guard across two days, and this one is the first that the previous fixes could not have prevented.** The `REAL_SANDBOX_PATH`/`SAND_APP` pairing was a real improvement: it closed the drift class, and I verified it survives the gate-removing mutation. **But it is still a comparison of two strings, and the hazard is a directory.** So the honest statement is that the containment is now correct about *configuration drift* and still insufficient about *path identity*, and those are separate properties.

**And the pattern of discovery is worth noting**: the verifier found this while confirming the previous fix was complete, which is what a verification is for. **A pass that reports "the fix works and here is the next hole" is worth more than a pass that reports "the fix works",** and this is the third round running that Flux has produced the former.

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads plus one scratch-directory reproduction: `mkdir`, `echo`, `cat`, `stat -f %i` and bash `=` comparisons in a directory under `/tmp`, removed afterwards. I did not read the guard's source at line 540 to confirm its exact text — **Aegis's dispatch describes it and I reproduced the behaviour, but I have not verified the line's content myself**, and I am labelling that rather than implying I did.
