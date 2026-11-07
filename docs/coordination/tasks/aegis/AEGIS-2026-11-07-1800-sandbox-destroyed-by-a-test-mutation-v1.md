| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-07-1800-sandbox-destroyed-by-a-test-mutation |
| status | delivered |
| author | aegis |
| date | 2026-11-07 |
| version | v1 |
| tags | aegis, incident, honesty, sandbox |

# Incident: a verification mutation replaced the sandbox application with a 95-byte fake

## What happened, measured by me at 18:00

Flux's verdict `ev-_uI4Dit7TkTWXJfDq44RO` came back **fail** on the `--apply` fixtures, and
it recorded that while re-running a mutation it had replaced
`/Applications/SeatLoom-Sandbox.app` with a fake binary. I verified this myself rather
than taking it from the verdict:

| measurement | result |
|---|---|
| `/Applications/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox` | **95 bytes**, a text file: `seatloom-build-commit=ee8a2dc42ee2` + `MUST_HAVE_PRESENT_MARKER` |
| bundle contents | only that file plus an `Info.plist` — not an application |
| mtime | Nov 7 17:34:07 |
| backup directory `~/SeatLoom-Sandbox-backups/` | **does not exist** — nothing to roll back to |
| sandbox process | none running |
| **main app** `/Applications/SeatLoom.app` | **untouched** — pid 90805, real 12.7 MB binary, Oct 28 |

So the sandbox application is gone. It is not damaged, it is *replaced* — the path now
holds a text file that a test wrote.

## Root cause, verified in source at `0a7f1a7`

`scripts/prepare-sandbox-build.sh` line 68:

```
SAND_APP="/Applications/SeatLoom-Sandbox.app"
```

and `SEATLOOM_SANDBOX_APP` appears **zero** times in that file. Meanwhile
`scripts/replace-sandbox.sh` line 46 does honour
`SAND_APP="${SEATLOOM_SANDBOX_APP:-/Applications/SeatLoom-Sandbox.app}"`.

So the fixture's redirect was inert for this script. Its mutation run executed the real
`--apply` path, which reached `rm -rf "$SAND_APP"` and `cp -R "$BUILT" "$SAND_APP"` on the
hardcoded real path.

**This is the same defect class we have now hit several times:** a control that looks
wired and is not. It is exactly the `MUST_HAVE` defect from `6dd68c5` — a parameter that
exists, is documented, and is silently discarded — with the difference that this one
deletes a real application instead of merely printing an advisory.

## What was lost, and what was not

**Not lost:** the main application, the live ledger, the git history, every commit, and
all the verified source work. I checked all of these.

**Lost:** the sandbox application only. And it was already not useful: Flux measured that
it was built from `1cee50c` on Nov 6 05:33, it did not contain the deletion guard, and it
could not state its own commit. I also checked the three other real built apps in the
worktrees — **none of them contains `merge_would_delete` either**, so no existing build
anywhere on this machine demonstrates the deletion guard.

## Why this happened on my watch

I dispatched a verification whose mutation would run the destructive path, on a fixture I
did not first check for a hardcoded real path. The dispatch said "isolated app roots and
commands" — and the script ignored the override that would have made that true. My
instruction assumed the isolation existed instead of verifying it. **A verifier who is
told "use isolated fixtures" and given a fixture that cannot isolate will measure nothing
and destroy something.**

## What I have done

1. Dispatched the fix at highest priority: make `SAND_APP` overridable in
   `prepare-sandbox-build.sh` in the same form the other script already uses; make the
   destructive path refuse when the target is the real `/Applications` path without an
   explicit acknowledgement; make the mutation fixture redirect to scratch; and prove by
   measurement that no mutation run can write outside the fixture.
2. Instructed Nimbus explicitly **not** to restore or rebuild the sandbox — that is a
   separate act needing your authorisation.

## What I have NOT done, and will not do without you

- I have not restored, rebuilt, or replaced the sandbox.
- I have not touched the main application.
- I have not merged anything, opened any card, or written to the live database.

## What you may want to decide

The sandbox is gone and there is no backup. Rebuilding it needs a real build plus an
install, both of which are authorised acts. Nothing on the critical path currently
depends on the sandbox — but the deletion guard cannot be demonstrated to you until a
sandbox exists, and I will not claim otherwise.
