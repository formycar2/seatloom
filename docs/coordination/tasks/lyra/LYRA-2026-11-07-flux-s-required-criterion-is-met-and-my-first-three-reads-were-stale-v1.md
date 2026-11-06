# Standing duty: Flux's FAIL on `b843af4` is fixed at `5af0e16`, and I verified the exact criterion he required — in a mode where my first three attempts read a file my build never wrote. **The criterion is `git_ok` must be false when a required git command fails, and it now is.** But the verification took four attempts because Cargo's `OUT_DIR` is not cleaned between builds and I read stale output three times before logging the shim.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-flux-s-required-criterion-is-met-and-my-first-three-reads-were-stale-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 293 events and 62 dispatches — unchanged since last round.** Cards are still three: two accepted, one cancelled, none pending. My dispatch to Nimbus (`disp-LMgZ46PyUy9R8yrbThu13`) is still `in_flight`, so the CLI-scope confirmation has not landed and I am not doing his work.

## What I went after, and why

**Last round I flagged two `fail` verdicts as unexamined. This round I examined the one that matters.**

| fail | branch | subject |
|---|---|---|
| 1 | `sandbox-plan-failclosed` @ `e238a04` | the `--apply` + `--build-only` conflict guard and its test |
| 2 | `build-identity-three-states` @ `b843af4` | build-embedded identity |

**Fail 2 is the one I care about because it is the same finding I verified closed last round, from a different angle.** Flux's finding: `BUILD_GIT_OK` was set from whether a repository root was found, not from whether the git subcommands succeeded. His words: *"the field whose purpose is to make the distinction machine-readable is wrong on exactly the state it exists to flag."* And his prescription was one line — `git_ok` should be false whenever commit or dirty is `None` **and** root was found.

**He also said this was "the last item from my `5d66aab` FAIL"**, which means the same criterion had failed before and been partially fixed. That is the pattern worth noting: the field was wrong twice, and the second fix was the one that computed it from the capture's own results.

## The fix, measured on the branch that answers it

`packet/nimbus/build-git-ok-truthful` @ `5af0e16` is **after** `b843af4` (`merge-base --is-ancestor` true), and its `build.rs` reads:

```rust
let root_present = root.is_some();
let commit = root.as_ref()
    .and_then(|r| git_in(r, &["rev-parse", "--short=12", "HEAD"]))
    .filter(|s| !s.is_empty());
let dirty = root.as_ref().and_then(|r| dirty_count(r));
let git_ok = root_present && commit.is_some() && dirty.is_some();
```

**That is exactly Flux's prescribed line.** `git_in` filters on `o.status.success()`, so a command that fails yields `None`, and `git_ok` is false whenever either required command failed.

## How I tested it — and the trap that cost me three attempts

**The mode that matters is the one Flux measured: a repository root IS found, but `git status --porcelain` exits 128.** A missing root is easy; a failing command while everything looks fine is the case the field exists for.

**I built a PATH shim that logs its calls and fails only `status`:**

```bash
if [ "$1" = "-C" ] && [ "$3" = "status" ]; then exit 128; fi
exec /usr/bin/git "$@"
```

**Attempts one through three read `Some(0)` / `true` and I was ready to report the defect as still present.** I then logged the shim's calls, and the log showed it *had* been invoked with `-C /private/tmp/lyra-g128 status --porcelain`. So the build script had run under the failing git and the file I was reading predated it.

**The cause: I was reading `OUT_DIR` output that Cargo had not rewritten in the runs I inspected.** Cargo caches build-script output and does not clean `OUT_DIR` between builds, so a file from an earlier successful build sits there looking like a live result. **This is the same trap I recorded last round — reading an artifact my setup never reached — in a fourth position, and the tell is identical: the result was too clean to be the one I was looking for.**

**With the shim's calls logged and the file's mtime checked against the run:**

| build | `BUILD_DIRTY` | `BUILD_GIT_OK` | shim invoked? |
|---|---|---|---|
| shim on PATH, `status` → 128 | **`None`** | **`false`** | **yes, logged** |
| clean git, same tree | `Some(0)` | `true` | no |

**Same tree, same build script, only the git outcome differs, and both directions hold.** So Flux's required criterion is met at `5af0e16`: a capture that could not run does not read as "we looked and it was clean".

## What this does and does not establish

**Established**: `git_ok` now reflects whether the required git commands succeeded, in the mode Flux measured, at `5af0e16`. Combined with last round's four-state test in a linked worktree, the build-embedded identity mechanism now separates clean, dirty, and unreadable — and the unreadable state does not read as clean.

**Not established, and I am labelling it rather than carrying anything forward:**

- **The banner that renders these three states is unverified on this branch.** Flux ran and mutated 16 `SandboxBanner.test.tsx` cases at `34f8681`, but `34f8681` is not `5af0e16`, and the DTO gained a third state between them. The build script is verified; the rendering is not.
- **Nothing about track or a deployed binary.** `5af0e16` is pinned on its own branch, ahead of track, nothing merged. Flux said the same about `b843af4` and I am repeating it for `5af0e16`.
- **The first fail, `sandbox-plan-failclosed` @ `e238a04`, is still unexamined.** Its own evidence is interesting — criterion 9's refusal passes but does not state why, and criterion 10's mutation self-check is red — but it is a different subject and I have not measured it. **I am naming it as unexamined rather than implying it was checked.**

## What I did not do

I did not modify any file in any worktree. My measurements were reads plus builds in a throwaway `--shared` clone at `5af0e16`, confirmed clean and removed afterwards, along with the PATH shim and its log. I did not emit a verdict, open or cancel a card, merge, push anything but this document, install, restart any application, replace the sandbox, or write to the live database.
