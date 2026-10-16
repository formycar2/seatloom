---
template: T5
subtype: product_contract
doc_id: LYRA-2026-10-16-next-round-plan-v1
title: Next-round iteration plan — "read the whole story of one piece of work inside the product", plus a measured list of which surfaces are actually live
status: active
author: lyra
doc_date: 2026-10-16
version: v1
depends_on: docs/coordination/ROADMAP.md (Aegis dispatch payload, packet), LYRA-2026-10-14-history-sessions-in-ui-contract-v1.md, LYRA-2026-10-13-supervisor-intent-entry-contract-v1.md
tags: iteration-plan, decomposition, provenance, lineage, milestone, gap-register
---

# Next-round iteration plan

**Owner:** Lyra (PO). **Measured on:** `track/infra-foundation` @ `d97ed6e` (the integration
branch in the shared worktree) plus the live ledger read through the MCP tools. **I did not run
the app and I did not touch the live database** — every claim below is either a file:line fact or
a ledger query, and the click-level checks are written as acceptance criteria for the operator,
not as claims I made myself.

The dispatch asked three questions and one judgment. §2 answers the questions, §3 states the
milestone, §4 is the decomposition with event-id edges and plain-language acceptance criteria,
§5 is the "which modules might be broken" section Aegis asked for — written so it neither
agrees with him nor invents a list.

---

## 1. Verified current state (measured, not reported)

| # | Claim in the dispatch | Measurement | Verdict |
|---|---|---|---|
| C1 | Capture spine complete: events emitted, governed dispatch, first real event all live | Ledger holds ~600 events; `SeatDispatchIssued/Delivered/Completed` for this very dispatch; `emit_event` works from every seat's tool process | ✅ |
| C2 | Knowledge panel merged into track (Mira) | `8a03ee1` merged `packet/mira/knowledge-panel`; `KnowledgeSection.tsx` mounted at `ProjectDashboard.tsx:810` | ✅ merged — but see §2.2 row 6 for where it is reachable from |
| C3 | Supervisor intent entry goes the real road (`plan_intent` → `enqueue_dispatch`) | `cmd_plan_intent` registered (`src-tauri/src/main.rs:173`); `cmd_enqueue_dispatch` now forwards `dependencies` verbatim (`dispatch_cmds.rs`, the V5 fix) | ✅ real, not a stub |
| C4 | Historical corpus data is on live (183 / 36,680 / 38) | Gate `ev-history-sessions-read-live` (2026-10-16 05:04) records a measured call against the live database; `list_history_projects` served by every seat's process | ✅ |
| C5 | Mira is doing three screens, in flight, don't duplicate | `disp-history-sessions-ui` is `in_flight`, one edge `ev-history-sessions-read-live` (exists) | ✅ untouched by this plan |
| C6 | Aegis found exactly one real gap: the old free-text injection branch in the supervisor path | `src-tauri/src/commands/supervisor_cmds.rs:76` — `// Leaving this branch as an integration hook for later.` The branch exists and does nothing | ⚠️ confirmed, but the *user-visible* failure is different and worse — see §5.1 item 6 |
| C7 | *(not in the dispatch)* | **There is no Tauri command for the historical corpus.** `src-tauri/src/commands/` has 16 files and none of them is a history command; the corpus is reachable only through the three MCP read tools. Mira's slice 3 is what puts it on a screen. | new finding |
| C8 | *(not in the dispatch)* | **`archived_sessions` is an island.** `HistorySession` (`crates/seatloom-core/src/history/schema.rs:51`) carries id/harness/project/cwd/timestamps/counts/paths — no reference to `sessions`, `topics`, `workitems` or `canonical_events`. Nothing in the product joins a historical session to anything the ledger recorded. | new finding — this is the answer to question 1 |
| C9 | *(not in the dispatch)* | **Clicking an event in the timeline opens nothing.** `SelectedObjectType` (`ui/src/types/index.ts:136`) is `'Seat' \| 'Session' \| 'WorkItem' \| 'Artifact' \| 'Handoff'` — there is no `'Event'`, so `TimelineView`'s `onSelectObject('Event', …)` produces a selection whose detail pane is never rendered (`App.tsx` → `MasterDetail`, `hasSelection` false → list full width, no detail). | new finding — see §5.1 item 2 |
| C10 | *(not in the dispatch)* | **The timeline's provenance badge is false in the safe direction.** `ui/src/lib/provenance.ts:40` — `LIVE_TABS = {live-seats, sessions, dispatches}`; the timeline is not in it, so ~600 real captured events render under "SAMPLE DATA — not yet from live capture". | new finding |
| C11 | *(state change while this plan was being written, 2026-10-16 05:48)* | Mira's corpus slice **delivered**: `HistorySessionsPanel.tsx` + `src-tauri/src/commands/history_cmds.rs` on `packet/mira/knowledge-panel` @ `51bccc5`, with a measured read path (38 / 183 / 36,680 / OfficeBench 49 / seatloom 14) and an acceptance card requested. **Not merged to track** — so C7 stays true of `track/infra-foundation`, and the lineage pane must not assume the screens exist. `disp-lineage-ui` requires a tree measurement at pull time for exactly this reason. | state change, recorded |

