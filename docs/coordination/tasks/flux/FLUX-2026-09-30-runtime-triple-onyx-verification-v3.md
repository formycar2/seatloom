# Task: Commit-pinned re-verification v3 — Onyx runtime triple, after the apply-db ordering fix

[Lyra -> Flux] Your v2 FAIL was correct and Onyx has fixed it. This is the re-verify at the new
merge candidate. Narrow packet: re-run what failed, confirm nothing else moved, and independently
check the one claim that would let me merge despite a known open defect.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-runtime-triple-onyx-verification-v3 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| to | flux |
| supersedes | `…-verification-v2.md` (FAIL, accepted) |
| target branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` |
| **target commit** | **`d4f40b5`** |
| previous target | `580bcf4` (FAILed) |
| onyx delivery | `docs/coordination/tasks/onyx/ONYX-2026-09-30-apply-db-ordering-fix-delivery-v1.md` |
| run location | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux` — already on `d4f40b5`, clean |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v3.md` |
| tags | flux, verification, commit-pinned, runtime-triple, onyx, v3 |

---

## 0. State, verified by me before dispatch

```
$ git worktree list
/Users/…/seatloom                 3bf9017 [track/infra-foundation]
/Users/…/seatloom-worktrees/flux  d4f40b5 [packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1]
```

Both clean. Your worktree is already on the target. The main checkout stays on `track` — do not
switch it. Commit your delivery doc in the **main checkout** on `track/infra-foundation`.

**Note on the branch ref:** Onyx could not advance the packet branch itself (your worktree held it
checked out) and correctly flagged this rather than forcing it. I moved the ref. One transparency
note so you can trust what you are verifying: I first moved it to `cfdd647`, one commit short,
which dropped Onyx's delivery-doc commit; I caught it from the reflog and fast-forwarded to
`d4f40b5`. Nothing was lost and no commit was rewritten — `580bcf4 → ef4af6b → cfdd647 → d4f40b5`
is a clean ancestry chain. Verify that yourself as part of §1; if the ancestry is not clean,
that is a **HOLD** on me, not on Onyx.

## 1. Identity and ancestry

Standard §14.2 capture in the worktree. Plus:

```
git merge-base --is-ancestor 580bcf4 d4f40b5     # must be true
git log --oneline 580bcf4..d4f40b5               # expect exactly: ef4af6b, cfdd647, d4f40b5
git diff --stat 580bcf4..d4f40b5 -- '*.sql'      # must be EMPTY — no .sql changed
```

The last one is the packet's central boundary: Onyx was forbidden to touch `.sql`, and claims it
did not. Prove it.

## 2. Re-run what failed

The v2 FAIL was `apply-db.sh` exiting 3 from an empty database. Re-run it at `d4f40b5` against a
**fresh scratch database**, expect **exit 0**:

```
CREATE DATABASE seatloom_verify_v3;      -- drop first if present
DATABASE_URL=<…/seatloom_verify_v3> bash scripts/apply-db.sh
```

Then in that scratch build re-run the assertions, as in v2:

- `sessions_missing_agent = 0`, `sessions_agent_runtime_mismatch = 0`,
  `seats_agent_runtime_mismatch = 0`
- `DISTINCT` domains inside the value-domain contract
- six seat rows carrying `Stepcode` / `ClaudeCode` / `claude-opus-5[1m]`
- **`NULL` harness on all five April session rows** — still the one that must not slip

Drop the scratch DB afterwards. Same tooling caveats as v2: shim `psql` if needed, disclose it, and
state that `apply-db.sh`'s host-`psql` path stays unexercised.

## 3. Independently check Onyx's blamelessness claim — the crux

Onyx reports (§4) that a **second consecutive** `apply-db.sh` run still fails, exit 3,
`null value in column "project_id" of relation "sessions" violates not-null constraint`, and claims
this is **pre-existing on `track/infra-foundation`**, independent of `009`, and therefore not a
regression introduced by this merge.

**That claim is what would let me merge a candidate with a known failing second run. Do not take it
on trust — reproduce it.** Onyx ran its control by `git archive`-ing `track` to `/tmp`; run your own
control however you find sound. The question to answer:

> Does `track/infra-foundation`, with no `009` anywhere, also exit 0 on run 1 and exit 3 on run 2
> with the same `project_id` error?

- If **yes** — the merge candidate is at exact parity with `track` on the first-run path, the
  second-run defect is pre-existing, and this is a PASS with a recorded open defect.
- If **no** — the merge introduces a regression, and that is a **FAIL**.

## 4. Verdict

PASS / HOLD / FAIL, same rule: partial verification is HOLD with the reason, not PASS with a
caveat.

Two things explicitly **not** reasons to withhold PASS, because I am carrying them:

1. **`verify-postgres-baseline.sh` still unrunnable** (podman has no compose provider — your v2
   §2.2 finding). Formally unproven, mine to carry. Do not run it; `compose down -v` destroys the
   volume the live app is attached to.
2. **The second-run `project_id` failure**, provided §3 confirms it is pre-existing. It needs a
   `.sql` change and a scoped follow-up packet, which I will issue; it is not this gate's business.

## 5. Delivery

`…-delivery-v3.md` in the main checkout on `track`: identity + ancestry captures; the `.sql`
no-change proof; the scratch build transcript with exit code; the re-run assertion table; your §3
control experiment with its transcripts and your independent conclusion; the verdict; and
confirmation the scratch DB was dropped and the main checkout never switched.

---

*Dispatched by Lyra · 2026-09-30 · re-verify at `d4f40b5` after Onyx's apply-db ordering fix ·
v2 FAIL accepted and addressed · §3 is the crux: verify the blamelessness claim, do not inherit it*
