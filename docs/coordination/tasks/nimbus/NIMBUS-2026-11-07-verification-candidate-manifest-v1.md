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

**The deletion guard** is a third subject and is **NOT in this candidate**.

The earlier version of this manifest said "no isolated guard verdict exists". That
statement was **too broad and is corrected here**: the ledger does carry
`ev-flux-verdict-deletion-guard-only-05`, a pass naming `34f8681` on
`packet/nimbus/deletion-guard-only`, covering the guard itself with a 5/5 guard
suite and three of Flux's own mutations red. So a verdict for the isolated guard
branch does exist.

What does not exist is a verdict for the guard **as part of this candidate**, and the
guard is still excluded, for two measured reasons:

1. That verdict's own `not_claimed` records that at `34f8681` the two identity defects
   were still open — the watch paths that do not exist in a linked worktree, and a
   dirty build indistinguishable from a clean one. Both are fixed later in this
   candidate's history, but not at `34f8681`.
2. The guard's blobs would come from `34f8681`, which is an ancestor of this branch's
   history, so including them would mean reusing a verdict to cover a commit it does
   not name — the thing this manifest exists to avoid.

So the exclusion stands, but the reason is now the correct one. What the candidate
**cannot** demonstrate is the guard at runtime, and that is stated rather than glossed.

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

## Operator handoff — measured, end to end

All three steps below were **run**, not described. The final commit for the handoff is
`9f0e50f`.

**Step 1, build only** — from this seat worktree, on the committed clean source:

```
bash scripts/prepare-sandbox-build.sh --build-only packet/nimbus/verification-candidate seatloom-build-commit
```

Result: **exit 0**. Frontend built, release built, and the binary's own embedded commit
`9f0e50f2e4e5` matched the worktree HEAD. Its own closing line states that nothing was
installed, replaced, stopped or started.

**Step 2, preview the replacement**:

```
bash scripts/replace-sandbox.sh --from-branch packet/nimbus/verification-candidate
```

Result: **exit 0**, listing the backup path, the seven actions in order, the processes and
paths involved, and what needs approval. It no longer refuses on identity, because step 1
produced a matching binary. It still stops before `--apply`.

**Step 3, preview a rollback**:

```
bash scripts/replace-sandbox.sh --rollback --list
```

Result: **exit 0**, reporting that no backup exists — meaning this script has never
replaced the sandbox, so there is nothing to roll back to yet.

Zero side effects were verified after all three: the main application (pid 90805) is
still running and untouched, no backup directory was created, and the sandbox binary's
mtime is unchanged from before the run.

## Blockers

1. **The build prerequisites must be installed first.** `pnpm install` at the repository
   root and in `ui/`. Both are gitignored, so neither can dirty the source — verified with
   `git status --porcelain` clean afterwards. Without them step 1 fails at
   "no node_modules", which is what its own message says.
2. **No verdict names this candidate.** Both subject verdicts name their own commits. Flux
   must verify the exact final commit before any operator authorisation, and the three
   separate passes are **not** a verdict on the combination.
3. **`/Applications/SeatLoom-Sandbox.app` is still the stale build.** Its executable's
   mtime is unchanged and it was not modified. The main application (pid 90805) was never
   touched.
4. **The deletion guard is absent**, deliberately. A verdict for the isolated branch does
   exist (`ev-flux-verdict-deletion-guard-only-05` at `34f8681`, pass, guard suite 5/5), but
   it predates the identity fixes and does not cover this candidate. The candidate
   therefore cannot demonstrate the guard at runtime, and this manifest does not claim it
   can.

## Defects found while making the handoff runnable

These were not in the dispatch's scope; they surfaced because the handoff was run rather
than assumed. All are fixed on this branch.

**The worktree root was guessed.** Both scripts computed `$REPO/../seatloom-worktrees`,
which resolves to a doubled path inside a seat worktree — so the loop over worktrees never
ran and the error blamed the branch. `replace-sandbox.sh` had a `[ -d ]` fallback and
`prepare-sandbox-build.sh` did not, which is how the same bug came back in one place after
being fixed in the other. Both now share `scripts/worktree-root.sh`, which asks
`git worktree list`.

**A failed build was reported as a successful one.** Measured: `pnpm build` failed with
`tsc: command not found`, and the script printed "frontend artefacts present" and
"application bundle present" anyway. Two defects compounding — the pipe
`pnpm build 2>&1 | tail -4` made `$?` the status of `tail`, and `[ -d ui/dist ]` was
satisfied by the **previous day's** `dist`. `tauri build` then bundled the day-old frontend
into the application, and only the binary-commit check caught it. Both build sites now
capture the exit status separately and delete the stale artefact first, so "dist exists"
can only mean "this build produced it".

That fail-open was hiding real errors in my own code: the frontend had never compiled this
branch's test file. Two TypeScript errors were fixed once they became visible.

This is the **third instance of the same defect family today**: a pipe or `|| true`
swallowing a command's exit status while a stale state satisfies the check meant to catch
it.

## What I did not do

No track merge. No live database. No `/Applications` replacement. No kill or restart. No
reopening of the cancelled card. No reuse of any verdict to cover a commit it does not
name. Lyra was not interrupted.
