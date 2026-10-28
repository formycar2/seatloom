# UX Review — Dispatch Inbox (派单 Tab)

- **Date**: 2026-10-09
- **Author**: Mira (UX/UED seat)
- **Subject**: SeatLoom dispatch-inbox view (派单 tab)
- **Dispatch**: `disp-mira-uxreview`
- **Basis**: design description — dispatches grouped by seat; status badges 就绪 / 进行中 / 阻塞 / 完成; priority; dependency edges; 5s auto-refresh

---

## 1. Verdict

The view is structurally sound and mostly readable. Three things already work well and should be preserved as-is:

1. **Status badges are self-explaining.** 就绪/进行中/阻塞/完成 covers the four states a seat actually experiences, and each badge reads at a glance without a legend. The colour-plus-label pairing (never colour alone) is the right call.
2. **Grouping by seat turns a list into a work queue.** The grouping matches the operator's mental model ("what does this seat have to do?"), not the database's ("what dispatches exist?"). This is the single most valuable decision in the view.
3. **Priority is visible and honestly bounded.** Priority is shown but presented as non-actionable context rather than a sort control or a promise of ordering. The view does not imply the operator can re-prioritise work that the routing engine owns. That honesty prevents a broken promise.

The suggestions below are refinements, not restructures — none of them require re-opening the grouping, badge, or priority decisions above.

---

## 2. Improvement suggestions

### S1 — Make the seat group header the summary line, not just a label

**Problem.** A group header that only names the seat forces the operator to scan the rows beneath it to answer the first question they actually have: *does this seat need me right now?*

**Fix.** Turn each group header into a one-line seat status strip: seat name + role, then counts for the four badges (`就绪 2 · 进行中 1 · 阻塞 1 · 完成 5`) and, when present, a single red token for the oldest blocked item (`阻塞 2h`). Collapsed groups should show the same counts inline so the whole inbox is scannable without expanding anything.

**Why.** It makes the grouping from a visual convenience into a triage instrument: the operator reads one line per seat and knows where to spend attention, instead of reading every row.

### S2 — Surface the blocking reason on the row, not one level down

**Problem.** 阻塞 is the only badge that implies an obligation (someone must act), and it is the one badge that explains nothing. A row that says only "阻塞" sends the operator into the detail view to learn who or what is blocked.

**Fix.** Every blocked row carries a short inline reason and, where a blocker is known, the blocking reference — e.g. `阻塞 · 等 Onyx 的 schema-008` or `阻塞 · 上游 dispatch 未完成`. Truncate with a title/tooltip rather than wrapping, so row height stays uniform. Keep the full chain in the detail view; the row only needs the immediate cause.

**Why.** 阻塞 items are the ones an operator must route or escalate. Moving the reason up one level is the difference between a status board and a work queue.

### S3 — Show dependency edges as direction and count, not as drawn arrows

**Problem.** Drawn dependency edges inside a grouped list are the highest-cost element in the view: they cross group boundaries, they conflict with grouping as the primary structure, and at realistic dispatch counts they degrade into noise. If they are drawn at all, they are drawn badly.

**Fix.** Replace edges with a compact affordance on the affected row: an upstream/downstream indicator with a count (`↑2 依赖` / `↓3 下游`), clickable to expand the referenced dispatch ids inline. Reserve a real graph rendering for the detail view of a single dispatch, where the neighbourhood is small enough to draw legibly.

**Why.** This keeps the dependency information — which is genuinely useful for answering "why is this blocked / what does unblocking this release?" — while removing the layout cost and the false implication that the inbox is a graph workspace.

### S4 — Make the 5s auto-refresh honest and non-destructive

**Problem.** A 5s poll that silently re-renders a list the operator is reading is disruptive: rows shift under the cursor, scroll position and expansion state can reset, and a status that flips 进行中→完成 between refreshes is never perceived as a change. The refresh is currently invisible as a mechanism and destructive as an experience.

**Fix.** Four parts, in priority order:
- **Preserve reading state across polls.** Keep expansion, selection, and scroll anchor keyed on dispatch id so a refresh never moves what the operator is looking at.
- **Show liveness explicitly.** A small "上次刷新 hh:mm:ss" line plus a subtle pulse on the poll tick, so the operator knows the data is live rather than stale.
- **Mark changes, don't just replace them.** When a row's status changes between polls, briefly highlight it (`就绪 → 进行中`) so the transition is observed instead of inferred.
- **Give the operator a pause.** Pause auto-refresh while a row is selected or the pointer is over the list, with an obvious manual-refresh affordance; never leave a 5s timer as the only way to get fresh data.

**Why.** Auto-refresh earns trust only when it never costs the operator their place. Otherwise the operator learns to distrust the view and stops using it as the source of truth.

### S5 — Give 就绪 items an explicit next-action affordance

**Problem.** 就绪 currently reads as a state but not as an offer. The operator sees that work is available but has no in-view way to act on it, so the inbox reports status while the actual dispatch happens elsewhere.

**Fix.** On each 就绪 row, expose the one action the state implies — a primary "派发" control (or a keyboard-driven equivalent), with the target seat pre-filled from the group. Keep it disabled-with-reason rather than hidden when the action is genuinely unavailable (e.g. the seat already holds an in-flight dispatch), so the constraint is visible rather than mysterious. Leave 阻塞 rows as read-plus-escalate only; do not imply they can be pushed.

**Why.** It closes the loop between "seeing ready work" and "sending ready work" without adding a second surface to maintain, and the disabled-with-reason pattern keeps the non-actionable claims from S4's honesty principle intact.

---

## 3. Suggested sequencing

| Order | Suggestion | Effort | Impact |
|-------|-----------|--------|--------|
| 1 | S4 — refresh state preservation + liveness + change marking | Medium | High — protects every other element in the view |
| 2 | S2 — inline blocking reason on 阻塞 rows | Low | High — unblocks the only obligation-bearing badge |
| 3 | S1 — seat group header as status strip | Low | Medium-High — makes grouping a triage tool |
| 4 | S3 — edges as counts, graph reserved for detail | Medium | Medium — removes layout cost |
| 5 | S5 — 就绪 row action affordance | Medium | Medium — closes the action loop |

S4 first: it is the only suggestion that affects whether the operator can trust any of the others.

---

## 4. Out of scope

- The actual dispatch/write path and any backend semantics behind priority ordering or dependency resolution.
- Whether 5s is the right interval (a timing question for Flux/Nimbus once liveness is observable per S4).
- Visual styling details, which belong to the theme/token system already in place.