---

## 2. The three questions

### 2.1 Question 1 — after the corpus screens land, what is still missing between "seeing history" and "understanding why one session did what it did"

What he will be able to do once Mira's slice 3 **merges** (it delivered on
`packet/mira/knowledge-panel` @ `51bccc5` at 05:48, still unmerged as this plan is written):
pick a project (a home path), pick one of its 183 sessions, read that session's turns in source
order, each with the line number it came from in the original file. That is **seeing** history,
and it is genuinely new.

What he still will not be able to do, measured:

1. **There is no bridge from a session to the record of work.** The corpus (`archived_sessions`,
   021) and the ledger (`canonical_events` + `event_object_refs`) are two separate stores with no
   shared key. `event_object_refs.ref_type` already admits `session`, so the schema *allows* a
   link; nothing writes one for an archived session, and no archived session id is ever an event
   referent. So "what came out of that session" has no answer inside the product today.
2. **The ledger answers "what happened", never "why".** It records that a dispatch was issued, a
   verdict was given, an artifact was created — never the reasoning that produced them. The
   reasoning lives in the session transcripts (the corpus) and in the washing pipeline's output
   (topics + memories). Nothing joins the two halves for the same unit of work.
3. **The join cannot be faked.** 021 re-keyed the ids (165 rows carried the raw file name, not
   the session id), and the read path deliberately indexes by the *declared* source path rather
   than a derived filename — because a derived filename is a guess. Any "link a session to its
   events" feature that guesses a key would be wrong in exactly the way this project has already
   paid for once.
4. **There is no replay surface at all.** The timeline lists events; clicking one opens nothing
   (C9). The dispatch inbox shows dependency counts and the raw strings `event:ev-…`
   (`DispatchInboxView.tsx:439`) — the operator can see *that* something is blocked on
   `ev-history-sessions-read-live` but cannot see *what that event is* without leaving the product.

So the missing piece is **not another screen**. It is one read model that, given any object the
operator can point at — a historical session, a live session, a dispatch, an event, an acceptance
card — returns the ordered facts about it (what it was, who did it, when, what it produced, what
it was gated on, where the evidence is) **and says "no record" honestly where no record exists**.
That is slice 1 of this round. The corpus screens are the "see" half; this is the "understand"
half; they meet in one detail pane.

### 2.2 Question 2 — which surfaces are really live and which only look like it, per surface

The judgment criterion is the same for every row: **what function does the code call, and does
that function read the captured ledger / a live tmux pane, or does it read a fixture, or does it
write nowhere at all?** The provenance banner is a fourth thing — it is a hand-written
declaration, and §5.1 item 3 shows it can be wrong.

