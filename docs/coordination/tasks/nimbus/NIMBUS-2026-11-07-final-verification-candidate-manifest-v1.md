# T3 — Nimbus — 2026-11-07 — The final verification candidate: three verdicted subjects, blob-verified, with its manifest

Dispatch: `disp-FLKgeRo9NbTBVSTttWWUv`
Branch: `packet/nimbus/candidate-final`
**Code candidate: `6dd68c538d55b04ad1ea12511f8e52147988770f`** — every measurement
below was taken at this commit, and this is the commit Flux should verify.
Branch tip: one documentation commit on top of the code candidate, adding this
manifest and changing no code. Its hash is deliberately not written here — an
earlier draft named it and went stale on every amend, which is the same class of
self-referential rot as a `rerun-if-changed` path that does not exist.
Base: track `0d6d730` (current, not an earlier tip).

To verify the code exactly, check out `6dd68c5` or confirm it is the parent of the
branch tip.

## The candidate, in one line

Built from current track, containing the exact approved file sets of three
subjects, every blob hash-verified against its approving commit, with the
deletion guard deliberately absent and no install, replacement, restart, or
live-database write performed.

## The three subjects and their verdicts

| # | Subject | Verdict | Commit | Blobs |
|---|---|---|---|---|
| 1 | build-embedded identity (`BUILD_GIT_OK` reports command success) | `ev-D6iiDBZBVxA__dhCiK2eq` (pass) | `5af0e16` | 3 |
| 2 | sandbox replacer preparation | `ev-M_SmZjyZ6eJjJ4eYWxqNH` (pass) | `77a7c5f` | 6 |
| 3 | build-failure regression fixtures | `ev-CFa3FvN_MxfsujsOBuo0J` (pass) | `9de5fb5` | 2 |

**All three subjects have verdicts. Nothing is missing one, and no verdict has been
reused for a subject it does not name.**

## Blob verification — every file against its approving commit

| File | Blob | Source | Match |
|---|---|---|---|
| `src-tauri/build.rs` | `7c43300a2a20` | `5af0e16` | OK |
| `src-tauri/tests/build_identity_incremental.rs` | `2027b2ee30f5` | `5af0e16` | OK |
| `ui/src/components/SandboxBanner.tsx` | `dcb4811c3282` | `5af0e16` | OK |
| `ui/src/components/SandboxBanner.test.tsx` | `ae25a58dbd7b` | `77a7c5f` | OK |
| `scripts/replace-sandbox.sh` | `871109d8e2bf` | `77a7c5f` | OK |
| `scripts/worktree-root.sh` | `aba26d61253f` | `77a7c5f` | OK |
| `scripts/test-worktree-root.sh` | `784d17ffe41a` | `77a7c5f` | OK |
| `scripts/test-replace-sandbox-failclosed.sh` | `e27849cbd690` | `77a7c5f` | OK |
| `src-tauri/src/commands/runtime_cmds.rs` | `b0ff0d899760` | `77a7c5f` | OK |
| `scripts/prepare-sandbox-build.sh` | `52bfdaf0b24e` | `9de5fb5` | OK |
| `scripts/test-build-failure.sh` | `92209cb1906b` | `9de5fb5` | OK |

11 of 11 byte-identical. Taken by `git checkout <commit> -- <file>`, never by branch
diff — a branch diff carries whatever else that branch merged.

## Two conflicts, both resolved by measurement and both recorded

### 1. `prepare-sandbox-build.sh` has two approved blobs

`d318bf1f93` (subject 2) and `52bfdaf0b2` (subject 3, which adds
`mkdir -p "$WT/target"` before the first log write).

**Measured:** with subject 2's blob, the fixture suite **fails** — its own mutation
anchor is gone and it reports that the mutation anchor was not found, which is the suite correctly
refusing to certify code that cannot be mutated the way it pins. With subject 3's
blob, all 18 checks pass.

**Resolution:** the candidate carries `52bfdaf0b2`. **Consequence, stated plainly:
subject 2's verdict does not cover this file as it appears in the candidate.** It
measured `d318bf1f93`. Subject 3's verdict does cover it — its criterion
`mutation_target_mkdir_removed` is RED against exactly this blob.

### 2. `SandboxBanner.test.tsx` has two approved blobs

`6cc34be110` (subject 1, verified by both `ev-D6iiDBZB` and, at `889d687`, by
`ev-yhzd26x4TKraRhJXbgVuo`) and `ae25a58dbd` (subjects 2 and 3, which adds
parameter types and drops a duplicate import).

**Measured — this one matters:**

| Blob | vitest | tsc |
|---|---|---|
| `6cc34be110` | 20 passed | **6 errors**, including `TS2300: Duplicate identifier 'cleanup'` |
| `ae25a58dbd` | 20 passed | **0 errors** |

**Both verdicts measured vitest, which does not type-check.** So the blob they
approved cannot pass `tsc`, and the verdicts' UI gates did not notice.

