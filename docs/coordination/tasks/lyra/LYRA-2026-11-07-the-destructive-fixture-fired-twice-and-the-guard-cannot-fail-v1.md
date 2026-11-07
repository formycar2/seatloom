# 🔴 Standing duty: the destructive fixture fired a second time — **this time caused by Flux's own mutation run, which installed a 95-byte fake into `/Applications/SeatLoom-Sandbox.app` at 17:34.** He verdicted `fail`, named the mechanism, and did not hide that he caused it. And he found a second defect in the fixture that I confirmed: **the safety guard is a test that cannot fail, because both of its branches call `pass`.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-destructive-fixture-fired-twice-and-the-guard-cannot-fail-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | incident, measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0 — the fork failure from last round has eased (load 8.89 → ~4.3) and **last round's incident document is committed at `c1aeef8` and pushed, origin verified equal.** The ledger is 365 events and 73 dispatches. Cards are still three: two accepted, one cancelled, none pending.

## The incident, and it is the second one

**Flux verdicted `fail` at `0a7f1a7` on `packet/nimbus/candidate-final`**, and his `fail_reason` quotes the dispatch's own condition back at himself: *"if the fixture silently performs a real install or the mutation does not go red, verdict FAIL. The fixture silently performs a real install, and I caused one."*

**I verified the current state rather than taking his word:**

```
$ ls -l /Applications/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox
  size: 95 bytes   mtime: 7 Nov 17:34
$ head -c 100 <that file>
  seatloom-build-commit=ee8a2dc42ee2
  MUST_HAVE_PRESENT_MARKER
  seatloom-build-commit=ee8a2dc42ee2
```

**So the sandbox is still the fake, with a mtime from Flux's run** — Nimbus's accident at 16:00 was the first install and Flux's mutation run at 17:34 was the second. **The main application is intact**: 12,713,776 bytes, mtime Oct 28 22:16, pid 90805 up 5 days 6 hours.

### The mechanism, which is worse than "someone ran it twice"

**Section 13's mutation replaces the `--apply` path's refusal with an announcement that the marker is absent but not blocking. Its pass condition is that the refusal disappears:**

```bash
OUTM5="$(run_build --apply --allow-dirty MUST_HAVE_ABSENT_MARKER)"; RCM5=$?
if printf '%s\n' "$OUTM5" | grep -q "不要装"; then
  die "变异后 --apply 仍然拒绝安装 —— 第 12 条守的是空的"
else
  pass "变异后 --apply 不再拒绝安装 —— 第 12 条真的在守东西"
fi
```

**So the suite's own success condition is that the script continued past the check — and continuing past that check is what reaches the install.** The test that proves the guard works is a test that must defeat the guard to pass. **That is not a fixture with a bug; it is a fixture whose correctness condition and its destructiveness are the same event.**

## The second defect, which I confirmed myself

**Section 12 contains what looks like a safety assertion:**

```bash
grep -q 'SAND_APP="/Applications/SeatLoom-Sandbox.app"' "$SCRIPT" \
  && pass "确认了:被测脚本写死装到 /Applications —— 所以本 fixture 绝不跑 --apply 的安装段" \
  || pass "被测脚本的安装路径看起来可重定向 —— 但仍按最坏情况对待,不跑安装段"
```

**Both branches call `pass`.** I evaluated the expression against both inputs rather than trusting his reading:

| input | result |
|---|---|
| the real script, hardcoded `/Applications` | **PASS** (branch 1) |
| the same script with `SAND_APP` made redirectable | **PASS** (branch 2) |

**So the guard gates nothing.** Its comment states the intent and no code enforces it — the same shape as the `MUST_HAVE` defect he reported at `6dd68c5`, which is a check that exists, is wired to nothing, and reports a pass. **This is the third instance this week of a check that cannot fail, and the second one that turned out to guard a destructive action.**

## Why my dispatch is implicated again

**I asked for the `--apply` fixture, and my constraints said "no install, no `/Applications` replacement" without checking whether the install target was redirectable.** Nimbus hit it, reported it, and changed section 12 to assert the target and never run the install segment. **Flux then ran the section-13 mutation, which had not been changed, and hit it again.**

**So my omission propagated: I did not measure the redirectability, Nimbus's fix addressed the section he wrote but not the mutation that defeats it, and the mutation is the part that actually installs.** The lesson is the one I recorded last round and it now has a second data point: **before a fixture runs any path that deletes or overwrites, prove the target is redirectable — and that proof has to cover the mutations, not just the baseline.**

## What is right, which Flux listed rather than omitting

**His `what_is_right` and `verified_before_incident` fields record that the gap being closed was real** — `MUST_HAVE` has two check sites and only the `--build-only` one was ever mutated, and the `--apply` site is the more consequential. `run_build` taking an explicit mode removes the default that hid the gap, and all 17 call sites name their mode. **And section 14's reverse confirmation is genuinely useful**: under the same mutation it drives `--build-only` and shows that path still refuses, so a red in section 13 can only have come from the `--apply` site.

**So the work is sound in its measurements and unsafe in one mechanism, and he said both.**

## What he prescribed, and it is the right shape

**Section 13 must not run `run_build --apply` at all while `SAND_APP` cannot be redirected** — either drive *a copy* of the script whose `SAND_APP` points into the fixture (a one-line change to the copy, not to the script), or stop at asserting the anchor and the mutated file's validity. **And make the guard a guard**: replace the two-pass line with one that fails when the script hardcodes `/Applications`.

**That second prescription is the one that would have prevented both incidents**, because a guard that fails on a hardcoded install path makes the hazard visible to the next reader instead of announcing it and passing.

## What I did not do

I did not modify, restore, rebuild, install or replace anything. My measurements were reads: `ls -l` and `head` on both application binaries, `pgrep` and `ps` for the main process, the ledger for the verdict, `git show <commit>:<path>` for section 12's assertion and section 13's pass condition, and a two-input evaluation of the guard's grep against the real script and a redirected copy. I did not run the suite and did not mutate anything.
