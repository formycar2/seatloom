# Standing-duty verification and a gap one layer below the one Aegis disclosed: his dirty-worktree guard is correct in all four directions and the disclosure about the `--apply`-only injection is accurate. But the variable he exports has no consumer at all — `RuntimeIdentityDto` has four fields and no `dirty` one — so the promise "the sandbox identity will state that it carries uncommitted changes" is fulfilled only in the terminal banner, not in the application the operator is looking at. That is the exact surface the whole mechanism exists to protect.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-the-dirty-flag-is-exported-and-nothing-reads-it-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, verification, measured, live, gate |

## What I verified, and it holds

Aegis fixed the hole I found (`LYRA-2026-11-04-aegis-fix-verified-and-the-uncommitted-changes-hole-v1.md`) in
`301d7f2`. I re-measured all four directions he claims, all dry-run:

| input | his claim | my measurement |
|---|---|---|
| `packet/nimbus/session-work-readmodel` (dirty) | exit 1, lists the files | **exit 1**, lists both files by path ✓ |
| `packet/mira/knowledge-panel` (clean) | exit 0 | **exit 0** ✓ |
| `9f29b060` (commit hash) | exit 1 (previous guard still live) | **exit 1** ✓ |
| dirty + `--allow-dirty` | exit 0, marked | **exit 0**, banner reads a `+2 uncommitted changes` marker (the script's wording is Chinese; rendered in English here) ✓ |

**The refusal message names the mechanism and the consequence, which is the part that makes it a guard rather
than a warning:** *this build would contain changes that exist in no commit, while the banner would print only a
commit — you would verify in the sandbox a thing labelled as one commit that is really someone's half-finished
work.* It lists the files, gives the escape hatch, and states that the sandbox identity will record the
uncommitted changes.

**He also recorded a procedural error of his own** — a `| head -12` pipe fed his own script SIGPIPE and gave
141, which he caught and re-ran without the pipe. **That is the discipline I have been holding him to, applied
to himself, and it is worth saying so.**

## The gap, one layer below the one he disclosed

He disclosed honestly: the `SEATLOOM_SANDBOX_DIRTY` injection sits on the `--apply` path only, the dry-run
exits at line 248 and the export is at line 349, so he verified the code path exists and not the effect.

**I checked that, and it is accurate.** Then I asked the question he did not: **does anything read the
variable?**

```
$ grep -rn "SEATLOOM_SANDBOX_DIRTY" --include=*.rs --include=*.ts --include=*.tsx .
(only the export in scripts/prepare-sandbox-build.sh)
```

**Nothing reads it. Zero consumers.** For contrast, the three sibling variables do have consumers:

| variable | consumers |
|---|---|
| `SEATLOOM_SANDBOX` | 22 |
| `SEATLOOM_SANDBOX_BRANCH` | 5 |
| `SEATLOOM_SANDBOX_COMMIT` | 5 |
| `SEATLOOM_SANDBOX_BUILT` | 4 |
| `SEATLOOM_SANDBOX_DIRTY` | **0** |

The consumer for the siblings is `src-tauri/src/commands/runtime_cmds.rs`, which builds
`RuntimeIdentityDto`:

```rust
pub struct RuntimeIdentityDto {
    pub is_sandbox: bool,
    pub branch: Option<String>,     // SEATLOOM_SANDBOX_BRANCH
    pub commit: Option<String>,     // SEATLOOM_SANDBOX_COMMIT
    pub built_at: Option<String>,   // SEATLOOM_SANDBOX_BUILT
}
```

**Four fields. No `dirty`.** The renderer is `ui/src/components/SandboxBanner.tsx:111-112`:

```
sandbox · branch <branch> · commit <commit> · built at <time>
```

**So the promise "the sandbox identity will state that it carries uncommitted changes" is currently kept only
in the terminal banner the operator does not see while clicking through the app.** In the application itself, a
sandbox built with `--allow-dirty` is indistinguishable from a clean one.

**And that is precisely the surface this mechanism exists to protect.** The failure the guard prevents is: the
operator opens a sandbox, believes it is commit X, and verifies acceptance against half-finished work. The
terminal banner protects the person who ran the command. The in-app banner protects the person doing the
acceptance. **Today only the first is protected.**

## What completing it would take, measured so the ask is not a guess

Four places, all measured today:

| layer | file | what is missing |
|---|---|---|
| Rust DTO + read | `src-tauri/src/commands/runtime_cmds.rs` | a `dirty_count`/`uncommitted` field + `env_value("SEATLOOM_SANDBOX_DIRTY")` |
| TS type | `ui/src/components/SandboxBanner.tsx:32-35` | the same field on `RuntimeIdentity` |
| render | `ui/src/components/SandboxBanner.tsx:111-112` | a clause that states it, not a silent absence |
| test | `ui/src/components/SandboxBanner.test.tsx:32-35` | the fixture carries four fields matching the DTO — a fifth must be asserted, or the test keeps passing while the field is dropped again |

**The test fixture is the load-bearing one.** It currently mirrors the DTO exactly, which is why nothing failed:
there is no assertion that the identity surface exposes every fact the installer knows. **That is the same
instrument family as before — a check that cannot fail, this time because its fixture was updated to match the
code instead of the requirement.**

## What this does not do to his fix

**It does not weaken the guard.** The default-deny is real and correct: a dirty worktree now refuses unless
`--allow-dirty` is passed explicitly, and the escape hatch prints the count in the run output. **Aegis's
disclosure was honest and his fix is load-bearing in all four directions I measured.**

**What is incomplete is the last step of his own stated intent** — that a `--allow-dirty` build must not be able
to pass itself off as a commit. In the terminal it cannot. In the app, it still can.

## Why I am recording this rather than dispatching it

**I am not patching another seat's script or product code, and this is not yet a dispatch.** The reason is that
the cold-start acceptance this protects is not currently runnable at all: the installed sandbox binary is from
10-21, the installed main app from 10-28, and neither contains the restore mechanism. **The honest statement is
that this gap is a prerequisite for a future `--allow-dirty` acceptance run, not a blocker for the one being
prepared — and I am labelling it as such rather than dressing it as urgent.**

**One thing I will say plainly: the pattern is now three-for-three.** The commit-label fix, the dirty guard, and
now the dirty flag each closed the layer they were aimed at and left the next one. **Each time the aim was the
terminal output, because that is what a shell script can see. The layer that matters is the screen the operator
reads while deciding whether to click Accept.**

## What I did not do

Every command was a dry-run, which by the script's own contract builds nothing, installs nothing and starts
nothing. I did not run `--apply`. I did not modify any worktree — `git status --porcelain` and `git diff` are
reads, and I did not touch Nimbus's uncommitted work. I did not edit `runtime_cmds.rs`, `SandboxBanner.tsx` or
its test. I did not merge, restart the operator's app, or write to the live ledger. My measurements: four
dry-runs, `grep -rn` for each `SEATLOOM_SANDBOX*` variable across `*.rs`/`*.ts`/`*.tsx`, reading
`runtime_cmds.rs:30-80`, `SandboxBanner.tsx:32-35,111-112`, `SandboxBanner.test.tsx:32-39`, and
`prepare-sandbox-build.sh:155-182, 244-248, 335-358`. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 4.
