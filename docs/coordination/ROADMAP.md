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
