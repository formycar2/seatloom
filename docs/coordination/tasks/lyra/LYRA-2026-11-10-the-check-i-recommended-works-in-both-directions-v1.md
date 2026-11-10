# Standing duty: **I tested the Mach-O check I recommended last round, and it works, and it is not in the script — in either path. `file -b` reports `Mach-O 64-bit executable arm64` for the real binary and `ASCII text` for both fakes, and the check is fail-closed in all three directions: missing `file`, missing binary, and inverted logic all refuse rather than pass. But `prepare-sandbox-build.sh` has no type check at all, and no script in `scripts/` does either — the single `Mach-O` match in the whole tree is a comment. So the recommendation stands as a real gap, and it is a one-line addition.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-check-i-recommended-works-in-both-directions-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending.

## The recommendation I had not tested

**Last round I recommended, as step 2 of three, adding a Mach-O type check to the build step of `prepare-sandbox-build.sh`.** I described it as "the thing that makes the step able to fail."

**That is a recommended fix, and my own recorded lesson is that a recommended fix is a claim.** So I tested it, including the plausible mistypings, which is the other lesson.

## The check works, both directions

| artifact | `file -b` | verdict under the check |
|---|---|---|
| real main-app binary | `Mach-O 64-bit executable arm64` | **pass** |
| 48-byte fake from `--build-only` | `ASCII text` | **refuse** |
| 95-byte fake sandbox | `ASCII text` | **refuse** |

**The check, in the form I would put in the script:**

```sh
case "$(/usr/bin/file -b "$BUILT/Contents/MacOS/seatloom-tauri" 2>/dev/null)" in
  *Mach-O*) ;;
  *) fail "构建产物不是 Mach-O 可执行文件 —— 这是一份假构建,不要装" ;;
esac
```

## The mistypings, run

**`grep -q "Mach-O"`** — the obvious form, and it works:

| pattern | real | fake |
|---|---|---|
| `Mach-O` | pass | refuse |
| `mach-o` | **refuse** | refuse |
| `macho` | **refuse** | refuse |
| `Mach` | pass | refuse |
| `executable` | pass | refuse |

**So the pattern must be spelled exactly `Mach-O`.** `mach-o` and `macho` both reject the real binary — which is fail-closed, so a mistyped pattern blocks every build rather than admitting a fake. **That is the correct failure direction, and it is worth stating: a typo here produces a build that cannot run, not a fake that can.**

**The inverted form `grep -v "Mach-O"` is the dangerous mistyping**, because with no output at all — `file` missing, path wrong — `grep -v` matches and the check passes. **I verified this: under `env -i` with `file` unfindable, the positive check refuses and the inverted check would not.** So the check must be positive (`case … *Mach-O*)` or `grep -q`), never negated.

**And I nearly recorded a false alarm here myself**: my first `env -i` test reported "通过", which would have been a hole. It was not — `sh` supplies a default PATH, so `/usr/bin/file` resolved fine. **I had to construct a genuinely empty PATH before the fail-closed behaviour appeared.** That is the second time this week that a test harness, not the subject, produced the surprising result.

## The gap, confirmed on both paths

| path | checks after the build |
|---|---|
| `--apply` (line 266) | executable at the path, then `MUST_HAVE` string present, then commit match — **no type check** |
| `--build-only` (line 476) | executable at the path, then `MUST_HAVE` string, then commit match — **no type check** |

**And across the whole tree:**

```
$ grep -rlE "Mach-O" --include="*.sh" scripts/ | wc -l
1        # and that one match is a comment in clear-stale-node-options.sh
```

**So there is no existing type check anywhere to reuse, and no script that would have caught the 48-byte or 95-byte fakes.** The check is genuinely additive.

## What this changes about my three-step recommendation

**Last round I gave three steps in order: restore the wrapper, add the type check, keep the build log.** **Testing the type check confirms step 2 is correct, cheap, and safe — a `case` statement that refuses on anything that is not Mach-O, that fails closed on every mistyping I could construct except the inverted form, and that I have verified against a real binary and both fakes.**

**But it also sharpens the ordering.** Step 1 (restore the wrapper) is in Nimbus's worktree and step 3 (keep the log) is in his branch — **neither is mine.** Step 2 is a change to `scripts/prepare-sandbox-build.sh`, which lives on his branch too. **So all three steps are on another seat's branch, and my role is to have tested them and to hand them over with evidence, not to apply them.**

**And there is a reason to prefer the type check over the wrapper restoration as the first thing done**: the wrapper restoration fixes one cause, and the type check fixes the class. **A worktree can acquire a new stub, a stale `target/release`, or a partial copy; the type check refuses all of them.** So if only one thing can be done, the type check is the one that keeps working.

## What I did not do

I did not modify any file in any seat's worktree, dispatch anyone, merge, accept or cancel a card, install, restart any application, or write to the live database. **I did not edit `scripts/prepare-sandbox-build.sh`** — it is on `packet/nimbus/candidate-final`, and editing another seat's build script without a dispatch is exactly the boundary I have held all week. My measurements were reads plus invocations of `file` and `/bin/sh` on artifacts I created in `/tmp`: the real binary, a 48-byte fake I synthesised, the 95-byte fake sandbox, and four spellings of the pattern. Both real applications are byte-identical to before.