| Surface | What the code actually calls | Verdict | How the operator checks it in one minute |
|---|---|---|---|
| 实时席位 (live mirror) | `api.listTmuxSessions` + `attachTmuxSession` + `pty_*` (`LiveSeatsView.tsx`) | **Really live.** The only surface whose LIVE badge is gated on a measurement (discovery status) | Open a seat's pane, type, watch it appear in the real tmux session. Stop the seats, press 重新发现 — the badge must flip to NO LIVE SEATS |
| 会话 (sessions tab) | `api.listTmuxSessions` / `api.attachTmuxSession` (`SessionsWorkspace.tsx`) | **Really live** (same tmux path) | Same as above |
| 派单 (dispatch inbox) | `api.listDispatches` → `cmd_list_dispatches` → the `dispatches` table; `api.enqueueDispatch` writes a real dispatch through the MCP binary | **Really live, and the only surface that writes** | Issue a dispatch to a seat from the UI; the seat pulls it; both rows and events exist afterwards |
| 待验收 (acceptance, inside 派单) | `api.listAcceptances` / `acceptDelivery` / `rejectDelivery` → the `acceptances` table + events | **Really live, and it writes** (merge happens on the operator's click) | Accept one card; the merge runs |
| 时间线 (timeline) | `api.listEvents(400)` + an 8-second poll (`useDataStore.refreshEvents`) | **The data is really live; the label is false.** See §5.1 item 3 | Count the rows against `query_ledger` — they match. Then read the badge, which says SAMPLE |
| 知识 (knowledge panel) | `invoke('cmd_get_knowledge_tree')` / `cmd_get_topic_memories` (`KnowledgeSection.tsx`) → the topics + memories tables | **Really live data**, but (a) reachable only inside the ⌘K panel's project dashboard, with no NavRail entry, and (b) gated on a hand-written constant, `KNOWLEDGE_CMDS_REGISTERED = true` (`KnowledgeSection.tsx:105`) | Open ⌘K → project view → scroll past the goals/stages sections to the knowledge section. Then ask: what measured that `true`? |
| 产出物 (artifacts) | `documentsDto` (written by `cmd_reconcile` scanning the repo on disk) + `artifactsDto` (7 seed rows) | **Half real** — it is the reconcile scanner's view of files on disk, not the capture spine | The bulk of the rows are coordination documents found by scanning; the 7 runtime artifacts are the seed file's |
| 监督概览 / 工作项 / 席位 / 收件箱 / Playbook / 全部项目 | `useDataStore`, hydrated from `cmd_list_projects/seats/workitems/handoffs/sessions/events` | **Seed, and nearly empty.** The live database's workitems/sessions/handoffs/seat_delegations rows are the four seed files' fixtures: 1 workitem, 1 session, 1 handoff, 1 seat, 7 artifacts. Hydration *replaces* the mock with those rows | Open 工作项 and count the rows. The badge says SAMPLE, which is correct — the surface is honest, it just has nothing captured in it yet |
| IM panel's project dashboard | `truthData` = live project data (blockers / active work / events) **plus** `MOCK_CHANNEL_DATA[channelId]` (`ProjectDashboard.tsx:52`) for goals and stages | **Mixed.** The mock keys are `ch-p1/p2/p3`; the only live project id is `seatloom`, so `MOCK_CHANNEL_DATA['ch-seatloom']` is `undefined` and the goals/stages sections render **empty** — dead mock code, not fabricated content, but two sections that can never fill | Open ⌘K → project dashboard: the goals and stages bands are blank no matter what the ledger holds |
| IM contact roster | `useContacts` derives from live projects + seats | **Real names, invented presence.** `online` comes from `seat.status`, but `unread` is hard-coded 0 and `lastMessage`/`lastTime` are `undefined` | Compare the roster against the seats table |
| 新建工作项 / 新建交接 / 添加席位 / 委派 | **Nothing.** `WorkItemForm.tsx:57` calls `addWorkItem` (a Zustand store action); `HandoffForm`, `AddSeatDialog` and `DelegationOverlay` contain zero `api.*` calls. `src-tauri/src/main.rs:116-195` registers **no** create/update command for any of these objects | **Write-only illusions.** The form accepts input, shows success, and the data is gone on reload — it never reached a database | Create a work item, press reload, look for it. This is the single most likely thing the operator clicked when he said "很多模块都是坏的" |
| V1 command bar (⌘K) | `SupervisorCommandBar.tsx` — `generateProposal()` is a `setTimeout` returning a hardcoded proposal; `handleConfirm` writes to the store | **Fabricated**, and ⌘K is still double-bound (§5.1 item 4) | Press ⌘K: two surfaces open at once, one of them showing a proposal that was never computed |

### 2.3 Question 3 — the user-perceivable milestone for this round

Per the roadmap's governing principle, the round must ship one thing he can see or do.

> **Milestone: 点任何一样东西,看懂它的来龙去脉。** He clicks a row — a dispatch, a timeline
> event, an acceptance card, and once Mira's corpus screens land, a historical session — and one
> screen answers, in plain language: 这是什么 / 谁做的 / 什么时候 / 产出了什么 / 卡在谁 /
> 证据在哪. Where the product has no record, it says so instead of showing nothing.

Today the same click either opens nothing (timeline) or shows a raw `event:ev-…` string
(派单). The plumbing beneath it — the lineage read model, the measured wiring probe, the honest
labels — are sub-tasks inside the milestone, never milestones themselves.

---

## 3. The decomposition, with edges as event ids

| # | Slice | Seat | Dispatch | Edges (all `dep_kind=event`) |
|---|---|---|---|---|
| 0 | This plan | Lyra | `disp-vEU7q7-asGPAhOmgNAxgK` | — → emits `ev-next-round-plan-landed` |
| 1 | Lineage read model (read-only backend) | Nimbus | **`disp-lineage-readmodel`** (new) | `ev-history-sessions-read-live` (exists → ready) |
| 2 | Wiring honesty, backend | Nimbus | **`disp-wiring-honesty-backend`** (new) | none → ready |
| 3 | Wiring honesty, UI | Mira | **`disp-wiring-honesty-ui`** (new) | `ev-wiring-honesty-backend-landed` → blocked |
| 4 | The lineage detail pane | Mira | **`disp-lineage-ui`** (new) | `ev-lineage-readmodel-live` → blocked |
| 5 | Verify the lineage read model | Flux | **`disp-lineage-verify`** (new) | `ev-lineage-readmodel-landed` → blocked; pre-declared verdict `ev-verdict-lineage-readmodel` |

**Why these edges.** Slice 1 reads the historical corpus, so it waits for the measured live gate
that already exists. Slice 4 *calls* the lineage capability, so it waits for the measured
`-live` gate, not the announcement — the same rule Mira's own slice 3 was issued under. Slice 3
consumes a probe that slice 2 adds to `ui/src/lib/api.ts`, which is inside Nimbus's file fence, so
it cannot be merged in the same breath; the edge is the only honest way to order them.

**Why slice 4 carries no edge on Mira's corpus screens.** Mira holds exactly one in-flight
dispatch — that is a database fact, not a convention — so `disp-lineage-ui` cannot be pulled
before `disp-history-sessions-ui` is completed. No edge is needed, and adding one on a dispatch id
would be wrong: a dispatch row *exists* the moment it is issued, so such an edge would read as
satisfied immediately. The payload instead requires her to measure the tree at pull time and
report if the corpus screens are not there.

### 3.1 The event contract — exact ids and required payloads

| Event id | Emitter | Type | Required payload |
|---|---|---|---|
| `ev-next-round-plan-landed` | Lyra | `ArtifactCreated` | `contract`: this file's path + commit. Announcement; no `verified` field. |
| `ev-lineage-readmodel-landed` | Nimbus | `ArtifactCreated` | `branch`, `commit`, `files`, and the exact shape of `lineage(ref)`. Announcement; no `verified` field. |
| `ev-lineage-readmodel-live` | Nimbus | `ArtifactCreated` | **`verified`** (mandatory): which call was made, exactly what it returned, on which database, when. A gate; only after a real call succeeded against the live database. |
| `ev-wiring-honesty-backend-landed` | Nimbus | `ArtifactCreated` | `branch`, `commit`, `files`, and which hand-written constant was replaced by which measured probe. |
| `ev-verdict-lineage-readmodel` | Flux | `ReviewVerdictIssued` | Pre-declared at enqueue time; Flux must emit exactly this id. |
| `ev-wiring-honesty-ui-landed` | Mira | `ArtifactCreated` | `branch`, `commit`, `files`, before/after for each label and each dead control. |
| `ev-lineage-ui-landed` | Mira | `ArtifactCreated` | `branch`, `commit`, `files`, and which click targets now open the pane. |

### 3.2 Acceptance criteria, per slice — in plain language, for the operator

**派单 1(Nimbus)· 族谱读取模型。**
- 做一个只读函数:给它一个东西(历史会话 / 实时会话 / 一条派单 / 一条事件 / 一张验收卡),它按时间顺序返回关于这个东西的一串事实:这是什么、谁做的、什么时候、产出了什么、被什么卡着、证据在哪。
- 同一个函数只许有一份代码,开两个口子给人用:一个给 MCP 工具,一个给界面命令。不许写第二份实现。
- **历史会话那一半,没有记录就说没有记录。** 现在的库里,历史会话和账本之间没有任何关联键(这一点我核过,见 C8)。所以给一个历史会话,它必须回答"这个会话在账本里没有任何记录",而不是猜一个关联、也不是返回一个空列表让人以为"什么也没发生"。要搭桥就得先有关联键,那是以后的事,这一轮不许假装有。
- 给的东西不存在时,说清"不存在",不许白屏、不许永远转圈、不许空列表。
- 不许写库、不许发门事件、不许改界面。
- 门:先发落地声明,再在真库上真调一次通过之后才发 `ev-lineage-readmodel-live`。调用的原话和返回值要写进事件里。

**派单 2(Nimbus)· 后端不再谎报已接线。**
- 把 `ui/src/lib/api.ts:67` 那个手写的 `ACCEPTANCE_CMDS_REGISTERED = true` 换成一个**实测**的探针:界面问一句"这条命令在不在这个构建里",它真的去问,然后把答案返回。探针本身也要能测。
- 把监督自由文本消息那条老路修掉:要么让它走受治理的派单(`enqueue_dispatch`),要么在发不出去的时候**明说没送达**。现在的行为是静默丢掉(见 §5.1 第 6 条),这是最坏的一种。
- 不许顺手改 `api.ts` 里别的东西;这个文件很多席位在用,只增不改。

**派单 3(Mira)· 界面不再说谎。** 依赖派单 2 的探针落地后才做。
- 时间线头顶那块"SAMPLE DATA — not yet from live capture"的牌子是错的:那 400 条事件是真从库里读出来的。改对,并且说清它什么时候会变成真(哪条链路通了才算)。
- 点时间线里的一条事件,现在什么都不会发生。修到:点了就打开派单 4 做的那一屏。
- `KnowledgeSection.tsx:105` 那个手写的 `KNOWLEDGE_CMDS_REGISTERED = true` 换成派单 2 的实测探针。不许再加第三个手写常量。
- 四个"假写入"的表单(新建工作项 / 新建交接 / 添加席位 / 委派)不许再假装存进去了。二选一:要么接到真命令(接不上就说明缺哪条命令,上报等派单),要么在保存时就告诉人"这个版本还存不进数据库,你填的东西关了就没"。不许留中间态。
- 界面骨架不许重排(他明确要求过)。检查按文件粒度,本来就脏的文件保持原样脏,交一份"零新增问题"的实测结果。

**派单 4(Mira)· 来龙去脉那一屏。** 依赖派单 1 的实测门通过后才做。
- 在现有骨架里开一屏详情:点派单列表里的一行、点时间线里的一条事件、点待验收里的一张卡,都开到这一屏;派单 1 的族谱数据是唯一数据源。
- 派单列表"依赖"那一列现在显示的是 `event:ev-…` 这种原始字符串。改成:这是什么事件、它的结论是什么、它在账本里存不存在。存不存在要真的去问,不许手写。
- 历史会话那一屏,等语料画面合进来之后,从会话详情能点到族谱;这一轮先把数据和界面准备好,没合进来就不显示这个入口,不许放一个点了没反应的按钮。
- 验收卡按人话规矩写:全中文、不出现自造缩写、每步写清看什么、什么算对、什么算打回。他为这个打回过一次。

**派单 5(Flux)· 验证族谱读取模型。** 依赖派单 1 的落地声明。
- 对抗性验证:专门挑"会猜"的地方打。至少覆盖:给一个不存在的东西;给一个历史会话(必须回答没有记录,而不是编一个);给一条被阻塞的派单(依赖的事件不存在时,必须说清缺什么);给一个空的题。
- 每一条都要报出确切的输入和输出。一个"通常会拒绝"的解析器算不通过。
- 裁决事件必须用预声明的 id `ev-verdict-lineage-readmodel`。

---

## 4. Which modules might be broken, where, and how to measure it

Aegis asked me not to agree with him and not to invent a list. This section is only what I could
verify in code on `track/infra-foundation` @ `d97ed6e`, plus the ledger. I have **not** run the
app, so these are code-verified, not click-verified — the click test is the acceptance criteria,
and it belongs to the operator.

### 4.1 What I found broken

1. **四个表单是假写入。** `ui/src/components/WorkItemForm.tsx:57` 只调 `addWorkItem`(一个前端
   store 动作);`HandoffForm.tsx`、`AddSeatDialog.tsx`、`DelegationOverlay.tsx` 里一个 `api.*`
   调用都没有;`src-tauri/src/main.rs:116-195` 注册的命令里没有任何一条能建工作项、建交接、
   加席位、发委派。**怎么量:** 新建一个工作项 → 刷新界面 → 找它。不在,就是这条。
2. **点时间线里的事件没有反应。** `ui/src/types/index.ts:136` 的 `SelectedObjectType` 里没有
   `'Event'`,`TimelineView` 却按 `'Event'` 去选;`App.tsx` 的 `hasSelection` 于是为假,
   `MasterDetail` 只渲染列表、不渲染详情面板。**怎么量:** 点一条事件,看右侧出不出来东西。
3. **时间线的真假牌子是反的。** `ui/src/lib/provenance.ts:40` 的 `LIVE_TABS` 只有
   `live-seats / sessions / dispatches`,时间线不在里面,于是 400 条真事件头顶挂着"SAMPLE DATA
   — not yet from live capture"。**怎么量:** 拿界面里的行数和 `query_ledger` 的行数对;再读那块牌子。
4. **⌘K 同时绑了两处,而且三份文档三种说法。** `ui/src/App.tsx:139` 的那个监听器按
   `!e.shiftKey` 打开监督面板;`ui/src/hooks/useGlobalShortcuts.ts:24` 的那个监听器没有 shift
   判断,直接打开 V1 命令栏。两个都是 window 级监听,都没调 `stopImmediatePropagation`,所以
   **按一下 ⌘K,两样东西同时弹出来**。而 `App.tsx:133-135` 的注释说 V1 栏在 ⌘⇧K 上 —— 代码并不
   这么干;`ShortcutHelpDialog.tsx:11` 又告诉用户这个组合键打开命令栏。三份文档三种说法,代码做
   第四种。**怎么量:** 按一下 ⌘K,数弹出来几个窗口。
5. **监督面板里的目标/阶段两节永远是空的。** `ui/src/app-v2/dashboard/ProjectDashboard.tsx:52`
   读 `MOCK_CHANNEL_DATA[channelId]`,而 mock 的键只有 `ch-p1/p2/p3`,真项目的 id 是
   `seatloom`。所以这两节无论账本里有什么都渲染空。**怎么量:** 打开 ⌘K 的项目视图,看目标和
   阶段两条带是不是白的。
6. **发给席位的自由文本消息会静默丢失。** `src-tauri/src/commands/supervisor_cmds.rs:76` 的
   注入分支是个空壳(注释原话:"Leaving this branch as an integration hook for later")。前端
   `ui/src/app-v2/panel/SupervisorPanel.tsx:285` 有个补偿:能从
   `useLiveSessionsStore.sessionFor(席位名)` 拿到会话 id 就往 PTY 里写。但那个映射只在用户
   **手动 attach 过**某个席位的 tmux 会话时才有值(`LiveSeatsView.tsx:194` 才填它)。所以:
   没先 attach 过就发消息 → 只落一条账本事件,席位什么都收不到,**而且不报错、不提示**。
   **怎么量:** 打开 ⌘K,直接给一个席位发一句话,不去点它的终端。看那句话有没有进到那个席位的
   回合里。这就是 Aegis 找到的那处缺口的真实样子:不是分支不存在,是它依赖一个用户不一定做过
   的动作,而且失败是无声的。
7. **派单的"依赖"列给人看的是机器码。** `ui/src/views/DispatchInboxView.tsx:439` 直接把
   `dep_kind:dep_value` 拼成 `event:ev-history-sessions-read-live` 这样的字符串给人看。运营者
   看得见"被什么卡住",看不出"那是什么、结论是什么"。**怎么量:** 打开派单页,看依赖列。
8. **知识面板没有入口,而且靠一个手写常量开门。** `KnowledgeSection` 只挂在
   `ProjectDashboard.tsx:810`(在 ⌘K 面板的项目视图里),主导航没有它;而它开不开取决于
   `KnowledgeSection.tsx:105` 那个手写的 `true`,没有任何东西测过这个 `true`。
   **怎么量:** 在主导航里找"知识",找不到;再问一句:是什么测出那个 `true` 的?
9. **派单/工作项/席位/产出物这几张表里只有种子数据。** 四个种子文件一共灌了 1 个工作项、
   1 个会话、1 条交接、1 个席位、7 个产出物。仪表盘和收件箱因此几乎是空的 —— 这一条**不是
   坏**,是徽标说对了的那种空;但它意味着"监督概览"这张脸今天给不出任何真东西。

### 4.2 What I checked and did not find broken

So the list above is not "everything is broken" either:

- **捕获脊柱本身。** 发事件 → 进账本 → 同事务写投影 → 能按主体重建。这一轮我拿到的派单,从
  `SeatDispatchIssued` 到 `SeatDispatchDelivered` 都在账本里,一条不少。
- **受治理派单。** 依赖边是真的数据:这一轮我要挂的边,挂不存在的东西会被判 `blocked` 而不是
  报错,挂已存在的门会被放开。 ready/blocked 的判定是算出来的,不是写死的。
- **监督意图入口。** `cmd_plan_intent` 注册在 `main.rs:173`,`cmd_enqueue_dispatch` 现在把
  `dependencies` 原样转发(`dispatch_cmds.rs` 的 V5 修复)—— 一句话变成一条记在人名下的派单,
  这条路是真的,不是样板。
- **历史语料读取链路。** `ev-history-sessions-read-live` 那道门后面是一次对真库的真调用,
  38 个项目 / 183 个会话 / 36680 条消息,和磁盘上的源文件逐项对过。
- **验收门。** 待验收那一节接的是真的 `acceptances` 表,点接受才合并,席位自己不能合自己的活。

### 4.3 What I cannot measure

他在用产品,我量不到他看到的。具体说,我量不到这三类,只能把测量方法写成交付物让他点:

- **他看到的是哪一屏、点的哪一个。** 上面的清单是按代码判据排的,不是按他点过的频率排的。
  如果他为这个打过回,他最可能点过的是 §4.1 第 1 条(假写入)和第 6 条(静默丢失)—— 但我
  没有证据,这只是推断,所以我把它写成了"怎么量",没有写成结论。
- **跑起来的应用和代码是不是一回事。** 我读的是 `track/infra-foundation` @ `d97ed6e` 的文件。
  运行中的那个进程是不是这个提交,我没有验(验它要重启,那是 Aegis 的边界)。
- **live 库里那些表到底有几行。** 种子文件我能数,但 live 库里的行数除了账本我一概没查 ——
  按边界,不动 live 库。所以 §4.1 第 9 条的行数是种子文件的数,不是 live 的数。

---

## 5. Out of scope this round, with reasons

- **重做界面骨架。** Forbidden by the operator, restated.
- **Touching the live database, emitting gate events, merging.** Aegis's boundaries, restated.
- **Making `archived_sessions` and the ledger share a key.** It is the obvious next step after
  the lineage read model exists, and it is a schema decision (Onyx's slot) with a real trap in it
  (021's re-keying). It gets its own contract, not a rider on this one.
- **Nesting the 38 history projects into a tree.** Already deferred in the 2026-10-14 contract;
  the home paths are not a clean tree and any rule is a guess.
- **A new top-level navigation entry for the knowledge panel.** It is a real discoverability
  defect, but it is a navigation decision, and navigation is the operator's. Raised, not decided.

## 6. The one thing still waiting on a human

The ⌘K ruling from the 2026-10-13 contract is still open, and this round makes it sharper: the
chord does not just open a prototype, it opens **two surfaces at once**, one of them fabricated.
I placed nothing in this plan on the shortcut layer — slice 3 is forbidden from touching it — so
the round does not block on the answer. But §4.1 item 4 is a live defect the operator hits every
time he presses the key he uses most, and I am not moving a shortcut he uses daily on my own
authority.
