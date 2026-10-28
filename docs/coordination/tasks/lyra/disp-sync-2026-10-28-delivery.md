# Delivery — priority-99 branch sync (merge track into packet/lyra/milestone-summary)

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | lyra-disp-sync-2026-10-28-delivery |
| status | completed |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| tags | branch-sync, merge, track, dispatch-ops |

## Done

Merged `track/infra-foundation` into `packet/lyra/milestone-summary` with `--no-ff` (no rebase, per
instruction) and pushed to **both** remotes, per `ENGINEERING_STANDARDS.md` §Repository topology.

| | |
|---|---|
| merge commit 1 | `7b4d0aa` — track @ `9b5173a` |
| merge commit 2 | `b823940` — track @ `3881d57` (track moved again mid-task) |
| branch tip | `b8239406823d882cb526a60a428ef9f0c5a83151` |
| pushed to | `gitlab` (primary) **and** `origin` (mirror); both verified by `git ls-remote` |
| `git merge-base --is-ancestor track/infra-foundation HEAD` | passes |

## The premise was wrong, and this is the part worth reading

The dispatch said *"your branch's pending cards can't be clicked right now"*. **They never could not
be.** Measured before merging:

- this branch (`packet/lyra/milestone-summary`) is **not** named by any row in `acceptances` with
  `status='pending'` — zero cards;
- and `packet/lyra/milestone-summary` **did not exist on either remote** before this push, so no card
  could have referenced it at all.

The stranded cards belong to other seats. Measured on the live database:

| When | Unclickable | Whose branches |
|---|---|---|
| before any action | 4 of 23 | `packet/nimbus/project-scoped-reads` (behind 1) |
| after I synced, mid-task | 11 of 23 | `packet/mira/knowledge-panel` (9 cards, behind 1) + others |

So the sync was performed as ordered — it is a legitimate instruction either way — but **the stated
reason did not apply to this seat**, and the seats that are actually stranded are **Mira and Nimbus**,
not Lyra. Reporting that rather than quietly filing the sync as done.

## Conflict resolutions — four, each a real decision

1. **`.seatloom/rollout-ack.txt`** (add/add) — **union.** Both acknowledgements kept, one line each
   (Lyra 03:19:47, Nimbus 03:19:29Z). The file's purpose is recording which seat came online; taking
   either side alone would erase a seat's record.
2. **`AGENTS.md`** (add/add) — **took HEAD (Lyra).** Track's side is Aegis's role file. Taking it
   would make this worktree behave as the L1 supervisor, working in the main checkout — wrong seat,
   wrong tree. Verified after both merges that HEAD's Lyra text is intact.
3. **`docs/coordination/ENGINEERING_STANDARDS.md`** (content) — **union.** Kept my v3 (the
   `## Read models` section, 2026-10-28) **and** track's new
   `## Pushing to track while a card is pending` section, in both the changelog and the body. The two
   rulings do not contradict each other; a blanket "ours" would have deleted Aegis's control, and a
   blanket "theirs" would have deleted my ruling. Version row `v3`.
4. **`docs/coordination/ROADMAP.md`** (content) — **union, verified non-duplicative.** Both sides are
   *purely additive to the same base* (measured: HEAD = base + 167 lines, track = base + 81 lines,
   neither deletes a base line), so both blocks are kept: Aegis's 10-16 snapshot and priority
   correction, then Lyra's 10-17 integration plan. My first attempt duplicated Aegis's sections
   (444 → 525 lines); I re-resolved with `git checkout -m` and confirmed 444 lines with each section
   heading appearing exactly once.

No `-X ours` / `-X theirs` strategy option was used anywhere. Each hunk was read.

## Verification

| Check | Result |
|---|---|
| `git merge-base --is-ancestor track/infra-foundation HEAD` | passes |
| conflict markers in any changed file | none |
| `bash scripts/verify-docs.sh HEAD` | `changed docs vs HEAD — all conform.` |
| `git diff --cached --check` | one hit, `AEGIS-2026-10-11-overnight-plan.md:134: new blank line at EOF`, traced to track's own commit `ff6b037` — **not** introduced by me, and out of scope for a sync dispatch |
| my artifacts survived both merges | `LYRA-2026-10-28-...rulings-v1.md`, `2026-10-27-...gate-class-rule-decision.md`, `2026-10-16-...acceptance-v1.zhs.md` all present at the tip |
| `AGENTS.md` still Lyra | yes |
| both remotes carry `b823940` | yes, `git ls-remote` on `gitlab` and `origin` |
| authored with `edit`/`write`, no code-modifying heredoc, no `cd <abs> &&` | yes |

## Escalation — three things for Aegis

1. **The sync target is moving.** Track advanced from `9b5173a` to `3881d57` *while the first merge was
   being resolved*. I merged again to reach the current tip. Until the track pushes stop, any sync is
   stale the moment it lands, and every card opened in the meantime is born unclickable. The control
   that is actually needed is the one Aegis already wrote into prose — *don't push to track while
   cards are pending* — and the `pre-commit` hook is scoped to `track/infra-foundation` in the main
   checkout only. Whether that hook is what ran for these pushes, I cannot tell from this worktree and
   did not check.
2. **Mira and Nimbus need the same dispatch.** 11 cards are stranded on `packet/mira/knowledge-panel`
   (9) and, before that, 4 on `packet/nimbus/project-scoped-reads`. This is reported, not acted on —
   merging another seat's packet branch is not mine to do, and `check-cards-clickable.sh` says so
   explicitly: *"Do NOT merge their branches into track yourself: that is the operator's Accept."*
3. **`.gitignore` came across with track's additions**, including `saih/` and `.saih/`. The worktree's
   own `.saih/mcp.json` is now ignored rather than untracked-but-visible. Noted, not changed — I did
   not touch ignore rules in a sync.
