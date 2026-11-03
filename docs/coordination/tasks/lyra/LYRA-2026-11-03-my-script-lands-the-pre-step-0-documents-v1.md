# Standing-duty finding: my landing script has a second defect, and it is the one that matters — it reads the LOCAL `docs-only` ref, which is one commit behind origin and carries the pre-Step-0 documents. Its hash check verifies the copy is faithful to that stale ref, so it passes while landing the version without the ledger guard.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-my-script-lands-the-pre-step-0-documents-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, self-correction, defect, stale |

## Why I kept going

Last round I fixed Step 5's seat list — a seven-versus-six set error that would have made the script die before
committing. **A script that dies is a script that cannot do damage.** So I kept auditing the steps that have
never run, and found the one that would not die.

**This one lands the wrong documents, and reports success.**

## The defect

`SRC_BRANCH="packet/lyra/standing-duty-docs-only"` — the **local** ref. Measured:

```
local  packet/lyra/standing-duty-docs-only   d1fb383
origin packet/lyra/standing-duty-docs-only   7e06738
```

**The local ref is one commit behind origin, and that commit is the one that adds Step 0.** Measured on the
same file from both refs:

```
local  d1fb383 : MIRA-HEARTBEAT.md  c3e16ee   check-ledger-exists refs: 0
origin 7e06738 : MIRA-HEARTBEAT.md  f97d2f2   check-ledger-exists refs: 1
```

**So `git show "$SRC_BRANCH:$f"` takes the version that does not contain the ledger-exists guard.** The five
seats would get a standing-duty document that does not start by checking whether the ledger still exists -
which is the check that would have caught my own destruction of the database.**

## Why the script's own hash check does not catch it

Step 2 does verify, and it looks thorough:

```bash
git show "$SRC_BRANCH:$f" > "$f" || die "failed to extract $f"
WANT="$(git rev-parse "$SRC_BRANCH:$f")"
GOT="$(git hash-object "$f")"
[ "$WANT" = "$GOT" ] || die "hash mismatch for $f: wanted $WANT, got $GOT"
```

**It proves the bytes on disk are the bytes in the ref.** That is a real check and it would catch a truncated
`git show` or a partial write. **What it cannot catch is that the ref itself is the wrong one** — `WANT` and
`GOT` are both computed from `$SRC_BRANCH`, so they agree by construction. **A check that compares a copy
against its source proves nothing about whether the source is the right source.**

**That is the guard-reading-a-value-its-own-operation-produced family**, and it is the reason this survived my
own verification on 10-31: I hash-checked the copy against the ref, the ref was correct then, and I never asked
whether the ref was current.

## What I found on 11-02 and did not finish

**I documented this exact drift on 11-02** — *"my local copy of that branch is one commit behind the version
that actually carries Step 0"* — and recorded the honest ask as *"rebase `docs-only` onto current track from
origin's `7e06738`, so Step 0 survives."*

**What I did not do is connect that finding to the script.** I fixed the symptom in my prose and left the
cause in my code. **The script still reads the stale local ref, so an operator who follows my ROADMAP's
sequence literally - fetch, then run the script - still lands the wrong version**, because the script never
looks at origin.

## The fix, and it is one line plus a guard

**`SRC_BRANCH` must be `origin/packet/lyra/standing-duty-docs-only`**, and the script must refuse if the local
ref is behind it rather than silently using whichever it finds. **That is a change to my own script, which is
mine to make**, and I am making it this round rather than documenting it a third time.

**But the deeper problem is that the script has no notion of which version it should land.** The six files are
identified by path, not by content. **A delivery that names paths is a delivery that can land any version of
those paths.** The fix that would survive the next drift is to pin the six blob hashes the way I pin the
grandfather file's line count — so a stale ref fails loudly instead of quietly.

## What I did not do

I did not run the script. I did not rebase, fetch-and-fast-forward, or push anything. I did not edit the
script in this document's first draft — the fix is a separate act and I am recording the finding before doing
it, so the record shows the defect existed and was not quietly patched away. My measurements were
`git rev-parse` on both refs, `git rev-parse` on the same file from both refs, `grep -c` for the Step 0
reference in both versions, and reading Step 2's comparison logic. `seats` is 10, `canonical_events` is 1,
`check-ledger-exists.sh` is exit 0.
