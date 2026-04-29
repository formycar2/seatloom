# SeatLoom Coordination Rules v1.1

| 项目 | 内容 |
|------|------|
| 文档 | Coordination Rules v1.1 |
| 状态 | Active |
| 更新时间 | 2026-04-29 |
| Owner | Lyra (PO) |

---

## 1. Why this exists

Terminal output is ephemeral. Once screen scrolls, decisions and evidence disappear.

This rule set makes collaboration durable:

- important output must be persisted as files
- terminal messages become short summaries + file links
- every decision has owner, date, and evidence path

---

## 2. Source-of-truth hierarchy

When conflicts happen, use this order:

1. Approved product/architecture docs under `docs/`
2. Latest accepted coordination artifacts under `docs/coordination/`
3. Draft artifacts (explicitly marked draft)
4. Terminal/chat text (never source-of-truth by itself)

---

## 3. Mandatory writeback rule

No role may claim completion without writeback.

For every substantial output (review, task packet, acceptance, decision), write:

1. a durable artifact file under `docs/coordination/`
2. a one-line summary update in `docs/coordination/MEMORY.md`
3. a daily log entry in `docs/coordination/memory/YYYY-MM-DD.md`

Terminal/screen message must follow the fixed summary contract in Section 11.

---

## 4. Artifact structure (required)

```
docs/coordination/
  MEMORY.md
  memory/
    YYYY-MM-DD.md
  tasks/
    lyra/
    mira/
    nimbus/
    flux/
  reviews/
    YYYY-MM-DD-<topic>-review.md
  acceptance/
    YYYY-MM-DD-<topic>-acceptance.md
```

Naming rule:

- Use `ROLE-YYYY-MM-DD-<topic>-vN.md` for task packets
- Use `YYYY-MM-DD-<topic>-review.md` for review findings
- Use `YYYY-MM-DD-<topic>-acceptance.md` for acceptance verdicts

---

## 5. Role-specific persistence contract

## 5.1 Lyra (PO)

Must persist:

- decision memo
- task packets (owner/deadline/done-definition)
- acceptance verdict and go/no-go

Required sections:

1. Decisions needed
2. Decisions made
3. Action packets
4. Blockers
5. Stage-gate status

## 5.2 Mira (UX/UED)

Must persist:

- UI delivery report
- changed file list
- flow coverage checklist (implemented/partial/missing)

## 5.3 Nimbus (Engineering)

Must persist:

- implementation packet
- commands run + results
- schema/contract deviation notes

## 5.4 Flux (QA/Ops)

Must persist:

- acceptance report with severity
- reproducible runbook
- evidence links (screenshots/logs/commands)

---

## 6. Daily cadence (AI-native)

### 6.1 Start-of-day (Lyra)

Create/update `docs/coordination/memory/YYYY-MM-DD.md`:

- today goals
- active blockers
- expected handoffs

### 6.2 Mid-day check

Each owner writes a short progress block in the same daily file:

- done
- in progress
- blocked

### 6.3 End-of-day close

Lyra closes day log with:

- accepted outputs
- deferred items
- tomorrow first action

---

## 7. Stage-gate protocol

Before gate review:

- all required artifacts exist in files
- no critical decision exists only in terminal text

Gate decision file is mandatory:

- `docs/coordination/acceptance/YYYY-MM-DD-<gate-name>-decision.md`
- include: entry criteria, evidence list, verdict, follow-ups

---

## 8. Anti-drift guardrails

- If terminal output exceeds 80 lines, author must persist file first, then continue.
- If a role posts long analysis without artifact path, Lyra should reject and request writeback.
- No unresolved blocker may live only in chat; must be recorded in daily memory file.

---

## 9. Immediate adoption checklist

- [x] Lyra publishes first decision memo using this rule
- [x] Lyra creates today memory file under `docs/coordination/memory/`
- [x] Mira/Nimbus/Flux switch to file-first reporting
- [x] MEMORY.md adds one durable decision: "file-first coordination adopted"

---

## 10. Language policy (mandatory)

To reduce ambiguity and improve cross-seat consistency:

- All persistent artifacts (docs, task packets, reviews, acceptance files) must be written in English.
- Terminal/screen updates should be written in Chinese for fast operator readability.
- Artifact filenames stay ASCII and role/date/version based.

If an output violates this policy, Lyra should request rewrite before acceptance.

---

## 11. Stable terminal summary contract

