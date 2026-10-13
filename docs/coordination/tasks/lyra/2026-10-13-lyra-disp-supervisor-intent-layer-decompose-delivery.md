---
template: T5
subtype: delivery
doc_id: 2026-10-13-lyra-disp-supervisor-intent-layer-decompose-delivery
title: Delivery — the Supervisor Layer intent-entry contract, its decomposition, and the one product-truth decision that needs Aegis
status: active
author: lyra
doc_date: 2026-10-13
version: v1
depends_on: AEGIS-2026-10-13-supervisor-intent-entry-brief.md, LYRA-2026-10-13-supervisor-intent-entry-contract-v1.md
tags: delivery, lyra, supervisor-layer, intent-entry, decomposition, contract
---

# Delivery — `disp-supervisor-intent-layer-decompose`

## What was delivered

The three-layer product contract for the Supervisor Layer intent entry, at
`docs/coordination/tasks/lyra/LYRA-2026-10-13-supervisor-intent-entry-contract-v1.md`
(`9cde733` on `packet/lyra/milestone-summary`), plus two new slices in the dispatch graph.

| PRD hard rule | Where it is satisfied |
|---|---|
| A user story | §2 — who, what they want, how they are stuck today |
| An interaction flow | §3, including §3.3, the ambiguity/failure table |
| A screen / UI surface definition | §4 |
| Pain / omission cost / behavior change | §5 |
| Out of scope, with reasons | §6 |

The referenced packet path in the dispatch payload
(`docs/coordination/tasks/aegis/AEGIS-2026-10-13-supervisor-intent-entry-brief.md`) did not
exist when the dispatch was pulled; it exists now and was read in full. The contract was
written against the brief plus eight facts measured in the working tree, listed below.

## The three measurements that changed the plan

**1. The hard part is already built.** `cmd_enqueue_dispatch` is registered
(`src-tauri/src/main.rs:171`) and already exposed to the UI as `api.enqueueDispatch`
(`ui/src/lib/api.ts:160`). Its implementation
(`src-tauri/src/commands/dispatch_cmds.rs:57`) shells out to `seatloom-mcp-emit` over stdio
with `SEATLOOM_ACTOR_REF = "human:supervisor"`. **A sentence typed by the user can already
become a governed dispatch attributed to the human.** What is missing is the front door, not
the machinery — which is why this contract does not propose building a supervisor.

**2. …but the wrapper silently drops preconditions.** `cmd_enqueue_dispatch` passes only
`target_seat_id`, `title`, `payload:{note}`, `priority`. The underlying MCP tool accepts
`dependencies: [{dep_kind, dep_value}]`. So "do X, but only after Y lands" typed by the user
is currently **downgraded to "do X"** with no signal. This became a hard requirement on the
backend slice: close the gap, or refuse the intent explicitly. Never silently drop.

**3. `Cmd/Ctrl+K` is bound twice, and every description of it is false.**
`ui/src/hooks/useGlobalShortcuts.ts:24` opens the V1 prototype; `ui/src/App.tsx:139` toggles
the Supervisor IM. Both are `window` keydown listeners and neither calls
`stopImmediatePropagation`, so **both fire**. Meanwhile `INT-02` says the chord opens the
command bar, the comment at `App.tsx:133-135` says the V1 bar is on `Cmd+Shift+K`, and
`ShortcutHelpDialog.tsx:11` tells the user the chord opens the command bar. Three documents,
three different bindings; the code does a fourth thing. **Today the chord opens the canonical
IM and a 385-line prototype with a hardcoded proposal, simultaneously.**

Also confirmed: `ACCEPTANCE_CMDS_REGISTERED` (`ui/src/lib/api.ts:67`) is a hand-maintained
boolean asserting a wiring state that nothing measures — the exact shape Aegis named in
boundary 3. It happens to be correct on this commit. Mira must replace it and any sibling with
a measured capability probe.

## The placement decision, and why

The intent entry is the **input row of the existing Supervisor IM panel**, with two explicit
modes — structured work, and a plain message to a seat (the existing behavior). Structured
first, free text second, never blended.

The V1 `SupervisorCommandBar.tsx` is **not** the target surface and is **not touched**: 385
lines, zero real calls, `generateProposal()` is a `setTimeout(800)` returning a hardcoded
object (`owner: 'seat-2'`, a composed rationale sentence, a fixed evidence array,
`budget: '150,000 tokens'`), and `handleConfirm()` writes to a Zustand store and stops. It is
also already dirty, so editing it would turn it red under the file-granularity check for no
product gain.

**Routing rationale is computed, not narrated.** `seats.capability_tags`,
`project_role_bindings.{role, authority_doc_refs, constraints, collaboration_template_ref,
active_delegation_id}` and `dispatches.status` already hold everything a "why this seat"
answer needs. The prototype's composed sentence was never a data limitation — it was a
placeholder that was never replaced. The contract requires per-candidate facts with sources:
which capability matched, which constraint excluded each other seat, how many dispatches that
seat currently holds.

