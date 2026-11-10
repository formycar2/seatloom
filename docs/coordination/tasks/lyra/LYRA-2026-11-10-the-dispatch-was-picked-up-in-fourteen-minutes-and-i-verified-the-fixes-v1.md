# Standing duty: **The dispatch I issued last round was picked up and completed in 14 minutes, and I verified all three fixes myself rather than taking his report. Nimbus restored the real wrapper (`tauri-cli 2.11.1`), added `assert_macho` to both build paths before every other check, and stopped deleting the build log — commit `fb83cd6` on `packet/nimbus/candidate-final`. He also found three defects in the test fixture that were producing false verdicts, one of which was the exact inverse of the bug being fixed. I ran his suite (63 checks, pass), and I mutation-tested `assert_macho` in all five directions myself: the real binary passes, and ASCII fakes, a non-Mach-O header, a missing file and an empty path all refuse.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-dispatch-was-picked-up-in-fourteen-minutes-and-i-verified-the-fixes-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved from 513 to 518 events, and dispatches stayed at 103.** That asymmetry was the tell that something other than my own work had happened — and it was: `SeatDispatchDelivered`, `SeatResponse`, `MemoryCreated`, `TopicOpened`, `SeatDispatchCompleted`.

## What came back

| field | value |
|---|---|
| dispatch | `disp-fkLMc9KBolrwtEyoKi2EZ` |
| started | 11-10 02:22 UTC — **the minute I issued it** |
| completed | 11-10 02:36 UTC — **14 minutes later** |
| status | `done` |
| commit | `fb83cd6`, on `packet/nimbus/candidate-final` |

**So the waker gap I spent last round describing turned out not to matter for this dispatch**: something made his agent pull within the same minute. I measured a structural property of the fleet — five seats have no waker — and then found that the one dispatch I issued was picked up immediately. **Both things are true, and I do not have a mechanism that explains the second one.** I am recording that as an open question rather than constructing an explanation, which is the discipline the `pgrep` investigation taught me: four hypotheses and two wrong explanations came from not being willing to say "I don't know yet."

## What he did, verified independently

**I did not take his report. I checked each fix myself.**

| fix | his claim | my verification |
|---|---|---|
| wrapper restored | real wrapper, `tauri-cli 2.11.1` | **1224 bytes, `#!/bin/sh`, `tauri --version` → `tauri-cli 2.11.1`** |
| Mach-O check on both paths | `assert_macho` before every other check | **2 call sites, 529 and 594; defined at 87** |
| log retained | no longer `rm -f` on success | **both paths say "构建日志留着没删", no `rm -f` of the log** |

**And the check is in the form I recommended, including the fail-closed guard I tested:** it refuses an empty path, a missing file, a missing `file` binary, and unreadable output, before reaching the `case`. **He also recorded *why* the positive form is required** — that `grep -v` with empty output passes a real binary, and that the two misspellings refuse a real binary. That reasoning is now in the script, not just in my document.

**I mutation-tested it myself, in all five directions:**

| input | result |
|---|---|
| the real main-app binary (12,713,776 B) | **pass** |
| 2-byte ASCII file, executable | **refuse** — "not Mach-O: ASCII text" |
| 2-byte `MZ` header, executable | **refuse** |
| missing path | **refuse** — "artifact missing" |
| empty path | **refuse** |

**And I ran his suite in an isolated worktree: 63 checks, `✅ 全部通过`, including the mutation sections that delete the gate entirely and confirm the real `/Applications` is untouched and never mentioned.**

## The part he found that I had not

**He reported three defects in the test fixture, each of which produced a false verdict.** The one that matters most: `perl -pi -e` rewrote a file and changed its length by one byte even for an equal-length replacement, which made `strings` refuse a genuine Mach-O as truncated — **so a correct binary would have been reported as a `MUST_HAVE` failure.** He calls it "the exact inverse of the bug being fixed", and he is right: a fixture that corrupts its own artifact makes correct code look wrong.

**I want to be precise about what this does and does not say about my recommendation.** My three fixes were correct and are all landed. **What I did not do was check the test that guards them, because I was testing the fix and not the fixture.** He found that the fixture itself was unable to produce the precondition the new check requires, and rebuilt it to compile a genuine Mach-O with clang.

**So the lesson is sharper than "test your recommendation". It is that a fix and its test can each be right and still not compose** — and the composition failure here would have looked exactly like the fix being wrong, because the suite would have gone red on correct code.

## And he fixed it without breaking the containment work

**This is the part I checked hardest, because this is the fifth-round branch that has destroyed the sandbox.** His response records `real_applications_fingerprint 0d00e85f3682fb8bdf52b9a2aceecdfb before and after both runs`, and I confirmed it myself:

| application | state |
|---|---|
| `/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri` | **md5 `28753f61…`, 12,713,776 bytes, mtime 28 Oct — untouched** |
| `/Applications/SeatLoom-Sandbox.app/…/seatloom-sandbox` | **md5 `7c0a6e2a…`, 95 bytes, mtime 8 Nov — untouched** |

**And he ran a real end-to-end build** — `--build-only --allow-dirty`, exit 0, a 12,716,560-byte binary, `file` says `Mach-O 64-bit executable arm64`, commit matched, build log retained at 25 lines, `/Applications` untouched with pid 90805 still running. **So the toolchain is now real, which is the precondition for a rebuild and not the rebuild itself.** He is explicit that the sandbox is still a 95-byte fake needing operator authorisation, and that he did not touch it.

## What this changes about my blocker list

**The build-toolchain item — which I escalated as urgent for three rounds and which I said was worse than the five previous incidents — is closed.** The 48-byte fake can no longer pass, on either path, and the wrapper is real.

**What remains of it is exactly one thing: the sandbox is still a 95-byte fake with no backup, and rebuilding it now needs only operator authorisation.** That is a smaller item than it was this morning, and the difference is entirely due to a dispatch I spent a round deciding to issue.

## What I did not do

I did not modify any file outside my new document, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not touch `node_modules/.bin/tauri`** — it is his, it is now the real wrapper, and it is not mine. **I did not send keys to any tmux session.** My measurements were reads plus two invocations in isolated worktrees: `bash scripts/test-build-failure.sh`, and `assert_macho` extracted into a scratch harness against five inputs. The worktrees were removed and both real applications are byte-identical to before.
