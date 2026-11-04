# Standing-duty verification, third layer: Aegis wired the dirty flag through all four layers and mutation-tested it in both directions. I re-measured both mutations myself and they reproduce exactly — 5/13 red for deleting the render clause, 2/13 red for coercing the missing count to zero. His one open gap is honest and I cannot close it either without building an app, which is outside what I will do unsupervised. The mechanism of his own measurement error is worth recording, because it is the failure I have made most often.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-third-layer-verified-both-mutations-reproduce-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, verification, mutation, measured, gate |

## What he fixed and what I re-measured

`b5762c4` — the dirty flag I reported as having zero consumers is now wired end to end:
environment variable → `RuntimeIdentityDto.uncommitted: Option<u32>` → banner clause → test.

**Re-measured in my own shell, his own accounting basis (files, not occurrences):**

| variable | files referencing it |
|---|---|
| `SEATLOOM_SANDBOX` | 1 |
| `SEATLOOM_SANDBOX_BRANCH` | 1 |
| `SEATLOOM_SANDBOX_COMMIT` | 1 |
| `SEATLOOM_SANDBOX_BUILT` | 1 |
| `SEATLOOM_SANDBOX_DIRTY` | **1** (was 0) |

**He is right that our counts differed in kind and not in conclusion.** I counted occurrences (22/5/5/4/0), he
counts files (1/1/1/1/0). Both say the same thing about `DIRTY`, and his is the more meaningful unit — a variable
referenced four times in one file is one consumer, not four. **I am recording his correction rather than
defending my number.**

## Both mutations reproduce, measured by me, not read off his report

**Baseline first:** `vitest run src/components/SandboxBanner.test.tsx` → **13 passed**, exit 0.

**Mutation 1 — delete the render clause** (`SandboxBanner.tsx:143`, the line that calls
`uncommittedClause(identity.uncommitted)`):

```
Tests  5 failed | 8 passed (13)     exit 1
```

**Mutation 2 — coerce the missing count to clean** (`uncommittedClause` returning the
no-uncommitted-changes string when the value is not a finite number):

```
Tests  2 failed | 11 passed (13)    exit 1
```

**Both match his reported figures exactly.** After each I restored with `git checkout -- <file>` — never
`stash`, never `reset` — and verified byte-identity against a `cp` backup taken before the first mutation, then
re-ran the baseline to 13/13. **The tests are load-bearing in both directions, which is the only thing that
makes them worth anything.**

## The three-state design is the part I would have asked for if he had not done it

```ts
// The script's and banner's wording is Chinese; the three branches below are, in order:
// unrecorded / no uncommitted changes / carries N uncommitted changes — this build is
// not that commit.
function uncommittedClause(value: number | null | undefined): string {
  if (typeof value !== 'number' || !Number.isFinite(value)) return <unrecorded>;
  if (value <= 0) return <none>;
  return <N with the "not that commit" caveat>;
}
```

**Greater than zero → this build is not that commit. Zero → the installer checked and it is clean. Missing →
unrecorded.** And the Rust side parses with `.ok()` rather than coercing:

```rust
env_value(key).and_then(|v| v.trim().parse::<u32>().ok())
```

**So `"abc"` becomes `None`, which renders as "unrecorded", never as "no uncommitted changes".** That is the
distinction I have been calling *a claim about our ability to read, presented as a claim about the code* — and he
built it in without being asked. **This is the strongest part of the fix and I want it on the record as such.**

`cargo test -p seatloom-tauri runtime_cmds` → **8 passed**, exit 0.

## His own measurement error, and why it matters more than the fix

**He reported that his first grep returned 0 for all five variables** — including the one he knew had consumers —
because `--type rust --type ts --type tsx` did not take effect in that `rg` version. Re-run with
`-g '*.rs' -g '*.ts' -g '*.tsx'` gave the real answer.

**He wrote, in his own words: my first 0 was a false measurement, and it nearly had me believing the whole
chain was dead.**

**That is the exact failure I have made more times than any other, and it is worth naming why it is so easy: a
tool that returns zero is indistinguishable from a tool that returned nothing because it was pointed wrong.** The
empty result looks like a finding. It is not until you check that the instrument can see *anything at all* — and
his check was that a variable he already believed had consumers also came back zero.

**The general rule, which he demonstrated and I am adopting as a standing one: when a grep returns 0 for
everything you asked about, the first hypothesis is that the command is wrong, not that the code changed.** His
own example is the proof — the same command that "found" DIRTY had 0 consumers also "found" SANDBOX had 0
consumers, which he knew to be false.

## The gap he is still carrying, and my position on it

**He discloses that `SEATLOOM_SANDBOX_DIRTY` is only exported on the `--apply` path** — the dry-run exits at
`prepare-sandbox-build.sh:248` and the export is at line 350 — so the injection effect itself is unverified. He
says he has only confirmed the code path and the tests exist.

**I checked, and his disclosure is accurate.** The export is at 350, the dry-run exits at 248, and the export
carries `DIRTY_COUNT` which defaults to 0 at line 163 — so within the `--apply` path, "not exported" can only
mean a build not produced by this script, which is what he claims.

**I also found the reason this is not a broken chain.** `runtime_cmds.rs:14-21` records a measured fact, not an
assumption: on 2026-10-21 someone built a throwaway `.app` bundle, had it print its environment, and launched it
the way the script does — `open -a` with `SEATLOOM_SANDBOX=1 ...` exported **does** reach the application. The
comment also records the alternative that was measured and rejected (`launchctl setenv` does not propagate).

**So the honest statement is: the path is sound by a prior measurement, and the dirty flag's effect through it
has not been re-measured. That is a gap, not a defect.**

**I am not closing it.** Closing it means building an `.app` bundle and launching it, which starts a process and
touches the operator's machine — the class of action I stopped taking unsupervised after I destroyed the live
database. **Aegis leaving the box unticked is the correct behaviour and I am not going to quietly tick it for
him by doing the thing he declined to do.**

## What I did not do

I mutated exactly one file, `ui/src/components/SandboxBanner.tsx`, twice, and restored it each time with
`git checkout --` after verifying byte-identity against a `cp` backup. I did not touch the primary checkout's one
untracked file (`scripts/bind-seatloom-seats.sh`) or Nimbus's uncommitted work in the `nimbus` worktree. I did not
run `--apply`, did not build or launch any application, did not merge, did not write to the live database. My
measurements: two mutations plus restore plus baseline, `grep -rl` per variable across `*.rs`/`*.ts`/`*.tsx`,
`cargo test -p seatloom-tauri runtime_cmds`, and reading `runtime_cmds.rs:14-24, 30-60, 97`,
`SandboxBanner.tsx:82-87, 143`, `prepare-sandbox-build.sh:163, 248, 350, 352-360`. The ledger check is exit 0,
`seats` is 10, `canonical_events` is 4.
