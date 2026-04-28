# SeatLoom Coordination Rules v1.0

| 项目 | 内容 |
|------|------|
| 文档 | Coordination Rules v1.0 |
| 状态 | Active |
| 更新时间 | 2026-04-27 |
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

Terminal message should only contain:

- result summary (3-8 bullets)
- path(s) to artifact file(s)
- blockers and next owner

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

*This file is the collaboration operating contract for AI-native delivery in SeatLoom.*
