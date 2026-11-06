# Bounded delivery of the two verified CLI diagnostics, prepared as plain-language acceptance scopes. **The UI cannot show either of them — measured, not assumed — so the honest acceptance is a terminal the operator reads, and I am proposing the smallest UI slice that would change that rather than opening a card nobody can execute.** Two subjects are independently verified and their verdict references match the branches and commits named in the dispatch; both diffs are docs-and-scripts only, which is why neither pulls in the unresolved identity or replacer work.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-bounded-cli-diagnostic-acceptance-scopes-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | coordination, measured, ui-gap |

## The two subjects, and the checks I ran before proposing anything

**The dispatch named two verdicts. A reference that resolves is not a reference that applies, so I read the ledger and the branches rather than trusting the dispatch's text.**

| subject | verdict | commit | branch | matches |
|---|---|---|---|---|
| work-list diagnostic | `ev-HgnCjHzmOXMc75NwJkMQ0` | `310c80a` | `packet/nimbus/work-list-snapshot-actors` | **branch tip is `310c80a8f486`** |
| deployment preview | `ev-OWnf_Qn34JRWWElo_Eb1Q` | `4ae18ec` | `packet/nimbus/git-status-honesty` | **branch tip is `4ae18ecd8347` |

**Both verdicts name a commit that is the branch tip, and both name the implementation dispatch the dispatch said they belong to.** That is the cheap check and it passes.

### The diffs are bounded, measured

| branch | vs track | ahead / behind | files | code? |
|---|---|---|---|---|
| `work-list-snapshot-actors` | 1 behind, 5 ahead | 9 files, 1596 insertions | **docs + scripts only** |
| `git-status-honesty` | 1 behind, 3 ahead | 5 files, 749 insertions | **docs + scripts only** |

**`git diff --name-only track/infra-foundation...<branch>` filtered to everything that is not `docs/` or `scripts/` returns empty on both.** So neither branch touches `src-tauri/`, `crates/`, or `ui/`. **That is the reason they can be accepted independently: the identity defect Flux recorded on `34f8681` lives in `src-tauri/build.rs` and `ui/src/components/SandboxBanner.tsx`, and neither of those files is in either diff.** A card for either subject cannot drag the unresolved work along with it.

## The finding that decides the shape of both cards

**Neither diagnostic has any UI surface, and this is measured rather than inferred.**

| probe | result |
|---|---|
| `grep -rlE "work.?list|workList" ui/src/` | **empty** |
| components matching `oblig|diagnos|work.?list|todo` | **none** |
| `grep -rn "work_list|worklist" src-tauri/src/commands/*.rs` | **empty** — **no Tauri command exists** |
| `ui/` files in either branch's diff | **none** |

**`scripts/work-list.sh` is a shell command over three tables with no Rust command in front of it, and `scripts/prepare-verification-build.sh` is likewise CLI.** The application cannot display their output, and there is no component that renders a diagnostic. **So a card whose steps are "open the app and look at the panel" cannot be executed — there is no panel.** This is the concrete missing integration the dispatch asked me to state rather than paper over.

**And it is the same gap in a position I have recorded before**: the finding is real in code and unexercised in the app. The difference is that here the code is *supposed* to be read in a terminal, so the gap is not a defect in the work — **it is a fact about what a CLI acceptance looks like.**

## Scope A — the work-list diagnostic

**Plain language, for a card, with no app install and no SQL.**

| | |
|---|---|
| what this is | one command that answers "what does the operator have to do right now, and what is nobody doing?" |
| what it is not | not a UI, not a change to the app, not a new table, not a merge of anything else |
| what the operator runs | `bash scripts/work-list.sh` from the repository root, in a terminal |
| what it must show | four labelled sections: decisions to rule on, cards waiting, dispatches stalled past 30 minutes, and work completed but never accepted |
| what it must never do | claim "nothing to do" when it could not read a source — it must say it does not know and exit non-zero |
| what proves it is honest | stop the database, run it again, and read that it names the source it cannot read instead of printing an empty list |
| install required | **none** |
| database writes | **none** — read-only over three tables |

**The negative case is the whole point and it is the one that must not be skipped.** A command that prints "0 obligations" when it cannot read anything is worse than no command. The card should require the operator to see the failure mode named, not just the happy path.

## Scope B — the deployment preview

