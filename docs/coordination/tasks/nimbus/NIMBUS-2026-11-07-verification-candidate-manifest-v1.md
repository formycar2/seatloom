# NIMBUS-2026-11-07-verification-candidate-manifest-v1

- seat: `seat-nimbus-001`
- dispatch: `disp-ptklu6l4R9DlOsW2d0q98`
- branch: `packet/nimbus/verification-candidate`
- base: `track/infra-foundation` @ `93ef3d3` (current tip, verified an ancestor)

## Why this branch exists

Three source fixes now pass **independently**, but the branches they were measured on are
not interchangeable. Measured:

- `track..5af0e16` carries the OLD 267-line `replace-sandbox.sh` and the old sandbox
  test.
- `track..649b449` carries the OLD `src-tauri/build.rs` (no `BUILD_GIT_OK`, no
  `BUILD_DIRTY`) and the old identity fixtures.

So neither whole branch can serve as a candidate. This branch is built from current
track and takes only the **exact approved file sets**, by blob, from the commits that
carry them.

## The two subjects, kept separate

**Subject 1 — build-embedded identity.** Approved at `5af0e16`
(`ev-D6iiDBZBVxA__dhCiK2eq`). Files:

- `src-tauri/build.rs`
- `src-tauri/tests/build_identity_incremental.rs`
- `src-tauri/src/commands/runtime_cmds.rs`
- `ui/src/components/SandboxBanner.tsx`
- `ui/src/components/SandboxBanner.test.tsx`

**Subject 2 — sandbox replacer and flag guard.** Approved at `649b449`
(`ev-WYOvqyMasQq4xRlqW5Cmg`), whose chain is `e238a04` → `a48a694` → `649b449`. Files:

- `scripts/replace-sandbox.sh` (approved at `e238a04`, byte-identical at `649b449` — verified)
- `scripts/prepare-sandbox-build.sh`
- `scripts/test-replace-sandbox-failclosed.sh`

**The deletion guard** is a third subject and is **NOT in this candidate**. Its verdicts
name `6e04dae` / `6f1c566` on `packet/nimbus/project-scoped-reads`; none names `ac3d1f3`,
the isolated branch. No verdict has been reused to cover it.

Each subject keeps its own verdict reference. No single verdict is treated as covering the
combined candidate, and the three separate passes are not read as certifying it.

## Blob verification

Every file was taken with `git checkout <commit> -- <file>` and then re-hashed against its
source commit. All eight code blobs match byte-for-byte:

| file | source | verified |
|---|---|---|
| `src-tauri/build.rs` | `5af0e16` | ✓ |
| `src-tauri/tests/build_identity_incremental.rs` | `5af0e16` | ✓ |
| `src-tauri/src/commands/runtime_cmds.rs` | `5af0e16` | ✓ |
| `ui/src/components/SandboxBanner.tsx` | `5af0e16` | ✓ |
| `ui/src/components/SandboxBanner.test.tsx` | `5af0e16` | ✓ |
| `scripts/replace-sandbox.sh` | `649b449` | ✓ |
| `scripts/prepare-sandbox-build.sh` | `649b449` | ✓ |
| `scripts/test-replace-sandbox-failclosed.sh` | `649b449` | ✓ |

No work-list, preview, dry-run-fixture or deletion-guard file is present. The branch
contains nothing but these two subjects and their delivery notes.

## Gates, measured on the candidate

- `cargo build --workspace` — **0 errors**.
- `cargo test --workspace` — **387 passed, 0 failed**.
- `build_identity_incremental` — **19 passed**.
- `test-replace-sandbox-failclosed.sh` — **38 checks, exit 0**.
- UI suite — **398 passed, 31 files**.
- `rustfmt --check` — 0 diffs on the files I touched. The only remaining
  `cargo fmt --check` diff is `src-tauri/src/commands/history_cmds.rs`, which is
  track-owned and untouched by this candidate.

## Operator handoff — build-only, dry-run, rollback

One reproducible sequence. Nothing is installed, replaced, killed or restarted by it.

**1. Build only** (no install, no replacement, no process touched):

```
bash scripts/prepare-sandbox-build.sh --build-only packet/nimbus/verification-candidate <a string only this candidate contains>
```

It builds, proves the commit embedded in the binary equals the worktree HEAD, and stops.
See the blocker below before running it.

**2. Preview the replacement** (default mode — nothing moves):

```
bash scripts/replace-sandbox.sh --from-branch packet/nimbus/verification-candidate
```

Measured on this candidate: it correctly **refuses**, because the existing binary at
`target/release/.../SeatLoom.app` embeds `34f8681c3856` while the candidate is at
`93ef3d36ad40`. Exit code 1, and the reason names the mismatch. That is the identity gate
working — a stale build must not be installable. After step 1 rebuilds, this should pass.

**3. Preview a rollback** (also zero-effect):

```
bash scripts/replace-sandbox.sh --rollback --list
```

Measured: exit 0, reports that no backup exists yet — meaning this script has never
replaced the sandbox, so there is nothing to roll back to.

**4. Only after your explicit approval**, and only after a fresh verdict naming this
exact commit:

```
bash scripts/replace-sandbox.sh --from-branch packet/nimbus/verification-candidate <string> --apply
```

That backs up first, then replaces, then verifies the sandbox came up, and prints the
one-command rollback.

## Blockers

1. **`prepare-sandbox-build.sh` cannot find the worktrees root when run from a seat
   worktree.** `WROOT` is computed as `$REPO/../seatloom-worktrees` with no fallback,
   which resolves to `.../seatloom-worktrees/seatloom-worktrees` inside a seat worktree.
   `replace-sandbox.sh` has the fallback; this script does not. **This is pre-existing on
   track** — line 49 of `track/infra-foundation`'s copy — and I did not introduce it and
   did not fix it here, because fixing it would add a third subject to a branch whose
   whole point is to contain exactly two. Measured: step 1 run from this worktree exits 1
   with "not a branch any worktree has checked out".
2. **The candidate is not yet built.** Step 1 has not completed, so step 2 currently
   refuses on the identity mismatch shown above. That is correct behaviour, not a defect,
   but it means the handoff is not yet clickable end-to-end.
3. **No verdict names this candidate.** Both subject verdicts name their own commits, not
   `93ef3d3` plus these blobs. Flux must verify this exact candidate before any operator
   authorisation.
4. **`/Applications/SeatLoom-Sandbox.app` still runs the stale `1cee50c` build** (pid
   20580), and the main app (pid 90805) is untouched. Neither was modified by this work.
5. **The deletion guard is absent**, deliberately, for the reason above.

## What I did not do

No track merge. No live database. No `/Applications` replacement. No kill or restart. No
reopening of the cancelled card. No reuse of any verdict to cover a commit it does not
name. Lyra was not interrupted.