To keep operator-facing updates easy to scan and stable across turns, all terminal/screen summaries must use this exact section order unless the human explicitly asks for a different one:

1. `Decision`
2. `Actions`
3. `Blockers`
4. `Artifact paths`

Rules:

- Keep the terminal content in Chinese, but keep the section labels exactly as written above.
- Under `Actions`, every bullet must include `owner / deadline / done-definition`.
- Under `Blockers`, write `- None` when there is no active blocker instead of dropping the section.
- Under `Artifact paths`, list every file that is needed to verify the update.
- Do not restate long analysis in terminal output; the terminal is a routing surface, not the durable record.

---

## 12. Direct seat dispatch rule

Lyra is responsible for driving the next owner directly when the owner is already known.

Rules:

- If the next action belongs to Mira, Nimbus, or Flux, Lyra should dispatch the packet or decision to that seat directly instead of asking the human to relay it.
- Direct seat-to-seat messages in tmux / PTY / seat chat must be written in English and must reference the governing artifact path(s).
- Human-facing terminal summaries should report the dispatch status after the seat has been notified.
- Ask the human for a decision only when product truth is still unresolved or when an explicit override is required.

---


## 13. Git branch classes

To avoid mixed-state verification and accidental cross-stream interference, SeatLoom uses four branch classes:

1. `main`
   - protected integration baseline;
   - contains only accepted work;
   - must stay buildable / testable at the agreed gate level.
2. `track/<domain>`
   - medium-lived branch for one active stream such as `track/infra-foundation` or `track/frontend-redesign`;
   - used when multiple seats work in parallel but the stream still needs one shared integration line.
3. `packet/<seat>/<packet-id>`
   - short-lived delivery branch for one bounded worker packet;
   - starts from its declared `track/*` branch or from `main` if no track branch exists.
4. `hotfix/<seat>/<topic>`
   - emergency or bounded repair branch;
   - must stay narrowly scoped and merge back immediately after acceptance.

Branch rules:

- No acceptance may target an unnamed floating workspace state.
- Every code-changing task packet must declare its base branch and expected delivery branch.
- `main` is never the default scratch branch for in-progress multi-seat work.
- Frontend and infrastructure should use separate `track/*` branches whenever they are both active.
- If a stream is not yet committed anywhere, it is still considered exploratory and cannot close a formal acceptance gate.

## 14. Commit-pinned verification rule

Flux verifies exact commits, not verbal delivery claims.

Required behavior:

1. Every verification packet must name:
   - target branch;
   - target commit SHA;
   - compare base commit if a delta review is intended.
2. Before verification starts, Flux must capture and report:
   - `git rev-parse --abbrev-ref HEAD`
   - `git rev-parse HEAD`
   - `git status --short`
3. If the checked-out commit does not match the packet's target commit, Flux must stop and return `HOLD`.
4. If the workspace is dirty and the packet does not explicitly authorize that state, Flux must stop and return `HOLD`.
5. Every verification delivery and every acceptance file must repeat the exact verified commit SHA.
6. Snapshot-only checks (for example, `scp` of an uncommitted workspace) may be used for diagnosis, but they do not close acceptance and must be labeled `PROVISIONAL` or `HOLD`.

Minimum fields for code-changing delivery artifacts:

- base branch
- base commit
- delivery branch
- delivery commit
- validation commands run
- artifact paths for evidence

## 15. Verifier bounded-fix exception

Default rule remains: Flux is a verifier, not a product implementation owner.

A narrow exception is allowed only when all conditions below are true:

1. Lyra issues an explicit `T3/fix` packet to Flux.
2. The fix is low-risk and bounded, such as:
   - build harness wiring;
   - missing asset/import;
   - verification script defect;
   - evidence-command typo;
   - non-behavioral configuration or warning cleanup.
3. The fix does not widen product scope, alter product logic, redesign UX, or mutate schema semantics.

If Flux performs such a fix:

- Flux must commit the change back to the current target branch immediately; no silent local edits are allowed.
- The commit message must start with `fix(flux):`.
- Flux must report `pre_fix_commit`, `fix_commit`, exact files touched, exact commands rerun, and the post-fix verification result.
- Lyra must treat the fix commit as a new verification target and cite that exact SHA in the acceptance artifact.

If the issue is larger than the bounded-fix exception, Flux must not patch it and must instead return findings for a Nimbus or Mira rework packet.


*This file is the collaboration operating contract for AI-native delivery in SeatLoom.*