**The no-guessing rule is the load-bearing section.** §3.3 lists ten conditions — unsupported
verb, owner matching zero seats, owner matching more than one, owner absent and not inferable,
unclear object type, a precondition the write path cannot express, no capable seat, budget
exceeded, insufficient evidence, write failure — and requires the system to **stop and ask** in
every one. The current prototype's actual behavior is to default to Nimbus, which is guessing.
Flux is required to test every row adversarially and report the exact input and output; a
parser that usually stops is a fail.

## The decomposition, with edges as event ids

| Slice | Seat | Dispatch | Edges |
|---|---|---|---|
| 0. Contract | Lyra | `disp-supervisor-intent-layer-decompose` | — → emitted `ev-supervisor-intent-contract-landed` |
| 1. Intent tool surface | Nimbus | `disp-supervisor-intent-mcp-surface-v2` (Aegis's) | `ev-supervisor-intent-contract-landed` → now **ready** |
| 2. UI against the contract | Mira | `disp-supervisor-intent-ui-v2` (Aegis's) | same → **in_flight**, already pulled |
| 3. Verify the tool surface | Flux | `disp-supervisor-intent-tools-verify` (**new**) | `ev-supervisor-intent-tools-landed` → blocked |
| 4. Wire the UI to the live tools | Mira | `disp-supervisor-intent-ui-wiring` (**new**) | `ev-supervisor-intent-tools-live` **and** `ev-verdict-supervisor-intent-tools` → blocked |

Why those edges: a verifier needs the code to *exist*, so a `-landed` announcement is correct
for slice 3. The wiring step actually *calls* the capability, so it needs the measured `-live`
gate — and it must not wire code that has not passed verification, so it waits on the verdict
too. All four expected event ids and their required payloads are written down in the
contract's §7.1, so nothing in the chain depends on a seat remembering what to emit.

Verified after emitting: the two Aegis dispatches flipped to `ready`/`in_flight` on the
contract event, and the two new ones are `blocked` on referents that do not exist yet. The
trigger works end to end.

## The one decision I need from Aegis

**Which chord owns the intent entry, and what happens to the V1 prototype's binding?** I
placed the entry inside the IM panel so the feature does not depend on the answer, but the
duplicate binding is a live defect on its own: today that chord opens the canonical IM *and*
a hardcoded prototype at the same time. I am not moving a shortcut the operator uses daily,
and I am not amending `INT-02`, on my own authority. Until he rules, slice 2 is forbidden from
touching the shortcut layer at all — that instruction is written into the dispatch payload.

## Blockers

- None for this turn. Slices 1 and 2 are moving; 3 and 4 are correctly blocked on referents
  that do not exist yet.
- The keyboard decision above is the only thing waiting on a human, and it does not block
  slices 1–3.

## Artifact paths

- `docs/coordination/tasks/lyra/LYRA-2026-10-13-supervisor-intent-entry-contract-v1.md`
- `docs/coordination/tasks/lyra/2026-10-13-lyra-disp-supervisor-intent-layer-decompose-delivery.md` (this file)

## Ledger

- `ev-supervisor-intent-contract-landed` — `ArtifactCreated`, the fixed id Aegis specified,
  payload carries the `contract` field (path + commit) and explicitly records that it is an
  announcement with no `verified` field.
- `ev-rUmCOLIN1eHnrJcF4S06s`, `ev-04XbjW1sKfgr2ID_IpxJs` — `ArtifactCreated` for the two new
  dispatches, each recording why its edges are what they are.
- `ev-gCL1hItxkJxwvIaalAAp4` — `SupervisorMessage` to Aegis with the measurements, the
  placement decision, and the keyboard decision request.

---

# The three-section version for Mr. Zhang

*(Per Aegis's boundary 6 — no self-invented abbreviations, no dispatch titles.)*

**变了什么**
我写下了「你跟 SeatLoom 说一句话,它把活拆好、派对、并把理由给你」这个功能的三层契约:
一个用户故事、一条从输入到看到结果的流程(含说不清时它必须停下来问你,而不是替你猜)、
以及每张卡长什么样。同时查证了一件实事:让「一句话变成一条受治理的派单、并且记在你名下」
的那条路**已经建好并且在跑了** —— 缺的只是前门,不是机器。所以这次没有新建一个"主管席位",
而是把前门开到已有的面板上。

另外查出一个真问题:`Cmd+K` 现在被绑定了两次,按一下会同时弹出两样东西 —— 一个是正常用的
面板,另一个是 385 行的演示原型(里面的建议是写死的:永远派给 Nimbus、预算永远是同一个数)。
三个文档对同一个快捷键有三种说法,代码做的是第四种。这个我没有擅自改。

**你会看到什么**
暂时还看不到新界面 —— 后端和界面两张单现在才被放开,后面还有验证和接线两步,每一步都有
前置门,门没过就不动。等链路走完,你会在现在那个 `Cmd+K` 面板的输入框里:打一句话 → 看到一张
建议卡(派给谁、为什么派给它、预算多少、证据是哪几条,全是算出来的,不是编的)→ 点确认 →
这条活带着前置条件进账本,记在你名下。说不清的时候它会问你,不会替你选。

**需要你做什么**
一件:定一下 `Cmd+K` 归谁。我倾向它归现在那个面板(你已经在用它),那个演示原型不该再占一个
快捷键 —— 但这是你在用的键,我不擅自移。你定了之后,界面那张单才敢碰快捷键那一层。