**Resolution:** the candidate carries `ae25a58dbd`, the strictly better blob,
byte-identical in subjects 2 and 3. **Consequence, stated plainly: no verdict's UI
gate covers it.** `9f0e50f`, which produced it, is named by no verdict — it is
mentioned in subject 2's `the_delta` prose, but that verdict's own `scope` field
says the identity subject is not re-certified there, and its commit is `77a7c5f`,
not `9f0e50f`. **Flux must re-measure the UI gate on this candidate.**

## Gates on the candidate

| Gate | Result |
|---|---|
| `cargo test --workspace` | **387 passed / 0 failed**, 67 ignored |
| `build_identity_incremental` | 19 passed / 0 ignored |
| `scripts/test-replace-sandbox-failclosed.sh` | exit 0 |
| `scripts/test-worktree-root.sh` | exit 0 |
| `scripts/test-build-failure.sh` | exit 0 (18 checks) |
| UI suite | **31 files / 398 tests passed** |
| `tsc --noEmit` | **0 errors** |
| `cargo build --workspace` | 0 errors (1 pre-existing warning, `state.rs:18`, track-owned) |
| Deletion guard | **absent**, confirmed: not in the tree, not in the test run |

## The operator handoff — exact commands and measured output

All four run at `6dd68c5`. **Zero side effects throughout**, verified before and
after: main app pid **90805** untouched, sandbox binary mtime **unchanged at 6 Nov
05:33:31**, zero backup entries created, zero sandbox processes.

### 1. Dry-run preview (the default — there is no `--dry-run` flag)

```
bash scripts/prepare-sandbox-build.sh packet/nimbus/candidate-final
```

Measured exit **0**. Output names branch `packet/nimbus/candidate-final`, build
directory `.../nimbus-psr`, and commit `6dd68c5` annotated as the tree that will be built. Main app
`/Applications/SeatLoom.app` is **not** in scope, and the preview states the main app is running at pid 90805 and that this script will not touch it. It also reports
being 1 commit ahead and 0 behind track. Then it stops and prints the two
paths, build-only and `--apply`, separately.

### 2. Build-only (no install, no replacement, no process touched)

```
bash scripts/prepare-sandbox-build.sh --build-only packet/nimbus/candidate-final
```

Measured exit **0**. Produces
`.../nimbus-psr/target/release/bundle/macos/SeatLoom.app`. Its final section is
the identity proof:

```
  commit embedded in the binary : 6dd68c538d55
  commit at the worktree HEAD    : 6dd68c538d55
  MATCH
```

and it states that nothing was installed, nothing was replaced, no process was stopped and none was started.

One advisory it printed and I am not hiding: a warning that no MUST_HAVE string was supplied, so that check was not verified, with a suggestion to supply one.
— no `MUST_HAVE` was supplied, so the "this build carries that work" check was not
run. If the operator wants it, supply one:
`bash scripts/prepare-sandbox-build.sh --build-only packet/nimbus/candidate-final MUST_HAVE_ME`.

### 3. Replacement preview (dry-run is the default)

```
bash scripts/replace-sandbox.sh --from-branch packet/nimbus/candidate-final
```

Measured exit **0**. Lists the seven ordered steps — backup with `BACKUP-META.txt`
describing the **old** sandbox, verify the backup before destroying anything, stop
only the sandbox process (matched on `seatloom-sandbox`, never the main app),
`rm -rf`, copy in, patch `Info.plist` and icon, start, verify the banner commit
against `6dd68c538d55` — then states that the main app is not in that list and that no step touches it.
and that the preview stops there, with `--apply` being the authorised action for a real replacement.

### 4. Rollback preview

```
bash scripts/replace-sandbox.sh --rollback --list
```

Measured exit **0**, reporting that no backups exist yet, which means the sandbox has never been replaced by this script. Correct and
expected — no replacement has been performed, so there is nothing to roll back to.
**This is also why `--rollback` on its own exits 1**: it requires a backup
directory, and there is none. That refusal is the guard working.

## What is NOT authorised, and what Flux must measure

**Nothing here is an authorisation.** No card, no merge to track, no install, no
replacement, no restart, no live-database write, no sandbox replacement.

**Flux must re-measure on `6dd68c5`, and not reuse the three passes:**

1. The **UI gate**, because the approved blob failed `tsc` and the replacement blob
   is covered by no verdict.
2. `prepare-sandbox-build.sh`, because the version in the candidate is subject 3's
   blob, not the one subject 2's verdict measured.
3. The **combination** — three passes on three branches is not a pass on one branch.

## Not done

- No track merge, no live-database write, no `/Applications` replacement, no kill or
  restart, no reopening of the cancelled card.
- The `MUST_HAVE` string check was not exercised; flagged above rather than left
  silent.
- I did not re-run Flux's mutations. This dispatch asked for the candidate and the
  handoff, not for re-verification.
