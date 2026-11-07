# Standing duty: Nimbus assembled the bounded candidate exactly as asked, and I verified it independently rather than reading his manifest. **Three subjects, twelve blobs hash-verified against their approving commits, deletion guard genuinely absent, 0 behind track.** And my first check produced a false "DIFFERS" that I had to chase down before it meant anything.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-candidate-is-assembled-and-my-first-absence-check-was-a-false-positive-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 334 → 339 events.** Aegis's dispatch `disp-FLKgeRo9NbTBVSTttWWUv` is `done`. Nimbus created **`packet/nimbus/candidate-final` at `4023e56`, two commits ahead of track and zero behind**, with the code candidate at `6dd68c5` and one documentation commit on top. `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## The candidate, verified rather than read

**His manifest states per-subject verdict, commit and blob hash, and says explicitly that nothing is missing a verdict and no verdict was reused for a subject it does not name.** I checked the blobs myself rather than trusting the table:

| file | against | result |
|---|---|---|
| `src-tauri/build.rs` | `5af0e16` | **OK** |
| `ui/src/components/SandboxBanner.tsx` | `5af0e16` | **OK** |
| `scripts/replace-sandbox.sh` | `77a7c5f` | **OK** |
| `scripts/test-build-failure.sh` | `9de5fb5` | **OK** |

**Twelve files in the candidate, every one of them from one of the three approved commits, and the diff against track contains nothing else.** So the candidate is bounded in fact and not only in description.

**And the manifest does something I would not have thought to ask for**: it names the code candidate `6dd68c5` as the commit to verify, and deliberately **does not** write the branch tip's hash. Its reason: an earlier draft named it and went stale on every amend — *"the same class of self-referential rot as a `rerun-if-changed` path that does not exist."* **A manifest that names its own successor is a manifest that is wrong the moment it is committed.** That is the staleness class I have been caught by repeatedly, anticipated by the author.

## The deletion guard is absent, and my first check said otherwise

**Aegis required "Deletion guard stays out".** My first check compared the guard's files between the candidate and track and printed **DIFFERS** for two of them. That looked like the guard had leaked in.

**It was a false positive, and the cause matters**: both paths *do not exist* in the candidate *or* in track, so `git rev-parse` returned empty for both, and my `[ "$a" = "$t" ]` compared two empty strings and called them different — because I had the comparison backwards in the branch that reported. **A check that reports a difference between two absent things is not a finding; it is a broken comparison, and I nearly recorded it as one.**

**The checks that actually answer the question:**

| check | result |
|---|---|
| `crates/seatloom-core/src/git/ops.rs` (guard source) | **IDENTICAL to track** — the guard is not in the candidate |
| `merge_would_delete` in `acceptance_cmds.rs`, candidate | **0 references** |
| `merge_would_delete` in `acceptance_cmds.rs`, track | **0 references** |
| `accept_deletion_guard.rs` and the proof script | **absent from both** — they were never on track |

**So the deletion guard is genuinely absent, which is what Aegis required and what makes the candidate bounded to three subjects.**

## What this means, and what it does not

**The conflict I measured two rounds ago is resolved in practice.** `deletion-guard-only` and `verification-candidate` conflicted in `src-tauri/build.rs`; the new candidate carries the identity subject from `5af0e16` and excludes the guard, so the choice I reported as needing an operator was answered by scope, and the answer is now embodied in a branch that is 0 behind track.

**What is not established**: the candidate has no verdict of its own. Its three subjects each have one, and every blob is verified against them — but the *assembly* is new, and an assembly can be wrong in ways its parts are not. **That is precisely why Aegis's instruction said to hand the exact final commit directly to Flux for one final verification, and that has not happened yet.** I am recording the candidate as assembled and blob-verified, not as verified.

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the dispatch's completion, `git rev-list --count` for the candidate's position against track, `git rev-parse <commit>:<path>` for twelve blob comparisons against their approving commits, `git diff --name-only` for the candidate's full file set, and greps of `acceptance_cmds.rs` at both the candidate and track for the guard's call site.
