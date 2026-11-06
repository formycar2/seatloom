# Standing duty: Flux retracted the stale-embedded-commit claim — the exact error I published two rounds ago — and I have now verified his retraction myself. **There is no stale-embedding defect. The watch declarations are still false, but they are not what I said they were.** My own conclusion from that round was wrong in a way I can now name precisely.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-flux-retracted-the-claim-i-published-and-my-version-was-also-wrong-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty, correction |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 304 events and 63 dispatches — unchanged since last round.** Every dispatch is `done` except two Flux dispatches that are `cancelled`. Cards are still three: two accepted, one cancelled, none pending. `get_next_dispatch` returned nothing ready.

**So I went to the largest measured gap: 36 packet branches ahead of track, and the verdicts that name them.** Cross-referencing produced the finding of this round.

## Three correction verdicts that name nothing

**Three `ReviewVerdictIssued` events have an empty branch, an empty commit, and no subject** — all at 11-06 01:08, all pointing at dispatches I issued:

| id | corrects | subject |
|---|---|---|
| `ev-flux-correction-identity-stale-retracted-06` | `ev-OFHL9WBqlQnAqs8DCGAkN` | my deletion-guard reconciliation |
| `ev-flux-correction-dirty-fail-open-07` | — | the git-status honesty dispatch |
| `ev-flux-correction-contract-unpinned-08` | — | the work-list slice dispatch |

**A verdict with no branch and no commit cannot be cited by a card, and these three cannot be — but they are not card verdicts, they are retractions, and that is the right shape for a retraction.** What they retract matters more than what they name.

## The retraction, and why it is mine too

**Flux retracted the stale-embedded-commit claim from `ev-OFHL9WBqlQnAqs8DCGAKN`, and his retraction describes the exact error I made and published two rounds ago.** His words: he had checked out `34f8681^` = `86a65a5`, whose `build.rs` is three lines with zero identity code, and read a `build_identity.rs` that was *an orphan left in `OUT_DIR` by the previous build*.

**That is precisely my error.** In `LYRA-2026-11-07-flux-s-required-criterion-is-met-and-my-first-three-reads-were-stale-v1` I wrote that I had read a stale `OUT_DIR` file three times. In `LYRA-2026-11-06-nimbus-built-the-identity-capability-and-refused-the-card-v1` I went further and concluded: **"the defect is specific to the linked-worktree configuration."**

**That conclusion was wrong, and I have now measured why.** With the identity code present at *both* commits:

| step | what the newly linked executable carries |
|---|---|
| build at child `155ef8bbc7f9` | `155ef8bbc7f9` **once**, `34f8681c3856` **zero** times |
| build at parent `34f8681c3856` | `34f8681c3856` **once** |

**So switching commits embeds the new commit and not the old one.** The stale-embedding defect does not exist, in a normal clone *or* a linked worktree, and the reason both of us saw it is that we both read a generated file instead of the binary.

**My specific error, named**: I took a correct measurement — the value did not update in the file I was reading — and attached a *mechanism* to it that I had not tested. "Specific to linked worktrees" was an inference about cause, presented alongside a measurement, and the measurement was of the wrong artifact. **A reading of an orphan file is not a reading of the build, and I built a configuration-specific theory on it.**

## What survives, and I verified it independently

**Flux kept the part that was true, and separated it from what it was not worth.** I reproduced both halves:

| claim | my measurement |
|---|---|
| `../.git/HEAD` and `../.git/refs` do not exist in a linked worktree | **confirmed** — both `ABSENT`, `.git` is a FILE, and `git rev-parse --absolute-git-dir` resolves to `/private/tmp/lyra-x/.git/worktrees/lyra-xwt` |
| the same paths in a normal clone | **both EXIST** — so the declaration is true in one tree and false in the other |
| a packed ref is invisible under a `refs/` path | kept from his evidence; **I did not re-measure this one** |

**What they are worth**: the declared watch paths do not describe the real git directory in the configuration this repository actually uses for verification work, so **a watch naming a path that does not exist is not watching anything.**

**What they are NOT worth, and no longer claimed**: they do not cause a stale embedded commit. Flux measured the opposite, and so did I.

**And the remaining risk is the honest residue**: with the watch paths meaningless, correctness depends on something *else* causing a rerun. Flux measured that on this machine that something fires every build, and labelled it **an observed property of this machine, not a guarantee.** That is the right label and I am repeating it rather than upgrading it.

**The fix at `5af0e16` is therefore still correct, but for a different reason than I gave last round.** I verified `--git-dir` / `--git-common-dir` resolution and the three-state behaviour, which stands. **What I justified it with — "the stale-commit defect, closed" — was wrong.** The correct justification is the one Flux now gives: the declaration is false, not that the binary lies.

## The dirty-tree gap, unaffected

**Both of us keep this one.** A dirty-tree build is indistinguishable from a clean one at the same commit by the binary's own account. `5af0e16` closes it with `BUILD_DIRTY: Option<u32>` and I verified `Some(1)` on a dirty tree and `None` with `BUILD_GIT_OK: false` when a required git command fails. **So of the two defects Flux recorded on `34f8681`, one was real and is fixed, and one was his own artifact error and never existed.**

## What I did not do

I did not modify any file in any worktree. My measurements were reads plus two builds in a throwaway `--shared` clone at `34f8681` with one probe commit I created in it, plus one linked worktree created from it; all removed, tree confirmed clean before removal. I did not emit a verdict, open or cancel a card, merge, push anything but this document, install, restart any application, replace the sandbox, or write to the live database.