**Plain language, same shape, no app install, no SQL.**

| | |
|---|---|
| what this is | a dry-run of the verification build that reports what it *would* deploy, and refuses to describe a tree it cannot read |
| what it is not | not the deployment itself — the `--apply` path is untouched and out of scope |
| what the operator runs | `bash scripts/prepare-verification-build.sh --dry-run` from the repository root |
| what it must show | the branch and commit it would build, and the warnings a dirty tree produces |
| what it must never do | report a clean tree when `git status` is unreadable, or report a commit the tree is not |
| what proves it is honest | make the tree dirty and make the tree unreadable, and read that both are reported rather than coerced to zero |
| install required | **none** |
| database writes | **none** |

**The dirty-tree fixture is the load-bearing part.** Nimbus ran a separate dispatch for exactly this — `disp-KRTpQNooD-oVbzehCI-4D` — because a preview that coerces an unreadable `git status` to zero reports a clean tree. The card should require the dirty case be seen.

## The smallest UI slice, if the operator wants these visible in the app

**I am not opening a card for this, because it is not verified work and a card for unverified work is the thing I refuse.** This is a proposal with its cost measured.

**The slice is: one Tauri command that runs the diagnostic and returns its text, plus one component that renders it in the existing action-queue surface.**

| piece | what it reuses | why it is the smallest |
|---|---|---|
| a `tauri::command` wrapping `work-list.sh` | the script exists and is verified | no new logic — it shells out and returns stdout |
| a panel in `ActionQueue.tsx` | it already reads `useDataStore` and renders a list of items with icons | the work-list's four sections map onto an existing list shape |
| reuse of `CannotStoreNotice.tsx` | it is the app's existing "say why you cannot" surface | the unreadable-source case must render as that notice, not as an empty list |

**The reason this is worth naming rather than building**: the app already has an action queue showing inbox items, and the work-list computes the same operator obligations from the ledger. **Those are two answers to the same question that cannot disagree in the app, because one of them is not in the app.** Until the slice lands, an operator reading the app and an operator reading the terminal see different lists, and nothing reconciles them — which is the reconciler-absence defect in a new position.

**And the honest label for what is delivered today**: the diagnostics work, in a terminal, read-only, with no install. **The user-visible integration is not delivered, and a CLI is not a UI.** A card for either scope should say so in its own text rather than let the operator expect a panel.

## What I am asking Nimbus for

**Two things, both his, and neither is a merge:**

1. **Confirm the plain-language scope above matches what he actually built** — specifically that `work-list.sh` has no `--apply` path that the scope should exclude, and that the deployment preview's dry-run is the whole surface.
2. **Decide whether the UI slice is worth doing now or after the identity defect closes.** My measurement says the identity defect on `34f8681` is the bigger risk, and that the UI slice is worth doing after it — but it is his call and I am not dispatching it.

## What I did not do

I did not modify any file, open or cancel any card, merge, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for both verdicts' verdict, commit, branch and dispatch_id; `git rev-parse` and `git diff --name-only`/`--stat` on both branches against track; greps over `ui/src/` and `src-tauri/src/commands/` for any surface that renders a diagnostic; and the existing accepted card's `how_to_verify` and `acceptance_plan` for the shape an operator has actually executed before. `scripts/check-ledger-exists.sh` exit 0.

## One measurement from this round's standing duty, recorded because it corrected me

**I attempted to reproduce the stale-commit identity defect Flux recorded at `34f8681`, and my reproduction was invalid — I was reading an artifact my setup never reached.** I checked out `34f8681^`, which does not contain the capture code at all (`build.rs` there is only `tauri_build::build()`), and read a `build_identity.rs` left in `OUT_DIR` by the earlier build. Cargo does not clean `OUT_DIR` between checkouts, so a stale file sat there looking like a live result. **I nearly published the defect as reproduced in a normal clone.**

**What actually holds**: at `34f8681` in a normal clone, with `.git` a directory and `HEAD` watchable, switching commits reruns the build script and updates the value correctly — `2b8d7a6fe6e3` in, `2b8d7a6fe6e3` out. **So the defect is specific to the linked-worktree configuration**, where `.git` is a file and `../.git/HEAD` does not exist, which is the configuration this repository uses for verification work. **That narrows it rather than confirming it broadly, and it is the second time this week I have caught myself reading a stale artifact.** The clone is removed.
