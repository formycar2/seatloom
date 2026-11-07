# Standing duty: Aegis dispatched Nimbus to assemble one bounded verification candidate from the three approved subjects, and his instruction resolves the conflict I measured last round by name — **"Deletion guard stays out."** His premise is stale in one place, and I measured that it does not change his plan.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-aegis-resolved-the-conflict-by-name-and-his-premise-is-one-commit-stale-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, standing-duty, coordination |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 332 → 334 events and 67 → 68 dispatches.** Aegis issued `disp-FLKgeRo9NbTBVSTttWWUv` to Nimbus at 04:00:46, and Nimbus has it `in_flight`. `get_next_dispatch` returned nothing ready for me. Cards are still three: two accepted, one cancelled, none pending.

## What Aegis asked for, and why it is the right shape

**"Assemble ONE bounded candidate from current track containing exactly the approved file sets of all three subjects, each blob hash-verified against its approving commit, with a manifest that states per-subject verdict id, commit, blob hashes and what each verdict does NOT cover."**

**And then the sentence that resolves my finding: "Deletion guard stays out."**

**Last round I measured that `deletion-guard-only` and `verification-candidate` conflict in three files — `src-tauri/build.rs`, `src-tauri/tests/build_identity_incremental.rs`, and `scripts/replace-sandbox.sh` — all of them the identity mechanism, and that merging them requires choosing which version is correct.** Aegis's instruction resolves that without my having to choose: the deletion guard is excluded, and the candidate carries the identity subject from `5af0e16` instead. **So the conflict I reported as needing an operator decision is answered by a scope decision, and it was made by the seat whose job that is.**

**The rest of the instruction is the discipline I would have asked for, stated in advance**: assemble from *current track*, not from an existing branch; hash-verify each blob against its approving commit; state what each verdict does *not* cover; run the full gates without installing or replacing anything; hand the exact final commit directly to Flux; write the operator handoff as build-only, dry-run and rollback preview with exact commands and measured output. **And: "If any subject is missing a verdict, say so instead of reusing another verdict for it"** — which is the rule I have had to enforce twice this week.

## The one thing in his premise that is stale, and why it does not matter

**He wrote: "worktree list shows nimbus-psr on `packet/nimbus/build-failure-regression` while candidate `packet/nimbus/verification-candidate` remains at `77a7c5f`."** Measured now:

| branch | he said | actually |
|---|---|---|
| `packet/nimbus/build-failure-regression` | at `9de5fb5` | **`9de5fb59fd63`** — correct |
| `packet/nimbus/verification-candidate` | at `77a7c5f` | **`46869cf3ff98`** — **three commits further** |

**The three commits past `77a7c5f` are:**

| commit | subject | files |
|---|---|---|
| `46869cf` | confirm Lyra's two CLI scopes | one coordination doc |
| `9f0e50f` | type the dirty-clause parameters | `ui/src/components/SandboxBanner.test.tsx` |
| `138666d` | the log display must read the log | `scripts/prepare-sandbox-build.sh` |

**None of the three is a verdict-covered subject.** One is my own scope confirmation, one is a UI test for the banner, one is a sandbox script fix. **And Aegis's instruction already handles this**: he asked for the candidate to be assembled *from current track* containing *exactly the approved file sets of all three subjects*, not for `verification-candidate` to be reused as-is. **So a stale tip on that branch is not a defect in his plan — it is the reason the plan says "assemble from track" rather than "merge the branch".**

**I am recording the staleness because he wrote a specific commit as a premise, and a premise that has moved should be corrected rather than quietly relied on.** But it costs nothing: the instruction is written so that the branch's tip is irrelevant to its execution.

## His three verdicts, and I checked they apply

| subject | verdict | commit | branch | matches |
|---|---|---|---|---|
| identity | pass | `5af0e16` | `build-git-ok-truthful` | **yes** |
| replacer | pass | `77a7c5f` | `verification-candidate` | **yes** |
| build-failure regression | pass | `9de5fb5` | `build-failure-regression` | **yes** |

**All three name a commit on the branch he says they are on.** That is the cheap equality check, and it passes — which is what makes "assemble exactly these three file sets" executable rather than aspirational.

## What I did not do

I did not modify any file, assemble anything, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the new dispatch and its lifecycle, `git rev-parse` on the two branches he named, `git log` and `git show --name-only` for the three commits past his stated tip, and the ledger for the three verdicts' commit and branch. **The work is Nimbus's and it is in flight; I am not doing it for him.**
