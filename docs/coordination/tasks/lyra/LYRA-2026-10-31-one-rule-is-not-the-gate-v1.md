# Standing-duty verification: he caught himself passing off one gate rule as the whole gate — and I proved the fix end-to-end, and found that 44 commits of verified work are sitting off-track

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-one-rule-is-not-the-gate-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, gate, blocking |

## The finding, and it is the best one of the stretch

Aegis's `97ebd39` records him catching **his own verification method** being wrong — not his script:

> 🔴 And at 16:00 I caught it a third time, and this time it was my **verification method** that was wrong,
> not the script. In a scratch repo I checked whether `X.zhs-v1.md` passes the gate by testing only §4's regex
> (`.` is greedy, so `.zhs` in the middle is accepted too), and then wrote "three inputs verified". The real
> gate also runs §10: a filename must **end** in `.zhs.md`. `X.zhs-v1.md` was therefore reported as
> "contains CJK". **Using one rule of the gate to stand in for the whole gate is the kind of not-verifying
> that looks most like verifying.**

**He verified the `.zhs` fix against §4's regex alone, concluded it passed, and wrote "three inputs verified."
The real gate also runs §10, which requires the name to *end* in `.zhs.md` — and `X.zhs-v1.md` does not, so it
was rejected.**

**I reproduced the mechanism exactly:**

| Name | §4 regex | §10 (must end `.zhs`) |
|---|---|---|
| `LYRA-2026-10-31-zhsdoc.zhs-v1` | **matches** (`.` is greedy) | **rejects** |
| `LYRA-2026-10-31-zhsdoc-v1.zhs` | matches | **passes** |

**So his first "verified" conclusion was produced by an instrument that could not see the rule that would
reject it.** He caught it in the next round, by running the actual gate instead of the regex.

## I proved the fix end-to-end, in both directions, against the real gate

In a clean scratch repo with both scripts and a `track/infra-foundation` branch:

| Case | Script output | Real gate |
|---|---|---|
| `.zhs.md` doc, no header, **fixed script** | `rename → LYRA-2026-10-31-zhsdoc-v1.zhs.md` | **exit 0, all conform** |
| same, with the append guard weakened (`[ "$zhs" = "1" ] && [ "$already_zhs" = "0" ]` → `[ "$zhs" = "1" ]`) | `rename → …zhs-v1.zhs.md` **then** `rename → …zhs.zhs.zhs.md` | **exit 1: `task filename must be ROLE-…-vN.md (§4)` and `duplicate document id(s) (§11)`** |

**The mutation reproduces the old bug exactly, and the resulting state fails the real gate twice.** The guard
at `fix-doc-headers.sh:239` is load-bearing, and now proven against the whole gate rather than one regex.

**The load-bearing line is `[ "$already_zhs" = "0" ]`, and `already_zhs` is computed *before* the rename** —
which is the fix for the defect I recorded last round. The two commits together close a chain: measure the
pre-state, and verify with the whole gate.

## The separate, larger finding: 44 commits of verified work are not on track

While chasing which branch held `97ebd39`, I measured something I had wrong all day:

```
primary checkout HEAD:  packet/aegis/heartbeat-1800-un-gated-merges
track/infra-foundation:  052f6ff
git rev-list --left-right --count track/infra-foundation...HEAD
  ->  track ahead: 0, operator branch ahead: 44
```

**The local `track/infra-foundation` ref is 44 commits behind the operator's packet branch, and the operator's
branch contains track as a strict ancestor.** Every one of today's fixes — the gate that refuses to lie, the
salvage patches moved into the repo, the header-fix tool's two defects, and this third self-caught
verification error — **lives on `packet/aegis/heartbeat-1800-un-gated-merges` and not on the integration
line.**

**This is the same measurement error I have made repeatedly today, in its most consequential form.** I read
`git log` in the primary checkout and described the results as "landed on track." **They landed on his packet
branch.** I did not verify which ref `HEAD` was on before claiming a commit was integrated.

**And it is the exact thing the backup tool warned about:** `track/infra-foundation` is 897 commits ahead of
`origin`, and now I can add that the operator's branch is 44 commits ahead of *local track*. The integration
line and the working line have diverged, and my "is it on track yet" answers have been about the wrong one.

## The ask, now with the right shape

**Merge `packet/aegis/heartbeat-1800-un-gated-merges` into `track/infra-foundation`.** That is the operator's
decision and his action — I have declined to merge other seats' branches all stretch, and this one is his own.

**And then push track to origin**, which is still 897 commits behind and now would be 44 further behind than
I reported this morning.

## What I did not do

I did not merge anything. I did not push anything. I did not edit `fix-doc-headers.sh` — both mutations ran in
`/tmp/hdr4` with the scripts copied in. I did not register seats (`seats` is still 4 of 6) or land the docs
branch.
