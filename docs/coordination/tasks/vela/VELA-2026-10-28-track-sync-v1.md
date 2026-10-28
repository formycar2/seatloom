# VELA-2026-10-28-track-sync-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-28-track-sync-v1 |
| status | delivered (merged, both criteria pass, pushed) |
| author | vela |
| date | 2026-10-28 |
| version | v1 |
| dispatch | none — operator instruction relayed via poke, not a governed dispatch |
| depends_on | VELA-2026-10-27-acceptance-card-blocked-no-verdict-v1 |
| tags | sync, merge, track, acceptance, cards, measurement |

The operator asked, 2026-10-28: `git merge track/infra-foundation` into my packet
branch and push, no rebase, because pending cards had become unclickable. This is
a **direct operator instruction**, not a governed dispatch — `get_next_dispatch`
returned nothing ready throughout, and the ledger shows the three priority-99
emergency sync dispatches target `seat-mira-001`, `seat-nimbus-001` and
`seat-flux-001`, none of them me.

## The premise did not apply to me, and I say so because it changes the risk

**Both of my cards were already granted on 2026-10-16**, days before Aegis's
pushes on 10-28:

| card | last event | state |
|---|---|---|
| `acc-disp-wash-vela-p1` | `AcceptanceGranted` @ 2026-10-16T06:54:32Z | granted |
| `acc-disp-wash-vela-p2` | `AcceptanceGranted` @ 2026-10-16T06:54:31Z | granted |

So no pending card of mine was blocked. The sync was still worth doing — my
branch really was stale — but it was hygiene, not an unblock. That matters
because a merge done under "cards are burning" pressure is exactly when a
careless rebase would destroy commit ids; here the stakes were lower, and I still
merged rather than rebased, because both cards name commits on this branch and a
rebase rewrites those ids.

## One measurement that would have misled anyone measuring naively

**`origin/track/infra-foundation` is a stale GitHub ref** — tip `4c13ffc`,
**2026-09-30**. Measured against it, this branch looks **244 commits behind**.
The live base is the **local `track/infra-foundation` at `7b72832`
(2026-10-28)**, which the acceptance tool's containment guard actually reads.
Against the real base the branch was **301 commits behind**.

I recorded the stale-remote trap in my 2026-10-27 delivery; it bit again here, so
it is worth repeating: any seat measuring divergence against `origin`'s copy is
reading a month-old base and will over-report by ~243 commits.

## What I did

1. **Dry-ran the merge in a scratch worktree first**, detached at my HEAD, so a
   surprise could not touch my real worktree. Result: *"Automatic merge went
   well; stopped before committing"*, zero unmerged paths.
2. **Verified my work survived the dry run**: `session_key.py`, its 29 tests, the
   `archived_sessions` index, and the `validate_session_keys` call inside
   `s5_writein.py` all present; `.cargo/config.toml` byte-identical on both
   sides, so nothing was lost.
3. **Ran the merge for real** — `--no-ff`, merge commit `e926c86`, so the sync is
   a visible event in history rather than a silent fast-forward.
4. **Ran the acceptance criteria** the operator specified:

```
$ git rev-list --count packet/vela/washing-pipeline..track/infra-foundation
0
$ git merge-base --is-ancestor track/infra-foundation packet/vela/washing-pipeline
SUCCESS
```

5. **Re-verified on the merged tree**: the wash suite is **66 tests, 0 skipped,
OK**; `s5_writein.py` resolves **87/87 records across 10 sessions, 0 validation
problems**, and still emits the manifest.
6. **Pushed** to `gitlab` (the live remote): `a79cf70..e926c86`. Re-fetched and
   confirmed local and remote are **0 / 0** divergent.

## The three-test discrepancy, resolved rather than ignored

The scratch dry-run reported **63 tests, my branch reports 66**. I did not wave
it off as flaky. Cause: `tests/test_l2_pointers.py` runs **3 tests** when the
scratch artifacts are present but **0** when they are not, because it resolves
paths relative to its own file location — and a scratch worktree has no
`.local/wash` scratch dir. Nothing was lost; the merged tree in my real worktree
runs all 66.

That is the honest version, but the lesson stands: a test count that changes with
where the tests run from is a fragile gate, and I would rather have found the
cause than accepted "close enough".

## What I did not do

Did not rebase. Did not merge my branch into `track` — that is the operator's
Accept. Did not delete or rewrite commits. Did not open any new acceptance card,
as the operator instructed. Did not touch the live database, and did not act on
the three emergency dispatches that are not mine to act on.
