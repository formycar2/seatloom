# SeatLoom Roadmap — the single living plan

| Field | Value |
|---|---|
| template | T7 |
| subtype | governance |
| id | ROADMAP |
| status | living — updated as state changes |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| tags | roadmap, plan, status, gap-register, p0 |

## 0. How to read this

This is the **single entry point** to what SeatLoom is building and where it stands.
It is a dashboard, not a design doc — the "why/how" lives in the deep design reviews,
linked from each phase. When status changes, **this file is updated** (by Aegis), so a
glance here answers "what's done, what's in flight, what's blocked, on whom."

Deep references (unchanged, authoritative for detail):
`reviews/2026-09-29-aegis-platform-design-macro-to-micro-v1.md` (abstraction A1–A7),
`reviews/2026-09-30-aegis-p0-scope-takeover-and-capture-v1.md` (P0 thesis),
`reviews/2026-09-30-aegis-data-flow-and-layering-v1.md` (capture tiers T1–T3),
`reviews/2026-09-30-aegis-object-model-and-traceability-v1.md` (object hierarchy, topic),
`reviews/2026-09-30-aegis-p0-build-plan-and-gap-register-v1.md` (P0 build detail; its §1
gap register is now superseded by §4 below as the living list).

## 1. Current status — the live view (2026-09-30)

| Layer | State |
|---|---|
| Workbench (CI, gates, 6 seats) | **DONE** — hard gate green on GitLab `track` |
| runtime-triple (harness/agent/model split) | **DONE** — reconciled to GitLab |
| **P0 step 1 — schema foundation (010)** | **DONE** — Flux PASS, merged, Lyra accepted, reconciled to GitLab (`231d0a5`) |
| **P0 step 2 — write API (A1)** | **DONE** — Flux PASS, merged, Lyra accepted, reconciled to GitLab (`b691ca4`). `append(event)→projections`, projections rebuildable from ledger |
| **A — live mirror (UM0)** | **DELIVERED, BLOCKED** — Mira's UI proven correct; blocked by a backend bug: `pty/mod.rs` uses `tmux pipe-pane -o` (a toggle), so re-attach kills the mirror. Fix dispatched to Nimbus (drop `-o`). Reliable UM0 = A + fix both merged, then Aegis restarts the shared UI |
| P0 steps 4 (MCP), 6 (dispatch), 5 (first event) | **QUEUED** |
| P1 / P2 | not started |

**Blocked-on-whom right now:** UM0 waits on Nimbus (pipe-pane `-o` re-attach fix); A (Mira) is
delivered and in Flux's queue. No decision is blocked on Mr. Zhang or Aegis.

## 2. Phases

> **Governing principle (Mr. Zhang, 2026-09-30):** every milestone ships a
> **user-perceivable capability**. Pure-plumbing steps (schema, write API, MCP) are
> sub-tasks *inside* a milestone, never milestones themselves. Every milestone's
> acceptance carries a "the user can now see / do X" clause. §2b is the acceptance lens;
> §3's engineering steps are the scaffolding beneath it.

| Phase | What it is | Solves | Done when | Status |
|---|---|---|---|---|
| **0. Workbench** | CI, quality gates, 6 seats on one harness | collaboration safety | hard gate green, red=no-merge enforced | ✅ done |
| **R. runtime-triple** | split `Runtime` into harness/agent/model | harness-agnostic identity groundwork | both sides on GitLab track, CI green | ✅ done |
| **P0. Capture loop** | take over the live agent loop; record it into one harness-agnostic data model | "the record, not the harness, holds the truth" | one real event exists because an agent did something, filed under project+topic, linked, retrievable, harness-independent (see §3) | 🔵 in progress (step 1) |
| **P1. Compounding** | richer capture + capability assets | reasoning/search/RAG traces; published Playbook/Skill/Tool; auto topic-segmentation | (scoped when P0 lands) | ⏳ queued |
| **P2. History & cost** | ingest the ~16 GB corpus; budget/token accounting | findability of past work; cost discipline | (scoped later) | ⏳ deferred |

## 2b. User-perceivable milestones (the acceptance lens)

Each rung is something the user can **see or do** that they could not before. Engineering
steps (§3) slot beneath a rung; a rung is "done" only when the capability is perceivable.

