# Standing duty: **I tested the two-step rebuild path I offered the operator last round, and the first step is a trap. `--build-only` reported success and produced a binary — but that binary is 48 bytes of ASCII containing the commit string the verifier looks for, because `$WT/node_modules/.bin/tauri` is a 330-byte stub that fabricates exactly that and exits 0. The real Tauri CLI is installed one directory away. So every check in the build-verification chain passes on a fabricated artifact, and my "authorise a build first" recommendation would have had him authorise a step that cannot fail.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-two-step-path-i-offered-fails-at-step-one-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending.

## What I did, and why

**Last round I verified the sandbox gate fires, and I offered the operator a two-step path: authorise `--build-only` first, confirm it produces a real binary, then authorise the install. I presented that as two small decisions instead of one large one.**

**A recommendation I have not run is a claim. So I ran step one.**

## What step one actually did

```
✅ 构建产物:.../target/release/bundle/macos/SeatLoom.app
✅ 提交:26a5d9442be6
一步都没装、一步都没替换、一个进程都没停、一个进程都没起。
```

**Every line true. And the artifact:**

| check | measurement |
|---|---|
| `target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri` | **48 bytes** |
| `file` | **ASCII text** |
| content | `seatloom-build-commit=26a5d9442be6` + `MUST_HAVE_ME` |

**A 48-byte text file. The same disease as the 95-byte fake sandbox, one layer down, inside the verification path.**

## The mechanism

`prepare-sandbox-build.sh:476` builds with `PATH="$WT/node_modules/.bin:$PATH" tauri build`. **There is no `tauri` on PATH at all** — the script's entire build capability is that prepended directory. In `nimbus-psr`:

```
node_modules/.bin/tauri           330 bytes  7 Nov 15:02   ← the stub
node_modules/@tauri-apps/cli/tauri.js                  ← the real CLI, installed
```

**The stub, verbatim:**

```bash
#!/usr/bin/env bash
mkdir -p target/release/bundle/macos/SeatLoom.app/Contents/MacOS
printf "seatloom-build-commit=%s\nMUST_HAVE_ME\n" "$(git rev-parse --short=12 HEAD)" \
  > target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri
chmod +x target/release/bundle/macos/SeatLoom.app/Contents/MacOS/seatloom-tauri
exit 0
```

**It fabricates the exact three things the verifier checks: an executable at the expected path, a `MUST_HAVE` string, and a `seatloom-build-commit=` matching the worktree HEAD.** And because the commit string is computed from the live HEAD, it matches — the script reports "二进制内提交:26a5d9442be6 / 工作区 HEAD:26a5d9442be6 / ✅ 一致".

## What is missing

**`prepare-sandbox-build.sh` has zero Mach-O / `file` / `otool` checks** — `grep -cE "Mach-O|file -|otool|lipo"` returns **0**. So the script checks that a binary *exists*, is *executable*, *contains a string*, and *reports the right commit*. **A text file satisfies all four.**

**And the stub is invisible to every git-level instrument**: it is untracked and matched by `.gitignore:4 node_modules/`. So it survives branch switches, is absent from `git status`, and cannot be caught by the dirty-worktree guards or the pre-commit hook. **It is a permanent shadow on the build in that worktree.**

## The family, audited

| worktree | `.bin/tauri` | |
|---|---|---|
| `seatloom` | 1122 B, 05-08 | (empty — no npm wrapper) |
| `d2preview` | 1122 B, 05-08 | genuine `#!/bin/sh` npm wrapper |
| `mira` | 1212 B, 09-30 | genuine npm wrapper |
| `nimbus` | 1224 B, 10-17 | genuine npm wrapper |
| **`nimbus-psr`** | **330 B, 11-07 15:02** | **the fabricating stub** |

**So it is one worktree, and its date is the day of the destruction incidents.** I am not claiming I know who wrote it or why — I have not established that, and unlike `pgrep` I am not going to spend seven rounds. **What I can state is that a build in that worktree cannot produce a real binary, and that the script has no check that would notice.**

## Why this matters more than the sandbox itself

**The sandbox being a 95-byte fake is visible: `file` says ASCII text.** **This one is not, because it sits behind four checks that all report success and a script that ends with "✅ 一步都没装".** If the operator had followed my recommendation, he would have seen a green build, concluded the toolchain works, and authorised the install — **and installed a 48-byte text file into `/Applications/SeatLoom-Sandbox.app`, destroying the last real artifact, with a full green verification trail as the evidence that it was fine.**

**That is worse than the five previous incidents, because each of those left a visible fake and a visible question. This one would have left a visible fake and a green check.**

**And the general form: a verification chain that checks an artifact's self-report is satisfied by anything that fabricates that self-report.** The commit string is *inside* the artifact. Checking it proves the artifact can print a commit, not that it was built from one.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, or write to the live database. **I did not delete, repair, or overwrite `node_modules/.bin/tauri`** — it is in another seat's worktree, it is untracked and gitignored so its loss is unrecoverable, and repairing the toolchain is not my call. My measurements were reads plus one gated invocation in an isolated worktree: `bash scripts/prepare-sandbox-build.sh … --build-only`, `ls`, `file`, `cat`, `md5`, `git worktree` add/remove, and `grep` over the script and the other worktrees' wrappers. The real sandbox and main app are byte-identical to before.
