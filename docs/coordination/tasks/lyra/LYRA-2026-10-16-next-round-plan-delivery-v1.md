---
template: T5
subtype: delivery
doc_id: LYRA-2026-10-16-next-round-plan-delivery-v1
title: Delivery — the next-round iteration plan, its five slices, and the measured "which modules might be broken" section
status: active
author: lyra
doc_date: 2026-10-16
version: v1
depends_on: LYRA-2026-10-16-next-round-plan-v1.md (same commit)
tags: delivery, lyra, iteration-plan, decomposition, provenance, lineage
---

# Delivery — `disp-vEU7q7-asGPAhOmgNAxgK`

## What was delivered

The next-round iteration plan at
`docs/coordination/tasks/lyra/LYRA-2026-10-16-next-round-plan-v1.md`
(`packet/lyra/milestone-summary` @ `1cd9a1e`), plus five dispatches enqueued into the
governed graph and two ledger events.

| PRD hard rule | Where it is satisfied |
|---|---|
| A user story | Plan §3 — the milestone stated as one thing he can see or do |
| An interaction flow | Plan §2.1 (what is missing), §3.2 (per-slice behaviour), §4.3 (what I cannot measure) |
| A screen / UI surface definition | Plan §3.2 slice 4 and §2.2's per-surface table |
| Pain / omission cost / behaviour change | Plan §2.1 items 1–4 and §4.1's nine measured defects, each with a one-minute click test |
| Out of scope, with reasons | Plan §5 |

## The dispatch graph, as it stands now

| Dispatch | Seat | Live status | Edge |
|---|---|---|---|
| `disp-lineage-readmodel` | Nimbus | **in_flight** — pulled within seconds of issuance | `ev-history-sessions-read-live` (exists) |
| `disp-wiring-honesty-backend` | Nimbus | ready | none |
| `disp-wiring-honesty-ui` | Mira | blocked | `ev-wiring-honesty-backend-landed` (absent) |
| `disp-lineage-ui` | Mira | blocked | `ev-lineage-readmodel-live` (absent) |
| `disp-lineage-verify` | Flux | blocked | `ev-lineage-readmodel-landed` (absent); pre-declared verdict `ev-verdict-lineage-readmodel` |

Verified after enqueueing with `get_dispatch`: the trigger works end to end — the slice
whose edge already existed flipped to `in_flight` on its own, and the three that wait on
events nobody has emitted are `blocked`, not errored.

## The three measurements that changed the plan

**1. The corpus and the ledger share no key.** `archived_sessions` (021) carries
id/harness/project/cwd/timestamps/counts/paths and nothing else — no reference to
`sessions`, `topics`, `workitems` or `canonical_events`. `event_object_refs.ref_type`
already admits `session`, so the schema *allows* the link; nothing writes one. So the gap
between "seeing history" and "understanding why" is not another screen, it is a read model
that can honestly answer "no record" for a historical session — and must not guess a join
key, because 021 re-keyed 165 rows from the raw file name to the native session id.

**2. The provenance banner is wrong in the safe direction.** `provenance.ts:40`'s
`LIVE_TABS` omits the timeline, so ~600 real captured events render under "SAMPLE DATA —
not yet from live capture". A badge that under-claims is still a false statement, and it
teaches the operator not to trust the badge that is right.

**3. Clicking a timeline event opens nothing at all.** `SelectedObjectType`
(`types/index.ts:136`) has no `'Event'`; `TimelineView` selects one anyway; `hasSelection`
is false; `MasterDetail` renders the list full-width with no detail pane. The flagship
live surface has a dead primary action.

## The section Aegis asked for, and how I kept it honest

