# Task: Commit-pinned verification — UM0 live tmux mirror + sample-data marking

[Lyra -> Flux] Mira's UM0 slice. This is the first packet whose real acceptance criterion
**cannot be established by any command** — the bar is that the mirror visibly moves. Read §3
before you start, and §2 before you run `prettier`.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-um0-live-mirror-verification-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| to | flux |
| verifies | `docs/coordination/tasks/mira/MIRA-2026-09-30-live-mirror-um0-v1.md` |
| delivery under test | `docs/coordination/tasks/mira/MIRA-2026-09-30-live-mirror-um0-delivery-v1.md` (on Mira's branch) |
| target branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| **target commit** | **`0701bf9`** (tip, carries the delivery doc) |
| code commit | `cd5788f` |
| base | `324e185` |
| run location | `/Users/…/seatloom-worktrees/flux` — **detached at `0701bf9`**, clean |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-09-30-um0-live-mirror-verification-delivery-v1.md` |
| tags | flux, verification, commit-pinned, um0, ui, live-mirror |

---

## 0. State, prepared by me

Your worktree is detached at `0701bf9` (Mira's worktree holds the branch). `ui/node_modules`
installed, clean. **Mira's and Nimbus's worktrees are off limits** — Nimbus is working the
`pipe-pane` fix in parallel right now.

This is a **frontend-only** gate: `git diff --name-only 324e185..0701bf9` must show zero files
outside `ui/` and `docs/`. Prove it.

## 2. The `prettier` trap — Mira flagged it, and it had already bitten my prep

There is **no `.prettierignore`** in the repo or in `ui/`. I confirmed. So `prettier --check .`
also walks `ui/dist`, and whether you ran `build` first changes the file count (Mira measured
96 → 103).

**I removed `ui/dist` from your worktree before dispatching** — it held a full build from the
step-2 gate, not just a stub, which would have skewed your counts and produced a confusing
disagreement with Mira's numbers.

**Run `prettier --check` BEFORE `build`**, or `rm -rf ui/dist` immediately before it. State the
file count you saw and the order you used, so your numbers and Mira's can be reconciled rather
than argued about.

(That the repo has no `.prettierignore` is a real defect — note it as a finding; it is not
Mira's to fix in this packet.)

```
pnpm --dir ui typecheck
pnpm --dir ui exec eslint .
pnpm --dir ui exec prettier --check .     # BEFORE build; report the count
pnpm --dir ui build
```

## 3. The bar no command can prove — does the mirror actually move?

Mira's claim is that the view lists the **real** seats from `listTmuxSessions()` and streams their
live panes. A green typecheck establishes none of that.

**Attach to a real seat and cause something to change in that tmux, then show the UI pane
reflected it.** Screenshot, captured pane diff, recorded sequence — your call, but it must show
**movement**, not a static list. State which seats you exercised.

### 3.1 Two hazards, both mandatory to handle

1. **Only a pane's FIRST attach works.** `pipe-pane -o` toggles: the second attach to the same
   pane silently closes the pipe (`pane_pipe=0`, zero bytes) while the command still returns
   success. I reproduced this myself. **Attach only to panes reading `pane_pipe=0`.** A re-attach
   failure is **NOT a Mira defect** — it is a known backend bug, Nimbus is fixing it in parallel
   (`NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1`). Do not fail her packet for it.

2. **Your attaching will toggle pipe state on live seats**, exactly as Mira's did. Before you
   start, record `pane_pipe` for all five seats. When you finish, **reset every one to 0** and
   confirm:

   ```
   for s in "Flux-Quality&Ops-seatloom" "Lyra-po-seatloom" "Mira-UX/UED-seatloom" \
            "Nimbus-TechArchi-seatloom" "Onyx-data-seatloom"; do
     tmux display-message -p -t "$s" '#{pane_pipe}'
   done
   ```

   All five read `0` right now — I checked. Leave them that way, kill no session, and say so.
   If Mr. Zhang has terminals open in the shared app they may need re-attaching afterwards;
   note it if you think you disturbed them.

## 4. The other half — is mock unmistakable?

Every seed-backed view must be unmistakably labelled **"SAMPLE DATA — not yet from live capture"**
or hidden. The test is a person's: could someone opening this app tell real from mock **at a
glance, without asking**? That was Mr. Zhang's original complaint and it is the point of the
packet. Say plainly whether it is met — this is a judgement, and I want yours, not a checklist tick.

Confirm the V1 AppShell was preserved, not replaced.

## 5. Scope constraints

- Zero files under `crates/`, `src-tauri/`, `infra/`; zero `.sql`.
- **`ui/src/lib/types-dto.ts` and `ui/src/lib/dto-to-v1.ts` untouched** — fenced in Mira's packet
  because Nimbus's step-2 packet permitted minimal DTO type fixes there. Verify they are unchanged;
  that fence is what made the two parallel packets safe.

## 6. Verdict and delivery

PASS / HOLD / FAIL. Partial verification is HOLD with the reason, not PASS with a caveat.

Delivery doc in the **shared checkout** on `track`: identity captures; the §1 scope proof; §2
commands with the file count and ordering; **your own §3 liveness evidence** and which seats you
used; the before/after `pane_pipe` table with all five restored to 0; your §4 judgement; §5
confirmations; and the verdict.

---

*Dispatched by Lyra · 2026-09-30 · UM0 · §3 is the real bar and no command can establish it ·
re-attach failure is a known backend bug, not Mira's defect*
