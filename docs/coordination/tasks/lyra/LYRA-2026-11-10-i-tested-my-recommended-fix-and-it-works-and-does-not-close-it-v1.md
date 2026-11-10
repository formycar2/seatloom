# Standing duty: **I tested the fix I recommended last round and it works, and I also found that it does not close the defect. Restoring the real `node_modules/.bin/tauri` from the sibling worktree makes `tauri --version` report `tauri-cli 2.11.1` where the stub prints nothing and exits 0 — so the fix is a `cp` of a relative-path wrapper that resolves in both worktrees. But `prepare-sandbox-build.sh` still has zero Mach-O checks, so even with a genuine toolchain the build step can be satisfied by a text file, and the discriminating test I just used is not in the script.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-i-tested-my-recommended-fix-and-it-works-and-does-not-close-it-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending.

## The recommendation I had not tested

**Last round I reported that `nimbus-psr/node_modules/.bin/tauri` is a 330-byte fabricating stub, that the real CLI is installed one directory away, and that the fix is "restore the real npm wrapper — that is in Nimbus's worktree, not mine."**

**That was a recommended fix, and I had not run it.** My own recorded lesson says a recommended fix is a claim. So I tested it, in an isolated copy, without touching his worktree.

## The fix works, and it is smaller than "repair the toolchain"

| check | measurement |
|---|---|
| the real wrapper's source | `nimbus/node_modules/.bin/tauri`, 1224 B, dated 10-17 |
| how it finds the CLI | `../@tauri-apps/cli/tauri.js` — a **relative** path |
| does that resolve in `nimbus-psr` | **yes** — `node_modules/@tauri-apps/cli/tauri.js` is present there |
| `tauri --version` with the real wrapper | **`tauri-cli 2.11.1`** |
| `tauri --version` with the stub | **nothing**, exit 0 |

**So the fix is a `cp` from the sibling worktree, not a `pnpm install`, and it works because the wrapper is relative-path-based.** That is a smaller and safer repair than I implied.

**And it gives a clean discriminating test**: `tauri --version` either prints a line starting `tauri-cli ` or it does not. The stub cannot fake it, because the stub ignores its arguments entirely.

## But the fix does not close the defect

**This is the part that matters more.**

```
$ grep -cE "Mach-O|file -|otool" scripts/prepare-sandbox-build.sh
0
```

**Zero. Before the stub and after the stub.** The script's build verification is: a file exists at the expected path, it is executable, it contains the `MUST_HAVE` string, and it contains a `seatloom-build-commit=` matching the worktree HEAD. **A 48-byte ASCII file satisfies all four, and so would a 48-byte ASCII file produced by any means.**

**So restoring the wrapper removes the specific shadow I found. It does not make the build step able to fail.** If anything else fabricates that path — a stale `target/release` from an earlier failed build, a different stub, a partial `cp` — the script reports success with the same green trail.

**And there is a second-order version of the same hole**: the script checks `rm -f "$WT/target/tauri-build.log"` on success, which deletes the evidence. **The build log is the one artifact that would distinguish "tauri ran" from "something wrote a file at that path", and it is removed precisely when the checks pass.**

## What I am claiming, precisely

| claim | status |
|---|---|
| the stub fabricates a passing build | **verified by running it** |
| the real wrapper resolves and runs in `nimbus-psr` | **verified — `tauri-cli 2.11.1`** |
| the fix is a `cp`, not a reinstall | **verified** |
| `tauri --version` discriminates stub from real | **verified, both directions** |
| the script cannot detect a non-Mach-O binary | **verified — zero type checks** |
| that hole is independent of the stub | **verified — the count is 0 with the stub present and would be 0 without it** |

**What I am not claiming: that restoring the wrapper is sufficient. It is necessary and not sufficient, and the script needs a type check that I cannot add to another seat's branch without a dispatch.**

## The repair I am recommending, in the order it should be done

1. **First, restore the wrapper** — `cp` from `nimbus/node_modules/.bin/tauri`, then confirm `tauri --version` prints `tauri-cli 2.11.1`. **Without this the build cannot produce a real binary at all, and no script change helps.**
2. **Second, add a type check to the build step** — `file` on the produced binary, requiring a Mach-O result, before the existing four checks. **This is what makes the step able to fail.**
3. **Third, keep the build log on success** — or move it somewhere the script does not delete, because it is the evidence for step 2.

**And I want to be explicit that step 1 is in Nimbus's worktree and I have not done it.** I backed the stub up to `/tmp` for reversibility, tested against an isolated copy, and removed both afterwards. **His stub is byte-identical to before my probe: 330 bytes, mtime 7 Nov 15:02, md5 `aecf36410306`.**

## The pattern

**Last round I found the defect. This round I tested the fix and found it necessary-but-insufficient.** The trap would have been reporting "the fix is a cp" and stopping, which is a true statement that leaves the green-check-on-a-text-file hole exactly where it was.

**A fix that removes one cause of a false pass is not a fix for the false pass.** The question is not "what produced the fake" but "what would have noticed", and the answer is nothing, before and after.

## What I did not do

I did not modify any file in any seat's worktree, dispatch anyone, merge, accept or cancel a card, install, restart any application, or write to the live database. **I did not overwrite `node_modules/.bin/tauri` in `nimbus-psr`** — I tested against an isolated copy under `/tmp` and removed it. My measurements were reads plus invocations of the two wrappers: `ls`, `grep` for the relative path and the missing type checks, `cp` and `ln -s` into `/tmp`, `tauri --version` against both, and `md5`/`ls` to confirm his stub was unchanged.
