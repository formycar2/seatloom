# LYRA-2026-04-28-Cross-Priority-Dataflow-Map-v1

| Field | Value |
|---|---|
| Owner | Lyra |
| Status | Active baseline artifact |
| Purpose | Clarify canonical objects, projections, and flow rules before implementation |
| Contract basis | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/interaction-spec-v1.0.md`, `docs/archive/product-history/acceptance-spec-v1.0.md`, `docs/architecture-decisions.md` |

## 1. Dataflow decision

SeatLoom is a continuity system, not a view-first mockup.

That means implementation must follow this data rule:

- users act on canonical objects
- the system appends ledger events
- Inbox / Timeline / Morning Digest are projections, not primary truth
- Detail Pane and Terminal reveal canonical state plus evidence

## 2. Canonical stores vs projections

| Data type | Canonical or derived | Written by | Consumed by | Notes |
|---|---|---|---|---|
| Project metadata | Canonical | init/open/switch flows | Title bar, switcher, All Projects | Owns project path, identity, and boundaries. |
| Project UI state | Canonical | UI shell on tab/filter/selection changes | project switch restore, startup restore | Per-project memory of tab, filter, selected object, terminal visibility. |
| Seat profile | Canonical | add/edit seat flow | Sidebar, detail, handoff forms, session ownership | Stable role/accountability anchor. |
| Session | Canonical | attach, wrap, resume, recover | Sidebar, Terminal, Detail, Inbox, Timeline | Owns runtime, seat, status, workdir, recovery state. |
| WorkItem | Canonical | create/edit/progress flows | Sidebar, Detail, Inbox, Timeline, handoff | Owns goal, AC, owner, priority, lifecycle. |
| Handoff | Canonical | create/send/accept/return/complete | Inbox, Detail, Timeline, WorkItem context | Owns purpose, expected outcome, sender/receiver, artifact refs. |
| Artifact | Canonical | capture/import/proposal flows | Detail, Timeline, Handoff, WorkItem evidence | Captured evidence is first-class, not note text embedded in chat only. |
| Checkpoint | Canonical | session end/artifact trigger/timed snapshot later | recovery, LaunchPack assembly | P0 uses deterministic summaries only. |
| LaunchPack | Canonical file artifact | switch/recover/rehydrate flow | target runtime, Detail, audit trail | Must be written before injection or file-path fallback. |
| Ledger event | Canonical append-only record | every state-changing action and important system event | Timeline, Inbox, digest, audits | Source for replay and projections. |
| Inbox item | Derived projection | projection engine from objects + events | Inbox view, Detail | Never the primary writable object. |
| Timeline row | Derived projection | event label/summary formatter | Timeline view | Raw internal event names are not shown directly. |
| Morning Digest | Derived projection | reconcile/startup digest compiler | Inbox top summary / overview | Summary only; user acts through Inbox or object detail. |
| Runtime profile/capability | Canonical P0/P1 extension | runtime registry and operator choices | attach/wrap/switch/session detail | Capability truth starts at P0 execution points; richer health lands in P1. |
| Runtime health snapshot | Derived P1 projection | runtime heartbeat/ops collector | runtime-health view, status badges | Not required for baseline shell. |
| SkillPack / Seat Pack | Canonical P1 extension | operator-curated reusable pack flow | launch/rehydrate/runtime selection | Must stay explicit and inspectable. |
| Automation run | Canonical P1/P2 extension | scheduled reconcile/pipeline/health job | Timeline, Inbox, artifacts | Every automation must leave a visible trace. |
| Proposal draft | Canonical P2 extension | bounded seat autonomy | human Inbox / review flow | Used for draft handoffs or child-work proposals. |

## 3. Core projection rules

1. A projection may trigger user attention, but only canonical object mutation clears the underlying work.
2. Every successful mutation must append at least one ledger event before views refresh.
3. Inbox sorts and filters actionability; it does not invent new object state.
4. Timeline formats readable labels and summaries from canonical event data; it does not become the event source.
5. All Projects reads per-project summaries only; it does not merge project-local object graphs.

## 4. P0 flow map

| Flow | Trigger | Canonical writes | Derived projections | Success output | Failure / fallback |
|---|---|---|---|---|---|
| F0-01 Project init and switch | Open repo, init confirm, switch project | project metadata, project UI state, init event, switch event | switcher state, overview, digest, All Projects summary | Current project shell loads in the correct state and boundary | Missing path, permission denial, or switch protection prompt; never silent drop. |
| F0-02 Seat and session entry | Add seat, attach session, wrap new session | seat profile, session record, session-start event, capture refs | Sidebar seat/session sections, Terminal tabs, Timeline row | Active operator and active runtime become visible immediately | Attach unavailable -> `Launch with wrap`; wrap failure -> reason + retry. |
| F0-03 WorkItem lifecycle | Create/edit/progress WorkItem | workitem record, gate-validation events | Sidebar workitems, Detail actions, Inbox when blocked/drifted, Timeline row | Work can move from draft to done with evidence and visible gates | Missing AC/owner/evidence blocks transition with visible reason. |
| F0-04 Inbox projection and action resolution | Handoff sent, session input required, workitem blocked/drifted, pipeline failure signal | canonical object mutation plus action event | Inbox queue, badge count, Detail, Timeline row | Action queue shrinks only after the underlying object leaves pending state | Invalid action stays disabled; user sees next-step guidance. |
| F0-05 Handoff closed loop | Create/send/accept/return/complete | handoff record, linked workitem refs, receipt events | Inbox entries, Timeline rows, WorkItem detail evidence | Ownership transfer becomes traceable and reversible | Missing required fields block send; return creates visible sender-side consequence. |
| F0-06 Replay formatting | Any captured event | ledger event plus projection metadata | Timeline row, event detail, object backlinks | User reads human labels and opens linked evidence | Unknown event type falls back to generic label plus structured payload, never raw internal name alone. |
| F0-07 Rehydrate and runtime switch | Switch runtime, rebuild, explicit rehydrate | LaunchPack file, target session record, link to source session/checkpoint/handoff | Detail preview, Terminal, Timeline row | First response in the new runtime does not require re-explaining the core task | Injection failure falls back to file path + short instruction; no clipboard-pack baseline. |
| F0-08 Interrupted-session recovery | Session becomes interrupted | recovery attempt event, recovered/new session record, optional checkpoint use | Inbox/input-required when needed, Timeline, Session detail | Recovery path follows native -> rebuild -> minimal order | Each failed recovery level emits reason and next available option. |
| F0-09 Reconcile and Morning Digest | startup, pre-pipeline, manual reconcile | reconcile event, drift/block/interruption findings on canonical objects | Morning Digest, Inbox items, Overview counters, Timeline row | User sees what changed and where to act next | Reconcile failure is visible and retryable. |

## 5. Inbox action linkage table

| Object case | Inbox entry | Allowed actions | Canonical mutation | Removal condition |
|---|---|---|---|---|
| Handoff in `sent` awaiting receiver | `handoff.pending` | `Accept`, `Return` | `accepted` or `returned` handoff state + receipt event | Handoff is no longer waiting on the receiver. |
| Session in `input_required` | `input.required` | `Resolve` | user supplies input or chooses an explicit fallback, then session moves to `running`, `completed`, `suspended`, or `failed` | Session no longer requires operator input. |
| WorkItem in `blocked` | `workitem.blocked` | `Resolve` | dependency/evidence/input is added, then WorkItem moves to `ready` or `active` by explicit rule | WorkItem leaves `blocked`. |
| WorkItem in `drifted` | `drift.detected` | `Resolve`, `Dismiss` | operator reconciles mismatch, then WorkItem moves to `ready`, `active`, or `done`; `Dismiss` hides the current alert only after the mismatch is recorded as acknowledged | Drift condition no longer requires action, or the current alert has been explicitly acknowledged. |
| Pipeline exhausted retries | `pipeline.failed` | `Resolve` | operator repairs the cause and re-runs, or records a blocking consequence on a linked WorkItem/Session | No pipeline failure remains pending human action. |

Notes:

- `Dismiss` is not a universal escape hatch. In baseline it is allowed only where the underlying object can remain valid after explicit acknowledgement.
- `Resolve` is an entry verb, not one fixed state jump. The end state depends on object type and evidence supplied.

## 6. Timeline label and summary seed catalog

| Event type | UI label | Minimum summary template | Required object refs |
|---|---|---|---|
| `session.started` | Session started | `{actor} started {runtime} for {seat}` | session, seat |
| `session.interrupted` | Session interrupted | `{actor_or_system} interrupted {session} during {workitem_or_runtime}` | session |
| `session.recovered` | Session recovered | `{actor_or_system} recovered work into {new_session}` | old session, new session |
| `artifact.created` | Artifact captured | `{actor} captured {artifact_type} for {object_ref}` | artifact, linked object |
| `workitem.updated` | WorkItem updated | `{actor} changed {workitem} to {status}` | workitem |
| `handoff.sent` | Handoff sent | `{actor} sent {handoff} to {receiver} for {workitem}` | handoff, workitem |
| `handoff.accepted` | Handoff accepted | `{actor} accepted {handoff}` | handoff |
| `handoff.returned` | Handoff returned | `{actor} returned {handoff} with feedback` | handoff |
| `launchpack.generated` | LaunchPack generated | `{actor_or_system} prepared context for {target_runtime}` | launchpack, session/workitem |
| `reconcile.completed` | Reconcile completed | `{actor_or_system} found {issue_count} actionable changes` | reconcile run or affected objects |

## 7. Handoff expiry rule for baseline

The `expired` state exists in the Handoff model, but baseline behavior is intentionally narrow:

- `P0`: no automatic timer-based expiry
- `P0`: sender or PO may explicitly mark a still-`sent` handoff as `expired` after it is judged stale
- `P1`: automatic expiry may be added only after notification/automation infrastructure is formalized

This keeps the state model intact without inventing hidden timeout behavior before the team is ready.

## 8. P1 extension flow map

| Flow | Trigger | Canonical writes | Derived projections | Boundary |
|---|---|---|---|---|
| F1-01 Manual artifact intake | Operator imports screenshot/note/file | artifact record, attachment event, linked refs | Timeline row, Detail evidence section, optional Inbox follow-up | Adds evidence without changing the core continuity model. |
| F1-02 Runtime health and discovery | App scans runtimes or receives health updates | runtime profile and health snapshot | runtime-health view, launch warnings, session badges | Improves runtime trust; does not replace project-local authority. |
| F1-03 Timed checkpoints and smarter context | scheduled snapshot or oversized LaunchPack compilation | checkpoint, compression metadata, SkillPack refs | recovery choices, LaunchPack preview, Timeline row | Must remain inspectable and file-first. |
| F1-04 Automation with history | scheduled reconcile/pipeline/health run | automation-run record, linked events/artifacts | Timeline row, Inbox if human action is required | No invisible background mutation. |
| F1-05 Live Mode | user toggles live monitoring | no new canonical work object; view-state only | live-updating Timeline projection | Replay semantics remain primary. |

## 9. P2 extension flow map

| Flow | Trigger | Canonical writes | Human control point |
|---|---|---|---|
| F2-01 Autonomous artifact / handoff draft | seat proposes an artifact or drafts a handoff | proposal or draft object plus ledger event | Human accepts, edits, or rejects before promotion. |
| F2-02 Child-work proposal | seat suggests decomposition | proposed WorkItem draft linked to parent | Human promotes to real WorkItem or discards it. |
| F2-03 Scratch collaboration | operator or seat opens temporary side-channel | isolated scratch object or ephemeral thread metadata | Accepted output must still be copied into WorkItem/Handoff/Artifact evidence. |

## 10. Engineering implication

Nimbus implementation should preserve these invariants from day one:

1. object stores are canonical; projections are recomputed views
2. IDs stay stable across events, views, and cross-links
3. capability truth is modeled explicitly, not implied by UI copy
4. recovery and rehydrate always create auditable artifacts/events
5. future automation and autonomy features must enter through visible ledgered paths