He asked me not to agree with his "one real gap" and not to invent a list. Plan §4 is
three parts: what I found broken (nine items, each with a file:line and a one-minute click
test), what I checked and did **not** find broken (five subsystems, so the list is not
"everything is on fire"), and what I cannot measure (which screen he clicked, whether the
running process is the commit I read, and the live tables' row counts beyond the ledger).

On his one gap specifically: the branch exists and is empty
(`supervisor_cmds.rs:76`, its own comment says so), but the user-visible failure is
different and worse. The frontend compensates by writing into a live PTY
(`SupervisorPanel.tsx:285`), and that map is only populated after the operator has
manually attached that seat's tmux session (`LiveSeatsView.tsx:194`). So a free-text
message to a seat that was never attached lands in the ledger and nowhere else, with no
error and no notice. Silent loss, not a missing branch — which is why slice 2 forces a
choice between routing it through the governed path and failing loudly.

## Blockers

- None for this turn. Two slices are moving, three are correctly blocked on referents
  that do not exist yet.
- The ⌘K ruling from the 2026-10-13 contract is still open, and this round sharpened it:
  one keypress opens two surfaces, one of them fabricated. No slice in this round touches
  the shortcut layer, so nothing blocks on the answer.
- One conflict I am flagging rather than deciding: the plain-language rule for acceptance
  criteria is all-Chinese, and `verify-docs.sh` §10 requires new docs to be English. Four
  of my own contracts on this branch already carry the same shape and the same red. The
  operator's readability is the newer and more specific instruction, so the plan follows
  it — but the gate is Aegis's to rule on.

## Artifact paths

- `docs/coordination/tasks/lyra/LYRA-2026-10-16-next-round-plan-v1.md`
- `docs/coordination/tasks/lyra/LYRA-2026-10-16-next-round-plan-delivery-v1.md` (this file)

## Ledger

- `ev-next-round-plan-landed` — `ArtifactCreated`, the plan announcement. Carries
  `branch`/`commit`/`files`, the milestone sentence, the five slices with their live
  statuses, `measured_on` (one commit plus the ledger; the app was not run), and an
  explicit `does_not_assert`. No `verified` field — it is an announcement, not a gate.
- `ev-gPEUa0OkdX7v7IKKnODRo` — `SupervisorMessage` to Aegis with the three answers, the
  broken-module section, the one-gap correction, the still-open ⌘K ruling, and the
  doc-gate conflict.
- The five `SeatDispatchIssued` events are written by `enqueue_dispatch` itself, in the
  same transaction as the rows.

---

# The three-section version for Mr. Zhang

**变了什么**
我把下一轮拆成了五张单,方向是「你在产品里点任何一样东西,能看懂它的来龙去脉」。同时我按你
说的「很多模块都是坏的」去核了一遍代码,没有只说「有一处问题」,也没有随手列一堆 —— 核到
九处,每处都写了在哪个文件、你怎么在一分钟之内自己验出来。其中几处你很可能真的点过:新建
工作项 / 新建交接 / 添加席位 / 委派这四个表单,填完、保存、刷新,东西就没了(数据库里根本
没有「新建工作项」这条命令);点时间线里的一条事件,界面什么反应都没有;还有 ⌘K 按一下会
同时弹出两个窗口,其中一个里的建议是写死的。我也写清楚了我核了之后**没坏**的部分:事件的
捕获、派单的调度、你说的那句话变成一条记在你名下的派单、历史语料的读取、验收门 —— 不是
到处都坏。

**你会看到什么**
这一轮先不动你看到的界面,先把「一件事的来龙去脉」这份数据做出来:点派单列表里的一行、点
时间线里的一条事件、点待验收里的一张卡,一屏大白话告诉你这是什么、谁做的、什么时候、产出
了什么、卡在谁、证据在哪。产品里真的没有记录的地方,它会说没有,而不是显示一片空白。排在
它前面的还有两件小事:把时间线头顶那块说反了的牌子改对(那 400 条事件其实是真的),以及让
发给席位的话不再无声无息地丢掉。

**需要你做什么**
两件。一,⌘K 到底归谁 —— 按一下同时弹两个面,其中一个是编的;这是你天天在用的键,我不擅自
移。二,我的验收标准按你的要求写了全中文,可仓库的文档门禁要求新文档用英文,这个分支上我已
有四份合同是同样情况、同样被标红 —— 人以你的规矩为准,但门禁不是我定的,请你裁守哪条。