| Milestone | What the user can now do (couldn't before) | Backed by | Status |
|---|---|---|---|
| **UM0 — one screen for the whole team** | open SeatLoom, watch all 6 seats' live terminals + IM in one window, instead of attaching 6 tmux sessions | existing tmux mirror + viewer | **available now — not yet switched on** |
| **UM1 — work leaves a clickable trace** | when a seat produces an artifact, a structured, clickable record appears in the timeline (filed under project + commit), not just terminal scroll | step 1 ✅ + steps 2, 4, 5 | step 1 done |
| **UM2 — progress at a glance** | dispatches / handoffs / verdicts render as a live work-graph: who's doing what, who's blocked on whom, what's queued — the UI answers "how is progress?" at a glance | step 6 governed dispatch + capture generalized | queued |
| **UM3 — find & replay** | "show me everything about topic X across sessions/harnesses"; deterministic search + T1→T2→T3 zoomable replay | P1 | not started |
| **UM4 — capability library** | see and reuse the team's published Playbooks / Skills / Tools | P1 | not started |
| **UM5 — history & cost** | the ~16 GB of past work is findable; per-project token spend is visible | P2 | deferred |

Interim reality until UM2: this ROADMAP (a markdown file) + Aegis are the "progress view";
the destination is the SeatLoom UI rendering it from the captured event model.

## 3. P0 build — the six steps

P0 delivers **one true thing that does not exist today**: a record in SeatLoom because an
agent, on some harness, did something — filed under its project and topic, linked to what it
touched, retrievable, and independent of which harness produced it.

| Step | Work | Owner | Done when | Status |
|---|---|---|---|---|
| 1 | **Schema foundation** — event envelope (G2), ref roles (G3), G5 verify, topics (G6), turn key (G12) | Onyx | migration idempotent; value-domain contract enforced; seed still loads | 🔵 vocab edit done, awaiting Flux verify |
| 2 | **Write API (A1/G1)** — `append(event)→projections`, single txn | Nimbus | a projection rebuildable from `canonical_events` alone | ⏳ queued |
| 3 | **`external_ref` (G4)** — pin seat sessions, harness-agnostic identity | Onyx+Nimbus | a session resolves to its object regardless of harness | ⏳ queued |
| 4 | **MCP emit server (G7)** — `emit_event(type, payload, refs)`, schema-validated | Nimbus | a seat's `--mcp-config` points at it; malformed event rejected | ⏳ queued |
| **6** | **Governed dispatch (G19)** — per-seat inbox + explicit `depends_on` + ready-aware scheduler (one-in-flight, pull next *ready* item, blocked items don't head-of-line-block) + cycle/stall detection → escalate | Nimbus | a seat receives the next message only when idle and its deps are satisfied; a cross-seat dependency cycle is detected and surfaced to Aegis, not hung; delivery recorded as events; no mid-turn injection | ⏳ **elevated to P0** (2026-09-30, Mr. Zhang) |
| 5 | **First slice** — one seat emits one real `ArtifactCreated` (`commit`+`digest`), lands via write API, links to workitem+topic+project, retrievable | Nimbus + Flux verify | a human queries the event and traverses to the artifact and back | ⏳ queued (acceptance of P0) |

Critical path: 1 → 2 → 4 → 6 → 3 → 5 (step 6 rides on 2+4; step 5 is the acceptance).

## 4. Gap register (living — supersedes build-plan §1)

| # | Gap | Priority | Status |
|---|---|---|---|
| G1 | Event as sole write primitive (A1) | P0 (step 2) | queued |
| G2 | Typed + versioned event envelope (A2) | P0 (step 1) | 🔵 in 010 |
| G3 | `event_object_refs.role` (A3) | P0 (step 1) | 🔵 in 010 |
| G4 | `external_ref` (A4) | P0 (step 3) | queued |
| G5 | `sessions.project_id` | P0 — **verify only** (008 added it) | 🔵 confirmed in 010 |
| G6 | Topic layer (table + event→topic ref) | P0 reserve (step 1) / P1 auto-segment | 🔵 in 010 |
| G7 | MCP emit server | P0 (step 4) | queued |
| G8 | File events reference git `(commit, digest)` | P0 (step 5) | queued |
| **G19** | **Governed dispatch** — per-seat inbox + explicit `depends_on` + ready-aware scheduler + cycle/stall detection→escalate (one-in-flight kills context pollution; dependency-awareness prevents head-of-line blocking; only a true cross-seat cycle deadlocks, and SeatLoom detects it because it holds the dispatch graph as data) | **P0 (step 6)** — elevated; auto-resolution/preemption → P1 | queued |
| G9 | Dual-authority resolution (A5) | P1 | deferred |
| G10 | Typing de-duplication (A6) | P1 | deferred |
| G11 | Capability asset (Playbook/Skill/Tool), published+versioned | P1 | deferred |
| G12 | T2 process traces; reserve `(session_id, turn_seq)` now | P0 reserve (step 1) / P1 capture | 🔵 key in 010 |
| G13 | Recording shell / terminal capture | P1 | deferred |
| G14 | Standing commitment / recurring task | P1 | deferred |
| G15 | SUT-version dimension for test results | P1 | deferred |
| G16 | Budget as first-class object (A7) | P2 | deferred |
| G17 | Historical corpus ingestion (~16 GB) | P2 | deferred |
| G18 | Interposition proxies / egress gateway | P2 | deferred |

## 4b. Dev-velocity TODO (iteration-speed optimizations)

Running list — any optimization that speeds the build/test/coordinate loop is logged here as it
is observed (Mr. Zhang, 2026-09-30). These are dev-ergonomics, not product gaps; none blocks a
milestone, each pays back time on every cycle.

| # | Optimization | Bottleneck it kills | Effort | Status |
|---|---|---|---|---|
| DV1 | **Shared compile cache** — `sccache` (`RUSTC_WRAPPER`, per-crate cache, concurrency-safe) across all worktrees + CI; or `CARGO_TARGET_DIR` (simpler, but serializes concurrent builds on a target lock) | each worktree cold-compiles ~450 deps from zero; repeated per worktree and per CI pipeline | S | **proven recipe (no infra): `cp -Rc <shared>/target ./target` in a fresh worktree — APFS copy-on-write, 18s + ~0 disk; then `tauri dev` rebuilt 2/452 and ran in 36s. Verify backend identical first (`git diff --stat`). sccache/CI still todo.** |
| DV2 | **Local pre-push gate dry-run** — `scripts/preflight.sh` = `cargo fmt --check` + `clippy` + `verify-docs.sh` + `cargo test` before push | red-pipeline round-trips on reconcile (MR !4 took 2) | S | todo |
| DV3 | **Doc-header conformance at source** — seat pre-commit hook running `verify-docs.sh`, and/or `version` baked into the doc template seats copy | every reconcile Aegis hand-fixes seat docs missing the `version` header (recurring tax) | S | todo |
| DV4 | **Worktree bootstrap script** — `scripts/seat-worktree.sh <seat>` creates the worktree + wires `node_modules` + shared target in one step | manual per-seat `git worktree add` + `pnpm install` each time | S | todo |
| DV5 | **Parallel DB tests** — per-test transaction/schema isolation to drop the `--test-threads=1` serial constraint | DB integration tests run serially (race avoidance) | M | todo |
| DV6 | **CI cargo cache** — cache registry + incremental `target` across pipelines (GitLab cache key), beyond the baked image | CI rebuilds from cold each pipeline | S–M | todo |

Related but tracked elsewhere: coordination routing latency (Aegis→Lyra→worker takes minutes per
hop) is addressed by **G19 governed dispatch** (§4, P0 step 6), not duplicated here.

## 5. Coordination model (so escalations don't evaporate)

- **Aegis is the supervisor session** (Mr. Zhang's assistant), **not a tmux seat.** There is
  no `Aegis-*-seatloom` seat by design. Seats escalate by parking in the daily log and/or via
  Mr. Zhang; Aegis observes `track` + daily log and dispatches decisions to Lyra directly.
  (COORDINATION_RULES §12 to note that the supervisor is a session, not a seat.)
- **Dispatch chain:** Aegis → Lyra (PO) → worker seats. Lyra authors packets; Aegis signs off
  load-bearing/irreversible decisions; Flux verifies; Lyra merges to local `track`.
- **One shared working tree:** never two code packets in flight; serialize. (This is *code*
  serialization; *message* serialization is G19, step 6.)
- **GitLab reconcile is Aegis-only** (only Aegis holds the PAT); local `track` → GitLab `track`
  via MR through the hard gate.

## 6. The irony, stated plainly

Until G19 and the P0 capture loop land, this roadmap is a **markdown file a human updates** and
seat coordination is **human-paced tmux send-keys** — exactly the manual, attention-burning,
pollution-prone loop SeatLoom exists to replace. The plan is to make SeatLoom itself hold this
view and run this loop. This file is the interim; the SeatLoom UI over the captured event model
is the destination.

## Session snapshot + 奔终态 reprioritization (2026-10-08)

**Decision (Mr. Zhang): go straight to the END-STATE, no throwaway transitional work.** The
capture-based structured UI satisfies see-team / command-via-Aegis / history, better than the
terminal mirror. So:
- **Critical path (reprioritized):** step 4 MCP emit (Nimbus) → step 6 governed dispatch / G19
  (Nimbus) → step 5 first captured event → structured UI: **UM2 timeline + UM3 history/replay**
  (Mira, over `canonical_events` + `archived_sessions` + pi session transcripts) + **structured
  supervisor-IM** (talk to Aegis → governed dispatch → events).
- **KEEP (not throwaway):** UM0 live mirror + the mirror-fidelity fix — survives as the "drop into a
  seat's raw terminal" DETAIL view.
- **DROP (throwaway):** scrollback-for-history hack (TUI has no scrollback anyway); transitional B1
  "mirror-drive Aegis pane" as the primary IM (final IM is structured).
- step 6 (governed dispatch) also fixes the **Lyra dropped-hop** problem — until it lands, every
  hop needs an Aegis nudge.

**State as of 2026-10-08 (for resume after compaction):**
- Migration: all 5 worker seats on PI (recipe in memory/project_harness_compliance_migration_pi.md).
  Aegis stays Claude Code for now, NOT in tmux (this harness session; aegis-feishu ≠ Aegis).
- UM0 live mirror + pipe-pane fix merged to track (1a34007). Mirror-fidelity fix delivered on
  branches — Mira da4213b (viewport=geometry authority) + Nimbus 45d6c15 (resize tmux window) —
  PENDING Flux-verify + Lyra-merge (stuck on Lyra's hop).
- Archive: AgentHistory 271 sessions / 14GB ingested as `archived_sessions` (migration 011).
- UI running (pnpm tauri:dev, vite :1420) on track; shows seats but mirror garbles pi TUI until the
  fidelity fix merges + UI restart.
- **Pending Aegis tasks:** (1) merge mirror fix (nudge Lyra), (2) reconcile local track (~30 ahead)
  → GitLab, (3) rebuild stale local DB (001→011; running DB lacks 008/010), (4) restart UI after
  mirror fix. Then begin the reprioritized critical path (step 4 MCP).

**UPDATE 2026-10-09 (resume after compaction):**
- **CAPTURE CHAIN COMPLETE — step 4 + step 6 both LANDED on track.** step 4 MCP emit MERGED
  (9a2a6af). **step 6 governed dispatch (G19) MERGED (`04e9033`, Aegis)** — `enqueue_dispatch` /
  `get_next_dispatch` / `complete_dispatch` on seatloom-mcp-emit; A1 gained `append_event_in_tx`
  (single append impl, 7-C) + 3 dispatch EventTypes & Dispatch RefType (7-A). Nimbus wiring 9b96ee8
  + COMPLETE 4938545. Independently verified by Aegis on a FRESH scratch DB (001-012 canonical,
  never live): all 13 proof points green + Layer A (fmt/clippy -D warnings/tests) on core+mcp-emit;
  seatloom-tauri untouched. Lyra dropped the merge hop AGAIN; Aegis hand-merged (as step4/012). Also
  unwedged Nimbus's pi git-rebase vi-hang (core.editor=true, durable). **So governed dispatch now
  exists; the loop can self-run via MCP-pull (get_next_dispatch) instead of manual nudging.**
- **NEXT = step 5 (first REAL captured event):** drive one emit_event through the real MCP stdio
  path into LIVE canonical_events (live DB is 001-012 ready; 34 events baseline). Prove it lands +
  projection + retrievable. Then structured UI (UM2 timeline).
- **Live DB reconciled to 001–012 (DONE, step 5 unblocked).** The 10-day podman `seatloom-postgres`
  was stale: had 001-005/008(partial)/009/011 but NOT 010 and NOT a full 008 (`sessions.project_id`
  missing → 010's G5 premise failed). Applied 008→010→012 idempotently in-container via
  `podman exec -i … psql < schema/0NN.sql` (NOT apply-db.sh — it re-runs seeds; and psql isn't on
  the host PATH). Gotcha: order matters — 008 (adds sessions.project_id) BEFORE 010; 012 lands
  independent of 010 (own DROP+reADD of the vocab CHECK). Verified: schema_version/turn_seq/role/
  one-subject-idx/dispatches/one-in-flight-idx all present; data preserved (34 events, 41 refs,
  **271 archived / 13.7GB intact**); double-apply of 010+012 = clean no-op COMMIT. So the LIVE app
  DB can now accept a real emit_event → canonical_events write (step 5).
- **14GB history backup CONFIRMED whole:** archived_sessions = 271 rows / 14,708,064,001 bytes.
- **Remaining Aegis tasks:** reconcile local track → GitLab (needs PAT; CODE/SCHEMA/DOCS only, never
  data); after step 6 merges → step 5 first real captured event (live DB now ready) → structured UI
  (UM2 timeline / UM3 history-replay / structured supervisor-IM). Aegis migration to PI deferred per
  Mr. Zhang (stay Claude Code; handoff brief /tmp/aegis-pi-handoff.md).

## Parked research conclusions — verified durable (2026-10-08 sweep)

Confirmed already durable (committed review docs + §4 gap register), NOT lost on compaction:
data-layer 4-types + T1/T2/T3, object hierarchy project→topic→workitem→event, topic semantic layer,
external_ref, capture strategy emit/observe/interpose, "signal→derived revisable attribution" rule,
capability assets published-not-forked (G11), recurring-task object (G14), SUT-version dim (G15),
recording-shell (G13), T2 traces (G12), AND the **data-use goal** — workflow optimization +
training vertical models from `(context, reasoning, result)` triples (data-flow review §"what T2 is
for").

**Added (was only in conversation, now recorded) — provider-proxy for model-I/O capture (refines
G18):** to capture model reasoning/prompts+completions and to absorb value from ANY harness
(including closed ones), SeatLoom runs **its own provider-proxy** in front of the model endpoints.
Now concrete post-migration: the PI seats route through `stepplan`/`modelproxy`/`subworker`
(`~/.pi/agent/models.json`) — point those baseURLs at a SeatLoom-local proxy and every seat's model
I/O becomes a captured T2/T3 stream, harness-agnostically. This is the model-I/O capture channel
(complements step-4 MCP emit, which captures seat ACTIONS). P1/P2.

**Flagged, NOT fabricated:** an "AD for runtime-vs-knowledge separation" was mentioned earlier but
its exact content is not in any durable doc — reconstruct/confirm when G11 (capability assets) is
designed; do not invent it.

---

## 晨间简报 — 2026-10-09 夜间自主驱动结果(给张先生)

**今晚已落地并验证(硬核机械部分,无需你拍板):**
1. **Live DB 对齐到 001–012**(之前缺 010、008 半应用)。在 live app 运行中幂等补迁移,数据零丢(含 271 归档 / 14GB)。step5 的 live 阻塞清除。
2. **P0 step 6 治理派单(G19)合入 track `04e9033`**。Nimbus 接线 enqueue/get_next/complete + A1 的 append_event_in_tx(单一 append 实现)+ 词表加 3 个 dispatch 事件/1 个 ref。我在全新 scratch DB 独立验证:**13 点行为 proof 全绿 + Layer A(fmt/clippy/test)绿**。Lyra 又掉了合并跳,我手工合(并解了 Nimbus 的 vi 卡死,永久 `core.editor=true`)。
3. **P0 step 5 首个真实捕获事件 — 在 LIVE 库达成**。经真 MCP binary 把"step6 交付"作为真实工作捕获:WorkItemCreated→WorkItemStatusChanged(done),canonical_events 34→36,**projection 同事务写入 + 可按 subject 重建时间线**。
4. App 健康(vite:1420 200,DB 连通)。全程留痕(daily log + 本快照 + memory)。

**→ 捕获脊柱完整:step4(MCP emit)+ step6(治理派单)+ step5(live 首事件)全部打通并验证。SeatLoom 现在真的在把真实工作捕获进真实库。**

**需要你拍板才动的收尾(我没擅自做,避免返工/碰你依赖的东西):**
- **A. 把 get_next_dispatch 接进各 seat 的 pi MCP**(= 让"Aegis 派单→seat 自拉执行"真正自运行,永久消除掉跳税——你的核心目标)。需编辑每个 seat 的 `~/.pi/agent/models.json` 加 MCP server 条目 + 约定 seat 回合末调用。**风险:动你正依赖的 seat pi 配置、可能需重启其 pi 会话(丢上下文)。建议醒来一起选一个 seat 先试。**
- **B. 监督 IM UI(L1,可拆独立窗)+ UM2 时间线 UI**。数据层已就绪(canonical_events / dispatches)。但 UI 必须贴合你已定方向(V1 AppShell 参照、IM 独立 WebviewWindow、L1 先于 L2)——想先跟你确认第一刀范围再建,免得做了不合你预期的。
- **C. 本地 track → GitLab 对账**:需要 `api` scope 的 **PAT**(现仅会话 cookie);data 永不上库。给我 PAT 即推 code/schema/docs 过 CI 门禁。

**安全的下一步(可不等你、我在心跳里推):** 事件/派单的只读查询层(时间线 + 派单收件箱查询)——UI 必需、纯增量、可测、不涉 UI 外观设计,不会返工。若确认是干净缺口我就建。

### 晨间简报 · 补充勘误 — UI 比预想完整得多(2026-10-09 夜巡发现)

夜间巡检实读 UI 代码,修正前面"UI 需从头建"的判断。**大量终态 UI 其实已建好并接到 live 后端:**
- **UM2 时间线 = 已完成且可用。** `ui/src/views/TimelineView.tsx` + `EventRow` + NavRail「时间线」入口;
  `App.tsx` 启动 `useEffect` 调 `hydrateFromBackend()`,Tauri 下 `api.listEvents(400)`→`cmd_list_events`→
  用真实后端数据重建 store。**用户点「时间线」就能看 live 的 canonical_events(含 step5 捕获,app 重载后)。**
  后端查询层(`repositories.list_events*`)、`timeline_cmds`、`CanonicalEventDto` 都在。
- **监督 IM 也已建:** `cmd_append_supervisor_message`(把监督消息写进 ledger 为 SupervisorMessage 事件)+
  `cmd_list_supervisor_messages` + `SupervisorCommandBar.tsx` + **可拆独立窗 `supervisor/SupervisorApp.tsx`**
  + `cmd_open_supervisor_window`。符合已固化方向(IM 独立 WebviewWindow、V1 AppShell 参照)。

**→ 真正的、唯一的终态缺口 = "指令投递链"(IM/Aegis 发话 → seat 真正收到并执行):**
1. **IM→派单未接:** `cmd_append_supervisor_message` 目前只写 ledger + 一个 **stubbed** 的 PTY 注入 hook
   ("integration hook for later",按 session-id 非 seat-id)。需把它接到 step6 的 `enqueue_dispatch`
   (把监督指令变成 dispatches 行)。这是纯增量的后端接线,不涉 UI 外观,低返工风险。
2. **seat 未自拉:** 各 seat 的 pi 还没接 `get_next_dispatch` MCP 工具,所以不会自动拉派单执行。
   = **seat-MCP 接入**(编辑各 seat `~/.pi/agent/models.json` 加 MCP server + 约定回合末拉取)。
   **敏感**(动你依赖的 seat pi、可能需重启其会话)——建议醒来一起先拿一个 seat 验证闭环。
3. 新表 `dispatches`(012)暂无 UI 视图(派单收件箱/依赖图)——待 IM→派单链通后再加可视化。

**结论:你离"经 UI 经 Aegis 指挥 seat"只差这一条投递链,且两端(enqueue_dispatch / get_next_dispatch)
都已建好并验证。** 醒来定两件事即可一口气接通:(1) 准我把 IM→enqueue_dispatch 接上(我可自主做,安全);
(2) 选一个 seat 做 MCP 接入试闭环(需你在场,因动 seat 配置)。

---

## 2026-10-16 快照(张先生问「计划是什么」「为什么推不动」之后的如实记录)

**这一节的写法换了。** 前面几节是我用缩写写的,张先生两次说看不懂、说我在说黑话。
所以这一节尽量说人话。他问的三个问题,答案都在这里。

### 一、现在真的能用什么(我逐条点过,不是听席位报的)

| 东西 | 状态 |
|---|---|
| 席位干的活自动记进数据库 | 能用。账本里 989 条事件,每一件派单的发出/送达/完成都有记录 |
| 派活走系统、席位自己领 | 能用。我发一张单,空闲的席位几分钟内自己领走 |
| 监督指令入口 | 能用。在界面里打一句话,系统读出意图、生成提议卡,确认后才真的派单 |
| 历史会话(按项目翻自己过去的对话) | **活做完了,在你手里等点确认**。38 个项目、183 个会话、36680 条消息,数据已在 live 库 |
| 知识面板 | 已合进主分支。但在主导航里进不去,要从 ⌘K 面板的项目视图里才看得到 |
| 时间线 | **标记是假的**:约 600 条真事件上面写着"示例数据,尚未来自实时捕获" |
| 点时间线上的一条事件 | **没反应**。类型定义里没有"事件"这一项,所以点开没有详情 |
| 监督面板的目标/阶段两节 | **永远是空的**。代码读的是假数据,而假数据的键和真项目 id 对不上 |
| 四个表单(工作项/交接/席位/委派) | **输入完提示成功,一刷新数据就没了**。后端没有对应的保存命令 |
| 应用图标 | **没有**。配置里图标是空数组,唯一的文件是个 32×32 的占位图。已派活 |

### 二、为什么之前说"没有活"

这是我这天最该认的错。心跳规矩让我每轮查"有没有派给我的单",我就只查这个,
查完没事就报没活。我没做两件事:**没去看你点下去会发生什么**,也没去看队列里别人有没有在推你关心的东西。

实际情形是:产品负责人已经把上面那些问题拆成了六片活,全部派出去了,在流转。
而做好的历史会话画面躺在她分支上三天 —— 因为按你定的规矩,验收卡只在你点接受时才合并。

**那道门是你为了不蒙自己设的,现在它成了最堵的一环。** 四张卡等你,最老的两张压了五天。

### 三、这一轮修掉的两个"不报错的检查"

1. **应用停了两天,监控一直报"开着"。** 判据是 `pgrep -f 'seatloom-tauri' | head -1`,
   而这个模式会匹配任何命令行里含这个字符串的进程 —— 我自己的命令、构建进程全中过,
   `head -1` 再随便挑一个,于是把一个早没用的构建进程当成了应用。改成按可执行名过滤。
2. **验收卡开了,但从没给你准备过"点之前能验"的程序。** 你指出这是死锁,对的。
   现在规矩改成**卡和构建一起到**:任何一张卡开到等你点,必须先用
   `scripts/prepare-verification-build.sh` 把带着那份活的程序构建好、装好、起来,再告诉你。
   那个脚本会证明二进制里真的带着要验的东西,证明不了就报错退出、不安装。

### 四、在流转的活(按依赖顺序)

1. 血缘读取模型(后端)—— 做完了才能点开时间线上的一条事件看它的来龙去脉
2. 接线诚实度(后端)—— 把写死的"已注册"开关换成实测
3. 接线诚实度(界面)—— 修那个说谎的时间线标记、死条目、四个假表单
4. 血缘详情面板(界面)
5. 验血缘读取模型(质量席位)
6. 应用图标(界面)—— 今天新派的

### 五、等你的事

**四张验收卡,用现在装着的这份程序全都能验**(洗数据流水线、监督意图入口都在主分支上,
历史会话在装好的那份里)。每张卡都写着"看什么 → 什么算对 → 什么算打回"。

另外一件:pi 的默认模型已按你要的设成 `modelproxy-anthropic/step-5-preview-sys`
(`defaultProvider` + `defaultModel` 必须成对设,只设模型名会被 pi 挑错 provider)。
但六个席位不吃这个默认值 —— 启动脚本从数据库里取模型显式传。要改席位的模型得改数据库,
那是我被权限封顶的动作,需要你跑一条命令。

---

## 2026-10-16 优先级更正(张先生:「镜像只是过渡态」)

他纠正了我两处,第二处改变排序:

1. **⌘K 打开的是监督面板,是消息对话框,不是镜像。** `SupervisorPanel.tsx` 的注释原话:
   「可拖拽、可调整大小的 IM 风格浮窗,包含左侧联系人列表和右侧对话区」。
   镜像(实时席位)是主导航的另一个页签,和 ⌘K 无关。我把两者混过,已改。
2. **镜像只是过渡态。** 这和 2026-10-08 他自己定的「直奔终态,不做一次性过渡」一致 ——
   本文件第 166 行 KEEP 镜像为「钻进某个席位原始终端」的详情视图,第 168-169 行
   DROP 把镜像当主要 IM 的做法,最终 IM 是结构化的。

**我据此承认今天排错了优先级。** 大量精力花在镜像上(实时席位重连已合并、
滚动历史在等验证),而结构化那几面的坏东西到这一刻才补派:
监督面板目标/阶段两节永远为空、派单依赖列显示机器码、给席位发自由文本消息会静默丢失。

**以后的排序判据(一句话):一件活是让「结构化界面」更能用,还是让「过渡态」更好看?前者优先。**
镜像类工作只在服务「钻进原始终端」这个具体用途时才做。

---

## 2026-10-28 · Lyra — the app the operator launches has no frontend, and three acceptance cards cannot be run

**The next round is one thing: make the product usable again.** Not a feature. Measured this turn
(heartbeat `LYRA-2026-10-28-heartbeat-findings-v1`):

**Fact A — CORRECTED, see the correction note in `LYRA-2026-10-28-heartbeat-findings-v1.md` §6.**
My first claim was wrong and is retracted. I said `/Applications/SeatLoom.app` is a dev build with
zero embedded frontend blobs, so a reload shows no UI. The instrument was wrong: I scanned for
**zlib** streams, and Tauri v2 embeds the frontend **brotli**-compressed — so the scan returns 0 for
every Tauri app, healthy or not. A check that cannot fail told me what I already believed.

**Fact A′ — measured correctly:** the installed app is **byte-identical** to
`target/release/bundle/macos/SeatLoom.app` (`md5 28925aa5158d2b121952aef8bc671126`). It is a **real
release bundle** embedding a **complete** frontend (2 HTML + 5 asset blobs, no dangling references).
What it is, is **stale**: built 10-27 14:02, while **40 commits have landed on track since**, 4 of
them product code. `verify-tauri-bundle.sh` names it `STALE` on 6 of 8 dist files. So the app works
and looks complete — it shows the **10-27 14:02** UI.

**Fact B — three of 28 pending cards cannot be run as written.** `acc-disp-CACpK8qlLm96WQY08itl-`,
`acc-disp-HlxnnSKW2lX-rMR02AFk2` and `acc-disp-YepFZb2hc62cZEO1UkJFX` name the ref
`track/infa-foundation` (missing an `r`) in `acceptance_plan` and `how_to_verify`; the command the
operator is told to run fails with `fatal: Not a valid object name`. The claims are true and the
verdicts are valid passes, so **Accept works** — the damage is that step 1 dies.

**Fact C — nothing accepted was lost.** All 26 accepted cards' 11 distinct `merge_commit`s are
ancestors of track. All pending cards are clickable and carry a valid pass verdict about their own
dispatch. The 10-17 mockup finding is closed: the `Pi` harness variant (`f316201`) is on track and
live carries `Pi` × 10.

**Order of work:** (1) land a current bundle — verified by decompressing its blobs and
content-matching `ui/dist`, never by listing `Resources`, never by grepping the executable, both of
which are wrong in both directions — installed over the app with the old one kept; the restart is the
operator's, not a seat's. (2) the three cards cancelled and re-opened under the same ids with the ref
spelled correctly — **done 2026-10-28 16:13**, verified runnable. (3) the ref-existence control —
**done as `scripts/check-cards-runnable.sh`**, mutation-tested, and it found two bugs in itself
(§7 of the findings note).

**Open on the operator, not on a seat:** whether the app should be a bundle build at all, or a dev
build with `pnpm dev` kept alive. The two answers need different work and only he can choose.

## 2026-10-17 · Lyra — the next round is integration, because the app is a mockup right now

Raised by Aegis's dispatch `disp-C2a6xeJUDA08dEAycc3WK`. Plan:
`tasks/lyra/LYRA-2026-10-17-next-round-plan-v1.md`. Operator-facing version:
`docs/coordination/acceptance/2026-10-17-app-is-a-mockup.zhs.md`.

### The one fact that gates everything

**The application cannot read the database, so every screen that reads through the data store is a
mockup. It started this morning and it is a regression, not a long-standing defect.**

| # | Link | Evidence |
|---|---|---|
| 1 | The operator ran `scripts/fix-seat-harness-to-pi.sh` this morning per his own ruling. All 8 live seat rows now carry `default_harness = 'Pi'`. | `select id, default_harness from seats` → `Pi` × 8 |
| 2 | `Harness` on track is `Direct, Stepcode, Custom(String)`. No `Pi`. | `session.rs:55` |
| 3 | `from_str("Pi")` falls to `parse_custom_tagged("Pi","Custom")`, which needs the literal `Custom(...)` shape. A bare `Pi` does not start with `Custom` → `Err`. | `session.rs:126-134`, `:151-160`, `HARNESS_EXPECTED` at `:145` |
| 4 | Every seat read validates that column: `row_to_seat` → `check_seat_domains` → `check_domain::<Harness>`. | `repositories.rs:2899-2916`, `:2855-2875` |
| 5 | So `list_seats` fails on **every row**. | 3 + 4 |
| 6 | `hydrateFromBackend` puts `api.listSeats()` in one `Promise.all` with seven other reads. One rejection rejects all. | `useDataStore.ts:1589-1598` |
| 7 | The `catch` sets `backendHydrated: false` and leaves `INITIAL_DATA` in the store. | `useDataStore.ts:1713-1718`, `:1392-1395` |
| 8 | **Therefore 8 of 10 NavRail tabs render fixtures.** | measured, plan §1 |

Corroborated independently and *called rather than read off code*: Nimbus's
`packet/nimbus/harness-pi` records `list_seats` succeeding with his `Pi` variant against the same
live rows. So the failure is exactly the missing variant and nothing else.

### Q1 — surfaces that look wired but aren't (all measured)

| # | Surface | Actually | State |
|---|---|---|---|
| 1 | 8 of 10 NavRail tabs | `useDataStore` → `INITIAL_DATA` | mock now |
| 2 | The LIVE badge on `live-seats`, `sessions`, `dispatches` | `provenanceForTab` returns `'live'` **before** consulting `backendHydrated` (`provenance.ts:65`) | the lie is on track |
| 3 | Any project's seat list | all 8 seats under every project, v0.1 compromise (`useDataStore.ts:1623`) | fixed on branch `f316201` |
| 4 | A seat's role | out-of-domain role silently swallowed, coerced to a default (`useDataStore.ts:1610`); live has `worker` | the swallow is on track |
| 5 | `ql_delivery` in the app | not present at all — the store shows `p-1/p-2/p-3` while live holds 39 projects | fixed on branch `cc0e409` |
| 6 | The QL project | **two rows, one directory** — `project_delivery/QL` and `ql_delivery` | unresolved |
| 7 | The ⌘K work graph | mock keyed `ch-p1/p2/p3` vs live id `seatloom` (`ProjectDashboard.tsx:454`) | on track |
| 8 | The four write forms | zero `api.*` calls; no create command exists | fixed on branch `d4a82dd` |
| 9 | The acceptance merge gate | existence-only, message promises green | fixed on branch `c63d7af` |
| 10 | My own two slices' downstream | **both stalled on ids that will never exist.** Nimbus emitted `ev-blocked-on-whom-dto-delivered`; Mira emitted a server-minted id. Both dispatches are `done`; neither `-landed` id exists | a known failure mode, recurred twice |

### Q2 — what the first screen still lacks

The default landing is still `live-seats` on track; the change to `dispatches` is on Mira's branch.
`LiveSeatsView` shows tmux terminals — a different question from *what is the fleet working on*.

| Missing | Substitute today | Worth doing |
|---|---|---|
| What each seat is working on | switch to the dispatch tab | **yes — this is his question** |
| Whether the app is reading the DB at all | a badge that lies on 3 tabs | **yes — cheapest high-value item, one line** |
| What needs his decision, in one place | he must know to open the dispatch tab; rulings live in the ROADMAP | yes — the K3 gap |
| The fleet at a glance incl. idle seats | the seats tab, currently mock | yes, downstream of §0 |
| What is blocked and on whom | the dependency column, raw ids on track | partly done |

### Q3 — QL's shortest path

Not a git repo (`git rev-parse` → fatal). MCP wiring already correct (`.mcp.json`). DB rows exist.
**The binding's role `worker` is out of the `SeatRole` domain**, so every read of that project's
bindings fails. The acceptance/merge path cannot work there — `request_acceptance` needs `base` and
`branch` to name existing local branches.

1. Run `scripts/fix-ql-role-domain.sh` — **operator only**.
2. Decide whether QL becomes a git repo — **operator only**, and it is the fork. If not: capture,
   memories, topics and dispatches all work; only the merge has no object, and the product should
   say so rather than offer a button that cannot succeed.
3. Resolve the duplicate project row — **operator only**.
4. Then dispatch to `seat-ql-001`, which today has **zero** dispatches.

**A correction to Aegis's brief, with evidence.** He wrote the out-of-domain role's consequence is
"ql_delivery shows 0 seats". The code does not do that: `useDataStore.ts:1610` catches and
substitutes an empty map, and `seatFromDto` coerces a missing role rather than dropping the seat —
so QL would show **all 8** seats with ql's role silently replaced. The real consequence is quieter
and worse. And right now neither happens, because §0 means the app is showing `p-1/p2/p3`.

### The decision

**The next round is integration, not new build.** One milestone he can perceive: *open SeatLoom and
see the real fleet.* Everything beneath it exists: Nimbus's `Pi` variant (Flux verifying now),
Mira's 11 UI commits (two cards open, waiting on his click), the role fix on track but unapplied,
the provenance one-liner, and the two re-emissions nobody has done.

No new surface, partition or interaction is proposed — he forbade it on 10-16 and nothing has
lifted that.

---

## 2026-10-29 · Lyra — what is actually true now, and the one thing that should be next

This section replaces nothing. It retires what is fixed, restates what is not, and adds what was
measured this week. Every claim below was re-measured on 2026-10-29 against the code and the live
database, not carried forward from the 10-16 snapshot. Where I could not measure something I say so
rather than guessing.

### Retired since 10-16 — measured fixed, do not re-dispatch these

| 10-16 claim | Measured now | Where |
|---|---|---|
| The app has no icon; the config is an empty array | **Fixed.** `tauri.conf.json` lists five icon paths, and `icons/` holds real 128×128 and `.icns` files | `src-tauri/tauri.conf.json:28-35` |
| Clicking a timeline event does nothing — there is no `Event` type | **Fixed.** `'Event'` is in `SelectedObjectType` and `case 'Event':` returns the event itself | `ui/src/App.tsx:58, 515, 606-610` |
| The supervision panel's goals and phases sections are permanently empty, reading fake data | **Retired — the sections are gone.** `SupervisorPanel.tsx` contains no goals/phases code at all, and `SupervisionDashboard` reads `useDataStore` with no mock or stub | `ui/src/app-v2/panel/SupervisorPanel.tsx`, `ui/src/components/SupervisionDashboard.tsx:13, 38-41` |
| The timeline is labelled "example data, not yet from live capture" | **Retired.** No `示例数据` marker remains anywhere in `ui/src`. The one surviving occurrence is `LiveSeatsView.tsx:447`, which says the opposite — that the view never fills itself with sample data | `grep -rn "示例数据" ui/src` |
| Four forms say they wrote the data and lose it on refresh | **Fixed as honesty, not as a write command.** `CannotStoreNotice` is rendered by all four forms (`WorkItemForm`, `HandoffForm`, `AddSeatDialog`, `DelegationOverlay` — 2 references each) and says in plain words that this build cannot store it and closing discards the input, naming the exact missing command | `ui/src/components/CannotStoreNotice.tsx` |

**On the last one, the judgement call.** I checked whether a write command *could* be wired: a
workitem is created by appending a `WorkItemCreated` event, which `write_api.rs:639` turns into the
row. `emit_event` is an MCP tool for seats. **So the form has no honest write path today, and
"Cannot store this" is the correct behaviour rather than a workaround.** Whether the UI should be
able to append ledger events at all is an architecture decision, not a UI one, and it is listed below
as an operator question.

### Still open — measured today, still true

| Defect | Evidence | Owner |
|---|---|---|
| **The reconcile notification can never appear.** `showReconcileSummary` is declared at `App.tsx:385` and `setShowReconcileSummary(true)` occurs nowhere in the file. The only writes are two `false` ones inside its own close handlers, so the guard at line 884 has a permanently false first conjunct. The count it would show is also the wrong quantity — `App.tsx:387` counts workitems with `status === 'Blocked'` under a reconcile-shaped name. Live has 7 workitems, all `done`, zero `Blocked`, so it would render `count={0}` anyway. | `grep -n "setShowReconcileSummary(true)" ui/src/App.tsx` → no match | Mira |
| **The document surface is built, tested and unreachable.** `DocumentsWorkspace.tsx` has a `reconcileResult` state and is imported by nothing except its own test. `App.tsx` has no documents nav entry. `list_documents` (`repositories.rs:2445`) has no caller. Meanwhile `run_reconcile` **is** wired — `src-cli/src/main.rs:69` and `src-tauri/src/commands/reconcile_cmds.rs:16` — so the engine runs and nothing shows its output. | grep for imports and callers | **operator decision first, then a seat** |
| **A fresh database cannot be built.** `apply-db.sh` lists 021 before 011 in `POST_SEED_SCHEMA`, but 021 does `ALTER TABLE archived_sessions` (line 147) and 011 creates that table (line 18). `ON_ERROR_STOP` kills the run there; 022's failure is a cascade, not a second defect. **Verified twice independently** — by me and by Aegis on his own scratch database. Fix verified: moving 021 after 011 gives zero failures and 32 tables matching live exactly. | `lyra_freshproof`, `aegis_seedproof` (both dropped) | Onyx |
| **The seed does not describe live.** Rebuild gives 6 seats all `Stepcode` / `claude-opus-5[1m]`; live has 10 all `Pi` / `stepplan-wash-anthropic/step-5-preview`. `documents` is 14 in a rebuild and **0** in live, and no migration deletes documents — the only `DELETE FROM documents` in the workspace is test teardown at `dispatch.rs:3485`, and all 14 files the seed names exist on disk today. | scratch rebuild vs live | Onyx (005 is written; needs merging) |
| **The seat drift above is not user-visible, which is why nobody noticed.** `defaultHarness` is declared in `ui/src/lib/types-dto.ts:29` and crosses the wire, but nothing renders it. The `live-seats` tab reads `cmd_list_tmux_sessions`, not the `seats` table. So this is schema truth, not product truth. | grep `defaultHarness` outside `types-dto` → tests only | — |
| **The five standing-duty documents are still not on track.** `on_track=0` for all five, so the waker — which reads a *relative* `docs/coordination/tasks` against the primary checkout on `track/infra-foundation` — still sees only Lyra. Ten nudges for Lyra, zero for the other five, measured live. | `waker.rs:112`, `lsof -p <waker>` | **operator merge** |

### The decision for this round — and it is not a UI fix

**Next: make the document corpus real, or say plainly that it is not.**

Everything else on the list is either assigned or is a one-line reorder. This one is different,
because it is the only item where the honest answer might be "delete the feature", and that is not a
call a seat should make quietly.

The facts: `run_reconcile` is wired into both the CLI and a registered Tauri command. It reads
`documents` — and live has **zero** rows there while a rebuild produces 14 that all point at files
which exist on disk right now. The view that would display them (`DocumentsWorkspace`) exists, has
tests, and has no route. The one control that mentions reconcile is dead.

So the project has a document-ingestion engine that runs, a corpus that is absent from live, and a
display surface that is unreachable. Three of the four possible states of that system are defects.
The fourth — documents are not part of the product — is a legitimate position, and if it is the
position then the honest action is to remove the dead code and the unrouted view, not to leave them
as a feature that looks half-built.

**Why this one and not the reconcile badge:** the badge is one dead control on one tab, and it is
Mira's to fix with the standing-duty check I already handed her. The document question decides
whether a whole subsystem ships, and it has been sitting unanswered since 2026-04-29, when the seed
that would have populated it was written.

**What I am explicitly not doing next:** not merging my own branch into track, not reordering
`apply-db.sh` (Onyx's file), not editing `App.tsx` (Mira's surface), not opening a card for anything
that has not been measured, and not writing `roles/VELA.md` on Vela's behalf — that is his first job
when he next has a turn and I have said so in his heartbeat.

### Operator decisions needed — these are yours, not a seat's

1. **Documents: ship it or remove it?** If ship: `DocumentsWorkspace` needs a route,
   `list_documents` needs a caller, and live's zero rows become a visible hole that must be filled.
   If remove: delete the unrouted view and say so in the roadmap, so nobody re-discovers it.
2. **Should the UI be able to write to the ledger?** Today a workitem can only be created by a seat
   calling `emit_event` with a `WorkItemCreated` payload. That is why all four forms are read-only
   and why the honest notice is the right answer. A UI that appends events is an architecture change
   with consequences for the append-only ledger, not a missing button.
3. **Merge `packet/lyra/milestone-summary` into track.** This is what makes the five standing duties
   real. Until it happens the fleet has one seat with standing duties and I have been reporting six.

---

## 2026-10-29 · Lyra — what the operator needs to decide, measured since the last section

This section is the operator-facing part. The section above it is the product analysis; this one is the
list of things a seat cannot decide, each with the measurement behind it. **Nothing here is a
hypothesis.**

### 1. The acceptance checker on track contradicts the product's own guard — and it has cost three cards

This is the most consequential item and the one with a definitive answer.

The product's own acceptance guard, `flux_verdict_facts` (`crates/seatloom-core/src/db/repositories.rs:779`),
states in its **own doc comment** that a verdict may name its dispatch by **either** channel:

> Both are things Flux actually writes — `payload->>'verdict'` and `payload->>'dispatch_id'`, or an
> `event_object_refs` row with `ref_type='dispatch'` … **Either channel counts.**

and the code is `names_dispatch = payload_dispatch_id == Some(id) || names_via_ref`. **That guard has
accepted the two-channel rule since 2026-10-16.**

`scripts/check-verdict-refs.sh` on track checks **only** the payload channel, and prints the loudest
sentence in the repository: *"Do not let the operator click those."* It is the only component in the
system that disagrees with the guard.

**Measured cost — three cards cancelled, each `feedback` field naming the script:**

| Card | Cancelled | Would the guard have accepted it? |
|---|---|---|
| `acc-disp-ibABeNk-nkEDAkI6Qj2sG` | 18:04:32 | **yes** — verdict `pass`, about the dispatch |
| `acc-disp-b5VbRor1fHhP3nN4YZI_l` | 18:04:32 | **yes** — verdict `pass`, about the dispatch |
| `acc-disp-reemit-stalled-announcements` | 18:04:48 | **yes** — verdict `pass`, about the dispatch |

Mira did not simply obey the script. She checked and pushed back, correctly: the verdict's payload
carries a `re_emitted_events` key whose `both_edges_exist` names the two landed event ids individually,
so it genuinely verified her re-emission; and `ev-AbuUI51WIx1Ks-Eb_4FZp` was already accepted through
the refs channel by two cards that are merged. She cancelled anyway, and her reason is the whole problem
in one line: **a card the operator cannot click is not a card.**

**The ask, deliberately narrow.** Not "merge my branch" — that carries five standing-duty documents and
four unrelated findings and is the operator's call. **Land `scripts/check-verdict-refs.sh` from
`packet/lyra/milestone-summary` onto track.** That one file stands between three correct cards and their
acceptance. If Aegis would rather write it himself, that is fine; the content is no longer in question,
because the guard's own doc comment states the rule the script violates.

### 2. The branch-sync refusal — 18 emergency dispatches and counting

`DispatchRejection::BranchNotSynced` refuses `request_acceptance` when a branch is behind its base, on
the stated grounds that it "would merge work that never saw the base's later commits". **Measured on a
scratch repository, twice, 26 hours apart: that is not what git does.** A branch 1 behind with
non-overlapping changes merges with exit 0 and all files present; the overlapping case produces a
CONFLICT and git refuses it itself.

**Measured cost: 18 dispatches** with "will be unclickable" in the title, across 5 seats, two days. Each
pulls a seat off its task so a card it already earned becomes clickable. The check fires at *request*
time, so a seat can be current, receive a card, and be told an hour later that its card is unclickable
because someone else committed.

**Three options, proposed 26 hours ago, still unanswered:**

1. **Downgrade to a warning carried on the card**, with the exact `git rev-list --left-right --count`
   figure. The Accept path merges safely; what remains of the check is a *disclosure* duty, and
   disclosure belongs on the card rather than in a refusal. *(My recommendation.)*
2. **Move the check to Accept time with a one-action auto-sync offer.** One interaction at the moment
   it matters instead of eighteen dispatches.
3. **Keep it, and stop calling the syncs EMERGENCY** — batch one per seat per round. Defensible, but
   then the fleet should not be interrupted eighteen times for a condition git already handles.

### 3. Still open from the previous section, unchanged

- **Documents: ship it or remove it?** The reconcile engine is wired (`src-cli/src/main.rs:69`,
  `src-tauri/src/commands/reconcile_cmds.rs:16`), the display surface is built and tested and unrouted,
  and live has **zero** `documents` rows while a rebuild produces 14 that point at files which all exist
  on disk. Three of the four possible states of that subsystem are defects; the fourth — documents are
  not part of the product — is legitimate, but then the honest action is deleting the dead code.
- **Should the UI be able to write to the ledger?** Today a workitem can only be created by a seat
  calling `emit_event`. That is why all four forms are read-only and why the honest notice is correct.
- **Merge `packet/lyra/milestone-summary` into track.** Five standing duties exist on that branch and
  are `on_track=0`. The waker reads a relative path against the primary checkout, so until this merge
  happens the fleet has **one** seat with standing duties, not six — and I have been reporting six.

### 4. NEW, and it is live right now — should the fleet's own conversations go into 历史会话?

Aegis asked you this at 04:01 today, after you asked why QL's history in the app looked nothing like
tmux. He traced it to the bottom: the archive holds **six** harnesses (`codex` 210, `stepcode` 20,
`claude-code` 18, `cursor` 14, `gemini` 9, `opencode` 1) and **not `saih`** — the one every seat runs on.
So opening QL history shows that seat's work in another tool months ago, and none of its work as a
SeatLoom seat.

Vela measured the corpus rather than opining: 40 `saih` files, 51,748 records, roles assistant 25,173 /
toolResult 23,854 / user 1,598 / system 58, with ordinary tool calls (bash 18,874, codemode 2,208, read
983) plus your own typos (`cash` 82, `bodemode` 52) — which is itself evidence these are real
transcripts.

**Three things you should know before you answer, and one correction to the question as asked.**

**Correction first, because it changes the cost.** The question says adding a harness would change 011's
closed vocabulary and is therefore a migration. **It is not.** I measured: `archived_sessions` has exactly
one constraint — `PRIMARY KEY (harness, id)` — and no CHECK on `harness`, so it is free TEXT. The Rust
read path does not validate it either (`repositories.rs:3466` is a plain `harness: r.get(1)`). The
`Harness` enum in `objects/session.rs` is the **seat** harness, a different column on a different table.
So rows with `harness='saih'` are accepted by the schema today — no DDL, no migration, no
`apply-db.sh` registration. **The adapter is the whole change.** A false premise that makes an option
look more expensive than it is biases toward doing nothing, which is why I am correcting it before you
answer rather than after.

**Second: the option that is already chosen is the worst one.** Vela's point, and I agree with it: the
feature exists, covers five harnesses, and silently omits the one every seat runs on. You opened QL and
saw a live pipe stream instead of a recording, and nothing in the product told you why. Silence is a
choice, and it is currently making the decision for you.

**Third: the real gate is redaction, not schema.** Vela found **38 intranet-domain hits and 21
basic-auth hits in her own session file**. She also measured the already-archived corpus at the same
density (a 62 MB codex file: 32 intranet + 26 basic-auth; a 15 MB claude-code file: 9 + 79), so `saih`
is **not** a new risk class — which is an argument for archiving it *through* the pipeline that already
handles the others, not for leaving it out. She wants a **full, not sampled, redaction dry-run** before
anything is written. **That gate should stay yours**, because a leak through the archive is not
recoverable.

**The three options as Aegis put them**, with my note on each:

| Option | What it means | My note |
|---|---|---|
| A — add it | a thin `handle("saih", …)` source adapter | Cheaper than the question says. Leaves "cannot tell apart" unsolved. |
| B — do not add, but label | the panel says it does not cover the fleet's own conversations | Honest, and leaves the fleet's work invisible to the product that governs it. |
| C — add **and** distinguish | add it, filtered or tagged, so fleet working state is not mixed with your personal history | **I lean here.** It is the only option that solves both, and Vela's objection to mixing is the strongest argument on the table. |

**What I am not doing:** not ruling it. This is your surface and you are engaged on it right now. What I
am ruling is that the question must be answered rather than left as an accident of which harnesses the
corpus happens to name.
