# Task: Commit-pinned verification v2 — Onyx runtime triple (schema 009), in the Flux worktree

[Lyra -> Flux] Your v1 HOLD was correct on both counts and I have accepted it. Aegis has since
cleared HOLD-1 by giving you a dedicated worktree, and authorised the HOLD-2 script fix. This v2
re-runs only what was blocked, plus one substitution that answers your own order-dependence
argument. Everything you already proved in v1 stands and does not need redoing.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-runtime-triple-onyx-verification-v2 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| to | flux |
| supersedes | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-v1.md` |
| v1 delivery | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v1.md` (HOLD, accepted) |
| target branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` |
| **target commit** | **`580bcf4`** |
| **run location** | **`/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux`** |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v2.md` |
| tags | flux, verification, commit-pinned, schema-009, runtime-triple, onyx, worktree, v2 |

---

## 0. What changed since v1

**HOLD-1 cleared.** You now have a dedicated git worktree at
`/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux`. It shares `.git` with the main
checkout but is an independent working tree. I verified before writing this packet:

```
$ git worktree list
/Users/…/seatloom                 7652ad5 [track/infra-foundation]
/Users/…/seatloom-worktrees/flux  580bcf4 [packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1]
$ git -C /Users/…/seatloom-worktrees/flux status --short
(empty — clean)
```

It is already on the target branch at the target commit, and clean. The `pnpm tauri dev` watcher
(pid 91929) runs against the *main* checkout and does not touch the worktree, so §14.2 identity
now holds for the duration of your run.

**Rules:** `cd` into the worktree and do all verification there. **Do not switch branches in the
main checkout** — it stays on `track/infra-foundation` for coordination docs. Do not `git stash`
anywhere. Write your delivery doc into the **main checkout** and commit it on
`track/infra-foundation`, so the verification artifact does not land inside the delivery history
it is verifying.

**HOLD-2 authorised as a §15 bounded fix — with a hard stop, see §2.**

## 1. Carried forward from v1 — do not redo

§2.1 (scope constraints), §2.3 (verification queries), §2.4 (row state), §2.5 (retained columns
with deprecation comments), §2.7 (artifact completeness) all PASSed in v1 against `580bcf4`, and
your §2.8 finding on `18a69fb` is accepted (see §4). Re-state the v1 results in the v2 delivery by
reference; the scratch-database run in §3 will re-prove most of them from scratch anyway, which is
the point.

## 2. HOLD-2 — the §15 bounded fix, and where to STOP

`scripts/verify-postgres-baseline.sh` hardcodes `docker` in ~8 places; only `podman` is installed.
This is Onyx drift **D-5**, pre-existing at `9599136`.

**Authorised under COORDINATION_RULES §15** ("verification script defect"). Constraints:

1. The fix is **container-runtime detection only** — resolve a `$CTR` to `podman` or `docker`
   (prefer whichever is present; `podman` on this machine) and use it at every call site. Change
   **nothing** about what the script asserts, its step order, or its exit semantics. You are
   fixing how it invokes a container, not what a PASS means.
2. Commit message starts with `fix(flux):`. Report `pre_fix_commit`, `fix_commit`, exact files
   touched, and the diff in the delivery doc.
3. Commit it on `track/infra-foundation` in the main checkout, not on the packet branch.
4. Flag it for Nimbus review at the next gate. You are fixing your own harness, which you rightly
   flagged as sensitive in v1; the mitigation is that the change is mechanical and the diff is in
   the record.

### 2.1 DO NOT run the full `verify-postgres-baseline.sh` — hard stop

I checked the script before writing this. Line 59 is:

```
(cd "$INFRA_DIR" && docker compose down -v --remove-orphans …)
```

`down -v` **destroys the postgres volume**. The live `seatloom` database is the one Mr. Zhang's
running `pnpm tauri dev` app (pid 91929, up 30+ hours) is attached to, and SeatLoom hard-panics
without Postgres. Running the baseline verifier would wipe his database and take down his running
app as a side effect.

