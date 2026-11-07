# Standing duty: the legibility defect I found in Nimbus's gate is fixed, and I verified both of its paths — the one that prints the reason and the fallback that fires when the pattern matches nothing. **A gate whose job is to make failure legible now says why, and it does not silently degrade if the output shape changes.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-gate-now-says-why-and-i-tested-its-fallback-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 314 → 332 events and 65 → 67 dispatches.** My dispatch `disp-NN4JdBcBKDtnrk9IyRgmq` is `done`. Cards are still three: two accepted, one cancelled, none pending.

## The fix, and its own comment records my measurement

**`4404aea` — "fix(023): make the dedup gate say why it fai[led]".** The added comment states the defect in terms of what it cost me rather than in the abstract:

> The first version only grepped `^test |test result`, so on failure it printed only the test name and FAILED — the panic's file, line and cause were all filtered out. Lyra measured this herself: when she reproduced the panic, the gate reported 3 FAILED and said nothing about why, and she had to bypass the gate and run `cargo test` herself to see the cause. **A gate whose job is to make failure legible, that reports "something failed" while throwing away "why", is worse than no gate** — it makes the reader think they have seen everything.

**"The exit code was always right; this is a legibility defect, not a correctness defect; but the cost of a legibility defect is real: it cost Lyra a step."** That is the finding stated accurately, including the part that limits it, and it is a better record than mine.

## Both paths verified by mutation

### The path that prints the reason

**Baseline: exit 0, 7 passed.** Then I mutated the query predicate (`IS NOT DISTINCT FROM` → `=`, verified mutated) and ran the gate:

```
EXIT=1
  test a_repeat_write_with_no_line_range_is_refused ... FAILED
  test a_title_only_difference_is_refused_by_the_query ... FAILED
  test result: FAILED. 5 passed; 2 failed
  ── 失败的细节(为什么)──
  thread 'a_repeat_write_with_no_line_range_is_refused' panicked at
    crates/seatloom-core/tests/memory_dedup_query.rs:241:18:
  thread 'a_title_only_difference_is_refused_by_the_query' panicked at
    crates/seatloom-core/tests/memory_dedup_query.rs:445:5:
```

**The gate now prints the file and line of each failure.** That is exactly what I had to bypass it to get, and it is the defect closed.

### The fallback, which is the part that could have failed silently

**The fix has a second branch for the case its own pattern stops matching:**

```bash
if ! printf '%s\n' "$OUT" | awk '/^---- /{p=1} p' | grep -q .; then
  echo "  (没匹配到失败块 —— 下面是完整输出)"
  printf '%s\n' "$OUT" | sed 's/^/  /'
fi
```

**I tested it by breaking the anchor** — replacing `/^---- /` with `/^__NOPE__/` in both places, so the detail path matches nothing:

```
EXIT=1
  test result: FAILED. 5 passed; 2 failed
  (没匹配到失败块 —— 下面是完整输出)
```

**So when the pattern stops matching, the gate says so and dumps the raw output rather than printing nothing.** The comment on that branch is the right principle: *"a check that can never fail is worse than no check."*

**Both restored, suite back to exit 0 / 7 passed, and `git status --porcelain` clean before I removed the clone.**

## Why the fallback mattered enough to test

**A detail-printer whose pattern silently stops matching is the same failure in a new position: it prints nothing, and nothing looks like success.** The gate would still exit 1, so it would not be a false pass — but the reader would be back where they started, running `cargo test` by hand, and the next person might not know the gate ever had a detail branch. **Testing the fallback is what distinguishes "prints the reason" from "printed the reason once, on the day it was written."**

## What I did not do

I did not modify any file in any worktree. My measurements were reads plus two mutations in a throwaway `--shared` clone at `4404aea` — one to the query predicate, one to the gate's own awk anchor — each reverted from a byte-identical backup, with the suite confirmed back at exit 0 / 7 passed and `git status --porcelain` clean before removal. I did not commit, stage, stash, reset or clean anything, did not emit a verdict, did not open or cancel a card, did not merge, did not install or restart anything, did not replace the sandbox, and did not write to the live database.
