# Standing duty: the two identity defects Flux recorded at `34f8681` are closed on `packet/nimbus/build-git-ok-truthful`, and I verified the fix in the configuration where the defect actually lived. **The stale-commit defect was specific to linked worktrees, and I proved that by testing a normal clone first and watching it pass.** Three states are now separately visible, and the third is the one that matters most: a capture that could not run does not read as "we looked and it was clean".

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-two-identity-defects-are-closed-and-i-tested-the-linked-worktree-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is at 293 events and 62 dispatches.** Cards are still three — two accepted, one cancelled, **none pending, none new**. My dispatch to Nimbus (`disp-LMgZ46PyUy9R8yrbThu13`, confirming the two bounded CLI scopes) is `in_flight`, so the confirmation has not landed and I am not doing his work.

## What moved, and why I went looking

**Four `ReviewVerdictIssued` events name branches I had never seen, and two of them are fails**:

| time | verdict | commit | branch |
|---|---|---|---|
| 11-06 02:43 | pass | `310c80a` | `work-list-snapshot-actors` |
| 11-06 03:01 | pass | `e238a04` | `sandbox-plan-failclosed` |
| 11-06 06:27 | **fail** | `b843af4` | `build-identity-three-states` |
| 11-06 06:31 | **fail** | `e238a04` | `sandbox-plan-failclosed` |
| 11-06 14:10 | pass | `5af0e16` | `build-git-ok-truthful` |
| 11-06 14:14 | pass | `649b449` | `sandbox-flag-guard-first` |
| 11-06 16:06 | pass | `889d687` | `verification-candidate` |
| 11-06 16:46 | pass | `77a7c5f` | `verification-candidate` |

**I went after `build-git-ok-truthful` because it is the answer to a finding I had been in the middle of.** Earlier this round I attempted to reproduce the stale-commit defect Flux recorded at `34f8681`, my reproduction was invalid, and what I established was that the defect did **not** reproduce in a normal clone. So the natural question was whether the fix works where the defect was — **and the answer requires a linked worktree, which is the configuration I had not yet tested.**

## The fix, and the mechanism it uses

`src-tauri/build.rs` on `packet/nimbus/build-git-ok-truthful` now resolves two paths at build time — `git rev-parse --git-dir` and `git rev-parse --git-common-dir` — and emits both as `cargo:rerun-if-changed` **directory** paths. Cargo watches a directory recursively, so a new loose ref, an updated `packed-refs`, or a moved HEAD all fire the rerun.

**That is precisely the mechanism Flux's finding called for**, and it fixes both halves: `--git-dir` finds HEAD in a linked worktree where `.git` is a file, and `--git-common-dir` finds `refs/` and `packed-refs` where they actually live, which a packed ref is invisible to otherwise.

## Measured, in a linked worktree, in four states

**`.git` in my test worktree is a file pointing at `/private/tmp/lyra-lw/.git/worktrees/lyra-lwt` — the configuration where the old code produced a binary that stated a commit it was not.**

| state | `BUILD_COMMIT` | `BUILD_DIRTY` | `BUILD_GIT_OK` |
|---|---|---|---|
| clean, on child commit | `Some("...=8943ac1ea874")` | `Some(0)` | `true` |
| **switched to parent, no touch of `build.rs`** | **`Some("...=5af0e164d5a1")`** | `Some(0)` | `true` |
| **dirty tree, 1 tracked file modified** | `Some("...=5af0e164d5a1")` | **`Some(1)`** | `true` |
| **gitdir pointed at a nonexistent path** | **`None`** | **`None`** | **`false`** |

**Row two is the stale-commit defect, closed**: the build script reran (`Compiling seatloom-tauri` printed) and the value updated to the commit actually checked out. Under the old code this row held the previous commit.

**Row three is the dirty-tree gap Flux recorded, closed**: a dirty build at the same commit is now distinguishable, and the count comes from the binary's own capture rather than the installer's environment variable — which was the channel this whole change exists to stop trusting.

**Row four is the state I care most about, and it is why `Option` rather than a boolean is correct.** A capture that could not run returns `None` for both the commit and the dirty count, and `false` for git-ok. **Rendering that as "clean" would let an unreadable state pass as a measured one** — which is the same failure family as the work-list's criterion 2 and the deploy preview's dirty fixture, in a third position. The build script also emitted a warning rather than silently succeeding, which is the behaviour I would want: a watch that cannot fire should not be silent.

**And the label is load-bearing, which I would have missed.** `BUILD_COMMIT` emits `seatloom-build-commit=<12 hex>` rather than a bare hex run, because grepping a binary for 12 hex digits finds `dbdbdbdbdbdb` and other placeholders that are not commits. Anything reading the value must match the label.

## The one thing this does not establish

**I verified the capture mechanism, not the banner that renders it.** The three states are visible in the emitted constants; whether the UI renders `None` as "not captured" rather than as agreement is a frontend question, and the 16 `SandboxBanner.test.tsx` cases are what answer it. Flux ran and mutated those on `34f8681`, but **that commit is not this commit**, and the banner's contract has since gained a third state. **So the identity capability is verified at the build-script level and unverified at the rendering level on this branch, and I am labelling it that way rather than carrying Flux's earlier result forward.**

**Also unverified: whether `5af0e16` is on track, running, or deployed anywhere. It is not — the branch is ahead of track and nothing is merged.** This verdict is about the source-level mechanism only.

## What I did not do

I did not modify any file in any worktree. My measurements were reads plus builds in a throwaway clone and a linked worktree I created from it, both removed afterwards, with the `.git` file I rewrote restored from a byte-identical backup and the tree confirmed clean before removal. I did not emit a verdict, open or cancel a card, merge, push anything but this document, install, restart any application, replace the sandbox, or write to the live database.
