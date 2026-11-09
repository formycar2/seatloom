| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-1600-both-cards-are-merged-and-the-work-runs |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, acceptance, live-check, delivery |

# You accepted both cards. I ran the work on the mainline, and it does what the plans promised

## What happened

Both pending acceptance cards were accepted and merged:

| card | branch | merge |
|---|---|---|
| `acc-disp-GWV4D2pEBdr8cLzQhteTO` | `packet/nimbus/work-list-snapshot-actors` | `7cdac01` |
| `acc-disp-JLsCEk0-uugW-fJBIGyux` | `packet/nimbus/git-status-honesty` | `d2028db` |

1207 insertions and 749 insertions respectively, all on `track/infra-foundation`, all confirmed
present in the tree with `git ls-tree`.

## Protocol step 4b: I did not stop at the merge

The rule from 10-30 is that a merge is the **first** thing, not the last. Code on the mainline is
not the same as work that runs. So I ran both tools on the mainline.

**`scripts/work-list.sh --project seatloom`** — works. It prints the operator's actionable
obligations from the three existing tables, and the part I want to point out: it found
**`kind` values it does not recognise** (`coordination` 19, `research` 2) and listed them as
*"unrecognised kind ... listed as a review candidate, not automatically counted as something owed to you"*. It did not silently drop them and
it did not guess a category. That is the behaviour the plan asked for.

Its required negative case: a project that does not exist exits 1 and says
*"project 'does-not-exist-xyz' does not exist (ProjectNotFound). This is not 'this project has nothing to do' — there is no such project."*
It names the failure instead of printing an empty list.

**`scripts/prepare-verification-build.sh --dry-run`** — works, and it caught something I would have
missed: the build directory holds `track/d2-preview`, not the branch I asked about, so it warned
*"the build directory has track/d2-preview checked out ... what gets installed will be track/d2-preview"*. A dry run that
silently built the wrong tree would be worse than no dry run.

Its required negative case, which I tested with a `git` shim that fails only on `status`:

```
cannot read the working-tree state (git status exit 128)
cannot read the working-tree state — cannot tell whether this build is commit 237ab85, will not install now
```

Exit 1. It refuses rather than reading a failure as "clean" — which is the exact defect the
correction event `ev-flux-correction-dirty-fail-open-07` was about.

## What is now unblocked

These were two of the CLI diagnostics whose UI integration I have repeatedly reported as
**undelivered**. They are on the mainline now, as commands. The screens in the application still
do not show this information, and I am not claiming otherwise.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0 — the merges stranded
  nothing.
- **Zero pending cards.** That is what makes pushing permitted again.
- Sandbox: 95-byte fake, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Push status

The 11 commits now unpushed are the seats' branch commits, merged locally by the application when
you clicked Accept. My own commits were confirmed pushed earlier at `5803b27`. The retry loop is
working on the new ones; I will not claim them until the remote reads back the same hash.

## Needs you

- **The sandbox rebuild** is still the only thing standing between the remaining verified work and
  you. The guard is correct across every path spelling and fails closed, so my side is ready.
- **The UI integration** of these two diagnostics is still an explicit gap. If you want it, it is
  a new dispatch; I am not going to describe a command as a screen.
