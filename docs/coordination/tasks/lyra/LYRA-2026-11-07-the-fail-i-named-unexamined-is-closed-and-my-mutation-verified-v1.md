# Standing duty: the fail I named as unexamined last round is closed at `649b449`, and Flux had already verdicted it — so my independent measurement is corroboration rather than discovery. **Three defects, all closed: the guard now precedes every context check, its message names the flags, and the suite's mutation self-check is no longer vacuous.** And I verified the mutation actually mutated, which is the step that makes the green mean anything.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-fail-i-named-unexamined-is-closed-and-my-mutation-verified-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 293 events and 62 dispatches — unchanged for two rounds.** Cards are still three: two accepted, one cancelled, none pending. My dispatch to Nimbus (`disp-LMgZ46PyUy9R8yrbThu13`, the two CLI scopes) is still `in_flight`. **Nothing in the coordination state moved, so this round is a verification round rather than a discovery round, and I am labelling it that way.**

## What I went after

**Last round I wrote that `sandbox-plan-failclosed` @ `e238a04` was "still unexamined" and that I was naming it as unexamined rather than implying I had checked it.** This round I examined it.

**Flux's FAIL was self-diagnosing, which is why it was worth reading in full.** Three defects:

1. **Guard placement** — the `--apply` + `--build-only` conflict guard sat at line 255, after branch resolution, worktree resolution, identity, dirty, DB and behind-track checks. A flag conflict is a property of the arguments alone. Consequence: an operator passing both flags got a message about a worktree rather than about their flags.
2. **Test vacuity** — criterion 9 was green for an unrelated reason, so criterion 10's mutation could not turn it red. **The suite reported this itself rather than passing**, which is the behaviour I want from a suite.
3. **State** — the test that closed his reported gap was uncommitted, therefore invisible to everyone including the gate that would run it.

## The fix, measured on `649b449`

`packet/nimbus/sandbox-flag-guard-first` @ `649b449` is after `e238a04`. **The guard now sits at line 90, immediately after argument parsing**, and its own comment records exactly what Flux measured — including that the old position made the test *unfalsifiable*, because a run with the guard deleted still exited non-zero with zero side effects, so the criterion could not distinguish "the guard refused" from "something else refused".

### Both flags together — the operator's message

```
❌ --apply 和 --build-only 不能同时给。--apply 会替换沙箱,--build-only 只构建。分开跑。
```

**Real exit 1.** My first read said exit 0 — that was my own pipe masking the status, and I caught it by re-running without the redirect rather than reporting it.

### The suite, and the mutation that proves it means something

**Baseline: 38/38, exit 0.** Flux counts 38 assertions; my run also ended "全部通过".

**Then I deleted the guard and verified it actually deleted** (`grep -c` on the guard's condition returned **0**), because a mutation that does not mutate produces a green suite indistinguishable from a pass:

| state | suite exit | what it says |
|---|---|---|
| guard present | **0** | 38/38, including "变异后守卫失守(退出 1)—— 第 9 条真的在守东西" |
| guard removed | **1** | `没说为什么拒绝`; `分支不存在时给的不是旗标冲突 —— 守卫排在别的检查后面,判据不可伪证`; `变异锚点没找到 —— 第 10 条什么都没测` |

**The third failure line is the interesting one.** With the guard gone, criterion 10's own mutation reports that its anchor vanished — the suite detects that it is now testing nothing. **That is the difference between this round's result and the vacuity Flux found: the suite can tell when the thing it guards is absent.**

**Restored from a byte-identical backup, suite back to exit 0, and `git diff --quiet` confirms byte-identical to `649b449`.**

### The third defect, closed

**The test is committed** — `git log -- scripts/test-replace-sandbox-failclosed.sh` resolves to `649b449`, whose subject is "the flag-conflict guard belongs". So the invisibility Flux reported is gone.

## And Flux had already verdicted it

**`ev` pass @ `649b449` on `packet/nimbus/sandbox-flag-guard-first`, subject: "the flag-conflict guard precedes every context check, and the criterion is falsifiable".** His evidence: 38/38, criterion 9 now has six assertions including the two his dispatch named, criterion 10 green, and **two mutations of his own both red** — F-1 (guard moved back to the late position) fails exactly the position criterion, F-2 (guard removed) fails three criteria including the suite's own mutation reporting its anchor vanished.

**So my measurement corroborates his rather than establishing anything new**, and the honest label is corroboration. What it adds is that I reached the position independently and that my mutation was verified to have mutated by `grep -c` before I read its result — which is the discipline that has failed me four times this week.

**His `not_verified` list is also worth repeating because it bounds what both of us know**: no real app roots or commands, no database writes, no sandbox replacement authorization, the running main application untouched, and **the other 32 assertions were not re-mutated this round.** So the guard and its criterion are verified; the rest of the suite passes at baseline and is not re-established.

## What I did not do

I did not modify any file in any worktree. My measurements were reads plus two suite runs in a throwaway `--shared` clone at `649b449`, with one mutation reverted from a byte-identical backup and confirmed with `git diff --quiet`; the clone is removed. I did not emit a verdict, open or cancel a card, merge, push anything but this document, install, restart any application, replace the sandbox, or write to the live database.