**So: fix the script, prove the runtime detection resolves correctly (e.g. echo the resolved
`$CTR`, or a `--dry-run`/early-exit probe), and STOP before the destructive reset.** Do not run
the full script. I am raising the destructive-run question with Mr. Zhang separately; until he
rules, §2.6 stays formally unproven and that is my call to carry, not a defect in your run.

## 3. The substitution that answers your order-dependence argument

Your v1 §0 point was the sharpest thing in the report: observing the live database cannot
distinguish "idempotent and order-correct" from "worked once, 43 hours ago", because
`seats_agent_runtime_mismatch = 0` depends on `18a69fb` having landed before `009` backfilled.

`scripts/apply-db.sh` applies schema `001–005` → seed `001–004` → post-seed migrations `≥006` in
the canonical order, driven by `$DATABASE_URL`, with no container exec. Use it against a **scratch
database**, not the live one:

```
psql <admin conn> -c 'CREATE DATABASE seatloom_verify;'      # drop first if it exists
DATABASE_URL=<…/seatloom_verify> bash scripts/apply-db.sh     # full ordered build from scratch
```

This is strictly better evidence than v1's observation and is non-destructive to the live database.
Then, in the scratch DB:

1. Re-run every §2.3 query and the §2.4 row-state queries. A from-scratch build that reproduces
   `sessions_missing_agent = 0`, `sessions_agent_runtime_mismatch = 0`,
   `seats_agent_runtime_mismatch = 0`, the six seat rows carrying `Stepcode`/`ClaudeCode`/
   `claude-opus-5[1m]`, and **`NULL` harness on all five April session rows**, proves
   order-correctness rather than observing a lucky outcome.
2. Apply `009` a **second** time against the scratch DB and paste both transcripts. That is the
   idempotency proof v1 could not attempt.
3. Drop the scratch database when finished, and say so.

If `apply-db.sh` cannot build from scratch for a reason that is Onyx's defect rather than
environmental, that is a **FAIL** with the transcript — exactly the kind of thing the live-database
observation was hiding.

## 4. `18a69fb` — ruled, so you need not re-litigate

Accepted as a **named exception**, per your v1 §4 recommendation, on your reasoning that it is not
idle ride-along work: the `009` seed section sets the triple for six seats and the `onyx` row
exists only because of `18a69fb`, so the two are separable in authorship but not in practice.

I also accept drift **D-1** against myself: my v1-era packet §1.1 "facts measured at base commit
`9599136`" table was read off a **dirty working tree** and is wrong. Committed state at `9599136`
has five seats (`lyra`/`mira` = `GeminiCli`, `flux` = `Custom`) and zero `data_engineer` bindings.
Verified today with `git show 9599136:…`. Recorded in `docs/coordination/memory/2026-09-30.md`.

## 5. Verdict and delivery

Same three-way verdict and the same rule as v1: partial verification is HOLD with the reason, not
PASS with a caveat. §2.6 being formally unproven is **not** a reason to withhold PASS — I have
explicitly carried that, and the §3 scratch-database run is the stronger substitute.

Delivery doc `…-delivery-v2.md` must contain: the worktree identity captures; the §2 fix diff with
`pre_fix_commit`/`fix_commit`; the §3 scratch-build transcripts including **both** `009` applies;
the re-run query results; a per-assertion table; confirmation the scratch DB was dropped and that
the main checkout was never branch-switched; and the verdict.

## 6. Looking ahead — a gap I found that will bite the Nimbus gate

The worktree has **no `ui/node_modules`, no `ui/dist`, and no `target/`** (verified: 5.7 MB total).
Fine for this packet, which is git + psql only. The Nimbus gate needs `cargo check`/`test`/`clippy`
plus `pnpm exec tsc --noEmit` and `pnpm build`, so it will need `pnpm install` in the worktree and
a `ui/dist` stub for `seatloom-tauri` to compile — the same stub the CI job uses. **Do not do that
now.** I am raising it so the Nimbus gate arrives with it already solved instead of HOLDing on it.

---

*Dispatched by Lyra · 2026-09-30 · supersedes v1 · worktree clears HOLD-1 · §15 bounded fix for
D-5 with a hard stop before the destructive reset · scratch-database build answers the v1
order-dependence argument*
