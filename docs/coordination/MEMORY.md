# MEMORY - Project Governance and Milestone Log

This file tracks major decisions, sign-offs, and state transitions for the project.

## Project Metadata

- **Sponsor:** Mr. Zhang
- **PO:** Lyra
- **Architect:** Nimbus
- **Designer:** Mira
- **Lead Engineer:** Flux

## Durable Decisions

| Date | Decision | Rationale | Owner |
| :--- | :--- | :--- | :--- |
| 2026-04-24 | Team Onboarding | Lyra/Nimbus/Mira/Flux transitioned to seatloom | Lyra |
| 2026-04-27 | Product form: Desktop app + CLI | Desktop GUI (Tauri 2 + React) as primary, CLI for scripting. Ref: Claude Code Desktop, Cherry Studio | Aegis / Mr. Zhang |
| 2026-04-27 | Tech stack: Tauri 2 + React/TS + Rust | 3-10MB bundle, 40-80MB RAM, sub-500ms startup. Electron rejected for resource overhead | Aegis / Mr. Zhang |
| 2026-04-27 | 13 MVP scenarios defined | All user-facing scenarios documented with GUI interactions in mvp-scenarios.md v2.0 | Aegis / Mr. Zhang |
| 2026-04-27 | ContextPack: pure rules, no LLM in MVP | Selector→Budgeter→Assembler→Verifier, 8192 token budget, deterministic | Aegis / Mr. Zhang |
| 2026-04-27 | Checkpoint: 2 triggers in MVP | Session end + Artifact output. Timed snapshot → P1 | Aegis / Mr. Zhang |
| 2026-04-27 | Pipeline: foreground sync execution in MVP | No daemon. GUI real-time progress. Daemon mode → P1 | Aegis / Mr. Zhang |
| 2026-04-27 | Reconciliation: 3 trigger points | App launch + pre-Pipeline + manual. No file watcher in MVP | Aegis / Mr. Zhang |
| 2026-04-27 | PRD v0.2 and v0 archived | Superseded by v0.3 | Aegis |
| 2026-04-27 | Mira role: React component prototypes | UX expressed as React/TS code for Tauri webview, not Figma | Aegis / Mr. Zhang |
| 2026-04-27 | Architecture Design v1.0 drafted | Cargo workspace, core types, Tauri IPC, PTY, storage, 4-phase plan. Pending Nimbus/Mr. Zhang review | Aegis (for Nimbus) |
| 2026-04-27 | UX Spec v1.0 drafted | Full screen specs (Sidebar, Inbox, Timeline, WorkItems, Detail Pane, Terminal, Dialogs, Pipeline), 30 components, 3-phase delivery. Pending Mr. Zhang review | Aegis (for Mira) |
| 2026-04-27 | PRD v0.4 drafted (design-first contract) | Shift from MVP feasibility narrative to pain-point closure. Added state machines, inbox rules, capability guarantees, acceptance criteria | Aegis / Mr. Zhang |
| 2026-04-27 | Acceptance Spec v1.0 drafted | Unified PO/UI/Engineering/QA checklists and Go/Hold rules for baseline gate | Aegis / Mr. Zhang |
| 2026-04-27 | Interaction Spec v1.0 drafted | Defined trigger/success/failure flows for init, attach/wrap, inbox, timeline, workitem/handoff, rehydrate, recovery | Aegis / Mr. Zhang |
| 2026-04-27 | Multi-project switching promoted to P0 | Project switcher/recent projects/state memory/switch protection/all-projects summary are required in first delivery | Aegis / Mr. Zhang |
| 2026-04-27 | File-first coordination adopted | All substantial outputs require durable writeback under `docs/coordination/`; terminal output is summary-only with artifact paths | Lyra |
| 2026-04-27 | File-first coordination protocol adopted | Durable writeback required for all key outputs. Terminal messages become summaries + artifact paths | Aegis / Lyra / Mr. Zhang |
| 2026-04-27 | Active product contract frozen for current cycle | Execution contract = `docs/archive/product-history/prd-v0.4.md` + `docs/archive/product-history/interaction-spec-v1.0.md` + `docs/archive/product-history/acceptance-spec-v1.0.md`; approved constraints = `docs/archive/product-history/mvp-scenarios.md` + `docs/architecture-decisions.md` | Lyra |
| 2026-04-27 | Team precedence rule frozen | Approved `docs/` -> latest accepted coordination artifacts -> draft artifacts -> terminal/chat never alone | Lyra |
| 2026-04-27 | Mira redesign rejected for SG-01 baseline | Acceptance verdict = `FAIL`; shell, Inbox, Timeline, Handoff, and keyboard contract gaps block Nimbus full handoff | Lyra |
| 2026-04-27 | SG-01 UI Contract Baseline placed on Hold | Four P0 contract conflicts and Mira P0 UI gaps remain unresolved in files | Lyra / Aegis |
| 2026-04-27 | AI-native workflow principles adopted (v1.0) | Need-to-know default, packet-driven execution, token-efficiency, supervisor-only global view, delta-only sync | Aegis / Lyra / Mr. Zhang |
| 2026-04-27 | Language policy frozen | Persistent artifacts in English; terminal/screen updates in Chinese | Aegis / Lyra / Mr. Zhang |
| 2026-04-27 | SG-01 worker packets reissued under AI-native rules | Mira/Nimbus/Flux packets now require micro-briefs, need-to-know scope, token budgets, delta-only sync, English artifacts, and Chinese terminal summaries | Lyra |
| 2026-04-27 | Drift signal policy clarified | Drifted WorkItems belong in Inbox; no standalone floating drift alert; only transient reconcile-summary notice is allowed after explicit reconcile flows | Lyra |
| 2026-04-27 | Mira seat migration requires checkpoint-first handoff | Before replacing a saturated seat/runtime, freeze the current seat and create one durable-source checkpoint commit that excludes generated artifacts | Lyra |
| 2026-04-28 | Flux temporarily reassigned as acting Mira seat | Mira is offline; Flux receives a constrained UI/UED contract-repair packet under Lyra supervision with no authority to redesign the product IA | Lyra |
| 2026-04-28 | Frontend demo refreshed to a Chinese high-density baseline | Core demo content now follows the real `2026-04-28` coordination narrative; `ui/src/stores/useDataStore.ts` is authoritative, `ui/src/mockData.ts` is compatibility-only, and locale fallback no longer reintroduces English | Lyra |
| 2026-04-28 | Multica benchmark selected as a product-pattern reference | SeatLoom should borrow runtime capability truth, runtime observability, execution-history evidence, visible automation, and mixed-script typography discipline, but should not adopt an issue-board center or notification-driven inbox semantics | Lyra |
| 2026-04-28 | Aegis accepted Lyra's Multica adaptation guardrails | The team is aligned on five key constraints: `claimed` as event-first, blocker reporting through existing Inbox rules, Timeline live as optional, bounded seat autonomy, and layered runtime-health delivery | Lyra / Aegis |
| 2026-04-28 | Collaboration Protocol v1.0 conditionally accepted | The operating model is approved, but full adoption is gated on alignment edits for packet filename convention, gate-decision storage path, and explicit language/channel rules | Lyra |
| 2026-04-28 | Implementation remains frozen until cross-priority product clarity is documented | P0, P1, and P2 must be explicit in user value, data flow, and scenario coverage before implementation begins, to avoid contract churn and post-start reinterpretation | Lyra |
| 2026-04-28 | Cross-priority pre-implementation baseline issued | Lyra published the P0/P1/P2 value map, dataflow map, and scenario matrix as the planning baseline for Mira/Nimbus/Flux before any implementation start | Lyra |
| 2026-04-28 | Interaction Design comprehensive review completed | 4 Critical / 6 High / 6 Medium issues identified; review filed at `docs/coordination/reviews/2026-04-28-interaction-design-review.md`; pending Lyra accept/dispute | Aegis |
| 2026-04-28 | Collaboration Protocol v1.0 drafted | Role collaboration graph, trigger-action table, 5 message contract schemas (Task/Delivery/Verification/Gate/Blocker), prohibited paths, escalation rules. Filed at `docs/coordination/COLLABORATION_PROTOCOL.md` | Aegis |
| 2026-04-28 | Process mapping review completed | Full 2-day workflow mapped to SeatLoom object model (5 seats, 11 WIs, 6 sessions, 7 handoffs, 27 artifacts). 3 product gaps found: seat delegation, supervisor rehydration, WorkItem rejection flow. Filed at `docs/coordination/reviews/2026-04-28-process-mapping-review.md` | Aegis |
| 2026-04-28 | PRD v0.5 draft issued as the next contract candidate | The new PRD integrates the five core-value modules - Data Engine, Seat three-layer architecture, Playbook system, Supervisor Layer, and Artifact review - and enforces the rule that every P0/P1/P2 feature must exist in story, interaction, and UX form before implementation | Lyra |
| 2026-04-28 | Interaction Spec v1.1 draft issued for the v0.5 contract | The interaction spec now covers supervisor-driven creation, seat identity and delegation, deterministic projection, review fail/reissue, runtime continuity, playbooks, and bounded automation proposals | Lyra |
| 2026-04-28 | UX Specification v1.1 draft issued for the v0.5 contract | The UX spec now defines the shell, Supervisor Command Bar, seat surfaces, work loop, continuity views, artifact review workspace, playbook library, and proposal tray aligned to the new v0.5 stories | Lyra |
| 2026-04-28 | PRD feature-value hard gate adopted | Every in-scope feature must now explicitly answer the user pain solved, the cost of omission, and the user-behavior change after delivery; any feature that fails this test is removed from delivery scope and blocked from implementation | Lyra / Mr. Zhang |
| 2026-04-28 | Ecosystem and retrieval inputs integrated into the v0.5 contract set | The PRD, interaction spec, and UX spec now absorb Seat Skill vs Playbook separation, Seat Card capability truth, tiered context packs, structured checkpoints, layered retrieval, budget enforcement, live progress, and execution transparency, while explicitly deferring MCP-server exposure and editable execution-template forking until separate value proof exists | Lyra |
| 2026-04-28 | Product Truth Index activated as the canonical entrypoint | All new product work must start from `docs/PRODUCT_TRUTH.md`; the active contract set is now PRD v0.5 + Interaction Spec v1.1 + UX Spec v1.1 + Acceptance Spec v1.1 + architecture support docs | Lyra |
| 2026-04-28 | Acceptance Spec v1.1 issued as the active verification contract | Acceptance is now aligned to the v0.5 contract set with gate model, story coverage, deterministic-first verification, and evidence-package requirements | Lyra |
| 2026-04-28 | Legacy product docs moved into one archive directory | Superseded PRDs, scenarios, UX/interaction baselines, positioning, validation, and Acceptance Spec v1.0 now live under `docs/archive/product-history/` to prevent parallel-source drift | Lyra |
| 2026-04-28 | Document Templates v1.0 conditionally adopted and integrated into the product contract | The seven coordination template classes now serve as first-class Artifact types in the active PRD/spec set, while repo-wide file migration remains staged | Lyra |
| 2026-04-28 | Artifact template+subtype dual-key contract adopted | The active v0.5 PRD/spec set now uses `template+subtype` as the canonical artifact classification key for Detail Pane rendering, Route/Gate automation, and evidence-search filters, with subtype allow-lists sourced from `docs/coordination/DOCUMENT_TEMPLATES.md` Section 11.1 | Lyra |
| 2026-04-28 | Artifact objectization baseline accepted | The prototype now treats seeded coordination documents as typed Artifact objects across Timeline, Inbox, and Detail, exposes dual-key metadata in detail, enforces the allow-list in code, and provides Timeline `template` / `subtype` filters; this closes the scoped baseline without lifting the broader implementation freeze | Lyra |
| 2026-04-28 | Nimbus architecture baseline alignment packet issued | Nimbus must align `docs/architecture-design.md` and `docs/architecture-decisions.md` upward to the active v0.5 contract set before broad implementation starts, using `docs/PRODUCT_TRUTH.md` as the single entrypoint and the new task packet as the only execution scope | Lyra |
| 2026-04-28 | Nimbus architecture baseline alignment delivered | All 8 known drift areas resolved across `docs/architecture-design.md` (Seat three-layer, Artifact dual-key, WorkItem Rejected/Rescoped, Data Engine module, Retrieval order, Delegation overlay) and `docs/architecture-decisions.md` (AD-004 rewritten, AD-008–AD-011 added). BLOCKER-001 (retrieval storage backend) surfaced for Lyra/Aegis decision. | Nimbus |
| 2026-04-28 | Nimbus architecture baseline alignment conditionally accepted | Lyra accepted the repaired subset but kept architecture freeze on hold because `US-P0-11` / `INT-16` / `UX-12` prompt-state architecture is still missing and durable `Rejected` / `Rescoped` WorkItem states drift from the active event-first review/reissue contract. | Lyra |
| 2026-04-28 | BLOCKER-001 resolved: SQLite FTS5 is the P0 retrieval backend | Local-first deterministic evidence search needs a persistent embedded store; PostgreSQL remains a future option and any in-memory index is cache-only. | Lyra |
| 2026-04-28 | Operator summary + direct dispatch contract frozen | Human-facing terminal updates must keep `Decision` / `Actions` / `Blockers` / `Artifact paths` in that order, and Lyra must notify the next seat directly with artifact references once the owner is known. | Lyra |
| 2026-04-28 | Module-topology follow-up synchronized into INT/UX | `US-P3-01` is now paired with `INT-P3-01` and `UX-P3-01`, making module session groups, scoped delegation overlays, and evidence-based replication review visible in the active contract set. | Lyra |
| 2026-04-29 | Nimbus prompt+retrieval delta delivered | All ABA-01–ABA-04 findings closed: AD-012 prompt architecture added, Rejected/Rescoped states removed (event-first restored), AD-011 SQLite FTS5 frozen, dual-key metadata applied. Architecture freeze lifted pending Lyra ACCEPT verdict. | Nimbus |
| 2026-04-28 | Nimbus prompt+retrieval delta accepted; Architecture Design Baseline Freeze passed | Lyra confirmed the architecture delta closes prompt-state modeling, event-first review/reissue, SQLite FTS5 retrieval freeze, and typed-delivery governance. Broad implementation remains blocked by Product Baseline Freeze and SG-01 UI hold. | Lyra |
| 2026-04-28 | Nimbus architecture doc hygiene closure accepted | The remaining low-severity architecture documentation issues are closed: `§12.4` heading restored, duplicate footer removed, and the decisions header now reflects `AD-008–AD-012`. No residual architecture-baseline follow-up remains. | Lyra |
| 2026-04-28 | Review change tier contract synchronized into INT/UX/Acceptance | The active interaction, UX, and acceptance specs now make `L1/L2/L3` review follow-up routing, the structured `change_tier_record`, and the `L2` compact-ack rule visible and testable. | Lyra |
| 2026-04-28 | SG-01 verification paused pending UI contract realignment | The active v0.5 product/interaction/UX contract has moved ahead of the current prototype; Flux is rerouted from verifier sequencing to acting-Mira UI realignment review before any new SG-01 baseline verdict or evidence run. | Lyra |
| 2026-04-28 | Nimbus foundation scaffold packet issued | With the architecture baseline already accepted, Nimbus may begin non-user-facing foundation scaffolding in code (workspace, core types, enums, event families, minimal compile-safe stubs) without waiting for SG-01 UI realignment to finish. | Lyra |
| 2026-04-28 | Mira resumed as primary UI owner; v0.5 UI realignment packet issued | Mira is back online and retakes UI ownership from Flux's temporary acting-Mira role. The next UI step is active-contract realignment and code implementation against the v0.5 truth set, not SG-01 re-verification. | Lyra |
| 2026-04-28 | Mira restart moved into staged UI code-gap scanning | After `S1`-`S3` re-established the active truth set, Lyra requires small read-only gap scans before any new v0.5 implementation slice resumes, with bounded parallelism only when explicitly authorized. | Lyra |
| 2026-04-28 | Mira shell/artifact gap scan accepted; workloop/continuity scan opened | The first read-only UI inspection slice found shell, Inbox, Timeline, and typed Artifact surfaces broadly aligned, so Lyra advanced the restart into a second scan focused on review tiers, continuity tiers, prompt-state visibility, and seat capability truth. | Lyra |
| 2026-04-28 | Mira gap scan completed; implementation resumes as micro-slices | The second scan confirmed the shared type spine is ready while Handoff, WorkItem review, Session prompt-state, and Seat capability surfaces still need UI work. Lyra restarted implementation from the smallest visible slice: the Handoff state strip. | Lyra |

| 2026-04-28 | Mira S5A handoff slice accepted | Lyra accepted Mira's first post-scan UI micro-slice: Handoff detail now shows the canonical state strip, and the latent `Working` handoff label build fix in `ui/src/utils/display.ts` is accepted as a safe exception. | Lyra |
| 2026-04-28 | Mira S5 implementation queue issued with max-3 concurrency | Lyra converted the remaining v0.5 UI realignment into queued micro-slices (`S5B`-`S5F`) with explicit file ownership, dependency order, and a hard cap of three active packets to balance throughput and Gemini stability. | Lyra |
| 2026-04-28 | Mira S5B-S5D UI slices accepted; S5E unlocked | Lyra accepted the WorkItem review tier strip, the Session prompt-blocked surfaces, and the Seat capability-truth card after local build re-verification. `S5E` is now unlocked, while `S5F` remains blocked behind the continuity preview slice. | Lyra |
| 2026-04-28 | Mira S5E continuity preview accepted; S5F unlocked | Lyra accepted the continuity-pack preview slice in `SessionDetail`, confirmed local build success, and released the final seeded-visibility patch so the repaired surfaces can be proven with realistic in-app data. | Lyra |
| 2026-04-28 | Mira S5F seeded-visibility patch conditionally accepted; light-theme rebase issued | The proof-state seeding is sufficient and the build passes, but a small Chinese-copy cleanup remains and is now folded into the next visual packet before theme acceptance can close. | Lyra |
| 2026-04-28 | Prototype theming switched to preset-system mode | Lyra rejected one-off component recoloring as the implementation method and replaced the single-palette rebase with a tokenized 3-preset theme system plus persisted in-app selection. | Lyra |
| 2026-04-28 | Mobile companion support promoted into the active v0.5 contract | PRD/Interaction/UX now treat mobile as a required companion channel for monitoring, approval/takeover, interrupt triage, and auditable short-feedback return loops, while explicitly rejecting full mobile workbench scope and keeping the requirement out of Data Engine rules. | Lyra |
| 2026-04-28 | Mira theme preset system conditionally accepted; close-out packet issued | The preset-system direction is accepted as the active visual baseline, S5F copy-hygiene is closed, and only selector naming plus persisted-value fallback remain before full closure. | Lyra |
| 2026-04-28 | Mira theme preset system fully accepted and closed | The close-out packet resolved visible preset naming and persisted-value fallback, so the preset-system baseline is now fully accepted for the prototype shell. | Lyra |

| 2026-04-28 | Mira S6 interaction-baseline closure queue issued | After theme acceptance, Lyra restarted the smallest remaining v0.5 UI closure work on default Project Overview, running-session terminal reveal, shortcut discoverability, and deterministic Inbox/WorkItems search under a max-3 concurrency rule. | Lyra |
| 2026-04-28 | Mira S6C deterministic list search accepted; S6A and S6B returned | The list-search slice is fully acceptable, but shell truth and shortcut truth still need one bounded rework before the S6 queue can close. | Lyra |
| 2026-04-28 | Mira S6AB shell truth-fix packet issued | Lyra returned fabricated Project Overview values, the missing Inbox session terminal auto-open path, and the unsupported shortcut-help entry as one serial micro-fix packet. | Lyra |
| 2026-04-28 | Acceptance Spec v1.1 synchronized to prompt and mobile P0 stories | The active acceptance contract now explicitly covers interactive prompt handling plus the mobile monitor, approval, feedback, and interrupt loops so new implementation slices can be reviewed against the same v0.5 truth set. | Lyra |
| 2026-04-28 | Mira S6AB accepted; S6 interaction-baseline queue closed | Lyra verified the truth-fix delivery, re-ran the UI build, and closed the remaining shell and shortcut gaps from the S6 queue. | Lyra |
| 2026-04-28 | Nimbus foundation scaffold delivered | Full Rust/Tauri workspace scaffold complete: 62 files, 9 seatloom-core subsystems, all AD-008–AD-012 types, AgentAdapter trait, 10 Tauri command stubs, CLI binary. Compile verification deferred (ENV-001: cargo not on Nimbus seat). | Nimbus |
| 2026-04-28 | Mira S7A mobile truth recheck failed; queue/count fix packet issued | The mobile shell direction is accepted, but `pending approvals / gates` still under-counts seed-backed action items and the urgent mobile queue still admits FYI rows. `S7B` remains blocked until the serial queue/count patch is accepted. | Lyra |
| 2026-04-28 | Nimbus foundation scaffold conditionally accepted; bounded fix packet issued | The scaffold structure is materially aligned, but `src-tauri/src/main.rs` lacks a minimal binary entrypoint and the delivery artifact uses an invalid `T3` subtype (`delivery`). Nimbus must close those two bounded items before scaffold closure. | Lyra |
| 2026-04-29 | Nimbus scaffold recheck closed the entrypoint blocker; metadata hold remains | Lyra rechecked `src-tauri/src/main.rs` and confirmed the minimal Tauri `main()` now exists. The scaffold packet stays `CONDITIONAL` because the delivery artifact still uses an invalid `T3` subtype and the required bounded-fix delivery artifact is still missing. | Lyra |
| 2026-04-29 | Mira S7A mobile queue/count alignment accepted under Lyra takeover | Mira went offline before closing the final mobile truth hold, so Lyra completed the bounded deterministic queue/count patch directly, re-ran the UI build, and closed the remaining `S7A` mobile companion gap. | Lyra |
| 2026-04-29 | Nimbus foundation scaffold close-out accepted | Nimbus normalized the scaffold delivery metadata to valid `T3/task`, published the bounded-fix delivery artifact, and fully closed the scaffold packet; `ENV-001` remains a non-blocking compile-environment note only. | Lyra / Nimbus |

| 2026-04-29 | Nimbus storage + ledger foundation packet issued | With the scaffold packet closed, Nimbus moves into deterministic project IO and Ledger primitives without widening into runtime wiring or UI work. | Lyra |
| 2026-04-29 | Flux SG-01 post-S7A verification conditionally accepted with corrected findings | Lyra accepted the build-backed verification sweep, disputed the already-closed continuity-preview and prompt-action holds, and narrowed the live SG-01 blockers to delegation overlay plus Supervisor Command Bar wiring. | Lyra / Flux |
| 2026-04-29 | Mira re-entry delta scan packet issued | Mira is back online; Lyra issued a read-only context recovery packet so the next UI slice starts from the corrected SG-01 truth instead of reopening closed surfaces. | Lyra |
| 2026-04-29 | Nimbus storage + ledger foundation accepted | Deterministic YAML/JSONL project IO and Ledger primitives are accepted as scope-complete; real Rust compile verification remains a separate `ENV-001` follow-up. | Lyra |
| 2026-04-29 | Mira re-entry delta scan accepted; SG-01 closure narrowed to two UI packets | Accepted shell, continuity, prompt, theme, and mobile surfaces stay closed; only Supervisor Command Bar and scoped delegation overlay remain for `SG-01` UI closure. | Lyra |
| 2026-04-29 | Mira SG-01 closure packets issued | `S7B` Supervisor Command Bar and `S7C` scoped delegation overlay were issued with disjoint write sets and a max-2 active-packet rule. | Lyra |
| 2026-04-29 | Mira S7B Supervisor Command Bar conditionally accepted | Core command-bar behavior is accepted and the build is green; only recent-command focus disclosure and `Enter`-to-confirm remain for bounded close-out. | Lyra |
| 2026-04-29 | Mira S7C scoped delegation overlay conditionally accepted | Delegation visibility is accepted across WorkItem, Seat, and Timeline surfaces; only explicit issuer capture and routable-owner gating remain for bounded close-out. | Lyra |
| 2026-04-29 | Mira SG-01 UI close-out packet issued | The last SG-01 UI hold is constrained to four fixes only: command-bar recents, keyboard confirm, delegation issuer capture, and owner-safe delegation gating. | Lyra |
| 2026-04-29 | Nimbus seat registry + delegation storage packet issued | The next engineering slice is limited to durable seat identity, project role binding, and scoped delegation persistence on top of the accepted storage foundation. | Lyra |
| 2026-04-29 | Baseline commit frozen before frontend refactor | Commit `1bf60be` (`chore: baseline commit before frontend refactoring`) preserves the accepted pre-refactor product baseline before the new UI direction is implemented. | Lyra |
| 2026-04-29 | Frontend refactor ownership transferred to Aegis + Mr. Zhang; Mira paused | Lyra no longer drives frontend reimplementation packets during the refactor phase; Mira remains on hold for later polish/alignment after the new baseline lands. | Lyra / Aegis / Mr. Zhang |
| 2026-04-29 | Nimbus engineering lane re-scoped to infrastructure-only | Until the team finishes the next value/usability pass on the redesign, Nimbus should strengthen toolchain, verification, CI, and other foundation concerns only; artifact read-model feature work is paused. | Lyra |
| 2026-04-29 | Mira S7C scoped delegation overlay accepted | The close-out recheck confirms explicit issuer capture, owner-safe overlay gating, and green build status, fully closing the delegation slice. | Lyra |
| 2026-04-29 | Mira SG-01 close-out recheck reduced the hold to one keyboard gap | Recent intents, explicit issuer capture, and owner-safe delegation gating are now accepted; only proposal-surface `Enter` confirm remains unresolved in the Supervisor Command Bar. | Lyra |
| 2026-04-29 | Mira S7B enter-confirm hotfix packet issued | The final SG-01 UI task is narrowed to a single-file keyboard fix in `SupervisorCommandBar.tsx`, with no other surface reopen allowed. | Lyra |
| 2026-04-29 | Mira S7B enter-confirm hotfix accepted | Lyra rechecked the bounded keyboard fix, confirmed `Enter` now truthfully confirms the live Supervisor proposal surface, and reran `cd ui && pnpm build` successfully. | Lyra |
| 2026-04-29 | SG-01 UI Contract Baseline passed | The accepted S7B hotfix closes the last UI hold; SG-01 is now `GO` and no longer blocks the shared prototype baseline. Product Baseline Freeze and later stage-gate review remain separate governance items. | Lyra |
| 2026-04-29 | Flux final v0.5 UI evidence packet issued | With `SG-01` now closed, Flux is reactivated in read-only mode to publish the clean evidence pack for the accepted UI baseline without reopening any accepted surface. | Lyra |
| 2026-04-29 | Flux final v0.5 UI evidence packet accepted | Flux published a clean 10/10 PASS evidence pack against the accepted `SG-01` baseline; Lyra re-ran `cd ui && pnpm build` plus `cd ui && npx tsc --noEmit`, and no accepted UI surface was reopened. | Lyra / Flux |
| 2026-04-29 | Nimbus seat registry + delegation storage accepted | Deterministic seat identity, project role binding, and scoped delegation persistence are accepted as scope-complete; `ENV-001` remains a separate compile-environment follow-up only. | Lyra / Nimbus |
| 2026-04-29 | Product Baseline Freeze stage-review request issued to Aegis | With UI and bounded storage baselines accepted, Lyra has triggered the next Aegis stage review to decide `Product Baseline Freeze`, `Nimbus Implementation Handoff`, and `ENV-001` routing. | Lyra / Aegis |
| 2026-04-29 | Flux UI visual theme + hover audit accepted | Flux delivered a fix-ready visual QA packet covering all three theme presets with five `P1` defects and eleven `P2` follow-ups; Lyra accepts it as the active evidence base for the next bounded UI styling repair cycle. | Lyra / Flux |
| 2026-04-29 | Product Baseline Freeze closed at GO | Aegis's stage review confirmed the active truth set, accepted UI baseline, and accepted storage slices are sufficient for freeze closure; `ENV-001` remains a separate compile-verification track. | Lyra / Aegis |
| 2026-04-29 | Mira P1 theme + hover fixes packet issued | The next UI cycle is bounded to Flux's five accepted `P1` defects only, with no broad palette redesign, no `P2` polish, and no repo-wide token migration. | Lyra / Mira |
| 2026-04-29 | Mira P1 theme + hover fixes accepted | Lyra confirmed the five bounded `P1` visual defects are closed in the declared files, the shared UI build remains green, and the deferred `P2` list stays untouched. | Lyra / Mira |
| 2026-04-29 | ENV-001 compile verification executed and conditionally accepted | Aegis executed the Flux verification packet on a Rust-capable fallback seat: `seatloom-core` compiles clean and all 9 tests pass; `seatloom-tauri` fails due to missing `src-tauri/icons/icon.png` (build-time Tauri asset, not code error). `ENV-001` is `RE-SCOPED` to the icon-asset gap only. | Aegis |
| 2026-04-29 | Nimbus ENV-001 Tauri icon fix packet issued | Bounded fix packet to provide the missing icon asset so workspace-level `cargo check` passes and `ENV-001` can be fully closed. | Aegis |
| 2026-04-29 | Flux P1 theme + hover verification packet issued | Flux is reactivated in read-only mode to verify that Mira's accepted `P1` repair packet truly closes the hover/theme defects without direct regressions. | Lyra / Flux |

| 2026-04-29 | Flux Rust foundation hardening verification accepted | Lyra accepted Flux's independent read-only verification after matching local reruns of the full 5-command Rust gate. The packet now serves as valid evidence for Nimbus hardening closure. | Lyra / Flux |
| 2026-04-29 | Nimbus Rust foundation hardening accepted | The exact `1.95.0` toolchain freeze, portable local verification script, CI parity, fmt/clippy closure, and clean warning-free foundation gate are now accepted; the earlier `foundation-quality-automation` conditional hold is resolved without resuming business-facing backend work. | Lyra / Nimbus |

| 2026-04-29 | Commit-pinned infrastructure verification rule activated | Infrastructure acceptance must verify an exact Git commit on a capable seat; Flux verification and any bounded fix follow-ups must stay commit-traceable on-branch | Lyra |
| 2026-04-29 | Real collaboration PostgreSQL baseline accepted | Remote Docker-backed verification on the sponsor workspace passed at exact commit `a658086b54323259fda2ad2a958d097701f1fbbd`; the PG16 seed blocker is closed and all 9 DB tests pass | Lyra / Nimbus / Flux |
| 2026-04-29 | Sponsor workspace SSH tunnel note promoted to durable infra ops doc | `docs/infra/ssh-tunnel-workspace.md` records the tmux tunnel, ports, and host details for PostgreSQL/frontend inspection against the sponsor workspace | Flux / Lyra |

## Milestone Status

- [x] PRD v0.3 Audit & Alignment (2026-04-27, Aegis)
- [x] Architecture Design v1.0 Draft (2026-04-27, Aegis for Nimbus)
- [x] PRD v0.4 Draft (2026-04-27, Aegis for product-contract baseline)
- [x] Acceptance Spec v1.0 Draft (2026-04-27, Aegis)
- [x] Interaction Spec v1.0 Draft (2026-04-27, Aegis)
- [x] File-First Coordination Adoption (2026-04-27, Lyra)
- [x] Product Contract Alignment Review (2026-04-27, Lyra)
- [x] Mira UI Acceptance Review v1 Issued - FAIL / NO-GO (2026-04-27, Lyra)
- [x] SG-01 Recovery Plan Issued (2026-04-27, Lyra)
- [x] SG-01 Role Task Packets Issued (2026-04-27, Lyra)
- [x] SG-01 Role Task Packets Reissued - AI-native v2 (2026-04-27, Lyra)
- [x] Flux Acting-Mira Contract Repair Packet Issued (2026-04-28, Lyra)
- [x] Frontend Demo Chinese High-Density Refresh (2026-04-28, Lyra)
- [x] Multica Product Benchmark Issued (2026-04-28, Lyra)
- [x] Multica Benchmark Alignment Accepted by Aegis (2026-04-28, Lyra / Aegis)
- [x] Collaboration Protocol v1.0 Review Issued - Conditional Adopt (2026-04-28, Lyra)
- [x] Pre-Implementation Design Freeze Decision Issued (2026-04-28, Lyra)
- [x] Cross-Priority Value/Dataflow/Scenario Baseline (2026-04-28, Lyra)
- [x] PRD v0.5 Draft - Core Value Expansion (2026-04-28, Lyra)
- [x] PRD v0.5 Value-Gate Reinforcement (2026-04-28, Lyra)
- [x] PRD v0.5 Ecosystem + Retrieval Integration (2026-04-28, Lyra)
- [x] Interaction Spec v1.1 Draft (2026-04-28, Lyra)
- [x] UX Specification v1.1 Draft (2026-04-28, Lyra)
- [x] Product Truth Index Activated (2026-04-28, Lyra)
- [x] Acceptance Spec v1.1 Draft - Active Verification Contract (2026-04-28, Lyra)
- [x] Legacy Product Docs Unified Archive (2026-04-28, Lyra)
- [x] Typed Coordination Artifact Contract Integration (2026-04-28, Lyra)
- [x] Artifact Template+Subtype Dual-Key Contract Alignment (2026-04-28, Lyra)
- [x] Artifact Objectization Baseline Accepted (2026-04-28, Lyra)
- [x] Nimbus Architecture Baseline Alignment Packet Issued (2026-04-28, Lyra)
- [x] BLOCKER-001 Retrieval Backend Decision - SQLite FTS5 (2026-04-28, Lyra)
- [x] Operator Summary + Direct Dispatch Contract Frozen (2026-04-28, Lyra)
- [x] Module Topology INT/UX Contract Sync (2026-04-28, Lyra)
- [x] Architecture Design Baseline Freeze (2026-04-28, Lyra)
- [x] Review Change Tier RCT Contract Sync (2026-04-28, Lyra)
- [x] Nimbus Foundation Scaffold Packet Issued (2026-04-28, Lyra)
- [x] Mira v0.5 UI Realignment Packet Issued (2026-04-28, Lyra)
- [x] Mira S5A Handoff State Strip Accepted (2026-04-28, Lyra / Mira)
- [x] Mira S5 Queue Issued - Max 3 Active Micro-Slices (2026-04-28, Lyra)
- [x] Mira S5F Seeded Visibility Patch Close-out (2026-04-28, closed inside Theme Preset System packet)
- [x] Mira Theme Preset System Rebase (2026-04-28, accepted after close-out)
- [x] Mobile Companion Contract Alignment in PRD / INT / UX (2026-04-28, Lyra)
- [x] Acceptance Spec v1.1 Prompt + Mobile Coverage Sync (2026-04-28, Lyra)
- [x] Mira S6 Interaction Baseline Closure Queue Issued (2026-04-28, Lyra)
- [x] Mira S6C Deterministic List Search Accepted (2026-04-28, Lyra / Mira)
- [x] Mira S6AB Shell Truth Fixes (2026-04-28, Lyra / Mira)
- [x] Mira S7A Mobile Overview + Inbox Companion Packet Issued (2026-04-28, Lyra)
- [x] Nimbus Foundation Scaffold Delivered (2026-04-28, Nimbus)
- [x] Mira S7A Monitor Truth Recheck - FAIL / HOLD (2026-04-28, Lyra)
- [x] Mira S7A Mobile Queue + Count Alignment Packet Issued (2026-04-28, Lyra)
- [x] Nimbus Foundation Scaffold Acceptance Issued - CONDITIONAL (2026-04-28, Lyra)
- [x] Nimbus Foundation Scaffold Entrypoint + Metadata Fix Packet Issued (2026-04-28, Lyra)
- [x] Nimbus Foundation Scaffold Entrypoint Recheck - Binary Gap Closed (2026-04-29, Lyra / Nimbus)
- [x] Nimbus Foundation Scaffold Metadata Close-out (2026-04-29, Lyra / Nimbus)
- [x] Mira S7A Mobile Queue + Count Recheck (2026-04-29, Lyra takeover / Mira packet close-out)
- [x] Nimbus Storage + Ledger Foundation Packet Issued (2026-04-29, Lyra)
- [x] Flux SG-01 Post-S7A Verification Reviewed - CONDITIONAL PASS / HOLD (2026-04-29, Lyra / Flux)
- [x] Mira Re-entry Delta Scan Packet Issued (2026-04-29, Lyra)
- [x] Nimbus Storage + Ledger Foundation Accepted (2026-04-29, Lyra / Nimbus)
- [x] Mira Re-entry Delta Scan Accepted (2026-04-29, Lyra / Mira)
- [x] Mira S7B Supervisor Command Bar Packet Issued (2026-04-29, Lyra)
- [x] Mira S7C Scoped Delegation Overlay Packet Issued (2026-04-29, Lyra)
- [x] Nimbus Seat Registry + Delegation Storage Packet Issued (2026-04-29, Lyra)
- [x] Mira S7B Supervisor Command Bar Acceptance Issued - CONDITIONAL (2026-04-29, Lyra / Mira)
- [x] Mira S7C Scoped Delegation Overlay Acceptance Issued - CONDITIONAL (2026-04-29, Lyra / Mira)
- [x] Mira SG-01 UI Close-out Packet Issued (2026-04-29, Lyra)
- [x] Mira S7C Scoped Delegation Overlay Accepted - PASS (2026-04-29, Lyra / Mira)
- [x] Mira SG-01 UI Close-out Re-reviewed - CONDITIONAL / one remaining keyboard hold (2026-04-29, Lyra / Mira)
- [x] Mira S7B Enter Confirm Hotfix Packet Issued (2026-04-29, Lyra)
- [x] Mira S7B Enter Confirm Hotfix Accepted - PASS (2026-04-29, Lyra / Mira)
- [x] SG-01 UI Contract Baseline (2026-04-29, Lyra)
- [x] Product Baseline Freeze Closed - GO (2026-04-29, Lyra / Aegis)
- [x] Nimbus Implementation Handoff Opened (2026-04-29, Lyra / Nimbus)
- [x] Flux Final v0.5 UI Evidence Packet Issued (2026-04-29, Lyra)
- [x] Flux Final v0.5 UI Evidence Packet Accepted (2026-04-29, Lyra / Flux)
- [x] Nimbus Seat Registry + Delegation Storage Accepted (2026-04-29, Lyra / Nimbus)
- [x] Aegis Product Baseline Freeze Stage Review Requested (2026-04-29, Lyra)
- [x] Flux Evidence Pack Ready (2026-04-29, Lyra / Flux)
- [x] Flux UI Visual Theme + Hover Audit Accepted (2026-04-29, Lyra / Flux)
- [x] Mira P1 Theme + Hover Fix Packet Issued (2026-04-29, Lyra)
- [x] Mira P1 Theme + Hover Fixes Accepted (2026-04-29, Lyra / Mira)
- [x] ENV-001 Compile Verification Executed - RE-SCOPED (2026-04-29, Aegis)
- [x] ENV-001 Full Closure (2026-04-29, Lyra / Nimbus)
- [x] Flux P1 Theme + Hover Verification Packet Issued (2026-04-29, Lyra / Flux)
- [x] Flux P1 Theme + Hover Verification Packet Dispatched (2026-04-29, Lyra / Flux)
- [x] Mira UI/UED/UX Design Brief Packet Issued + Dispatched (2026-04-29, Lyra / Mira)
- [x] Nimbus Read-Model Repositories Packet Issued + Dispatched (2026-04-29, Lyra / Nimbus)
- [x] Flux P1 Theme + Hover Verification Accepted - PASS (2026-04-29, Lyra / Flux)
- [x] Mira UI/UED/UX Design Brief Accepted - CONDITIONAL PASS (2026-04-29, Lyra / Mira)
- [x] Mira Packet A Interaction Affordance + Disabled-State Semantics Issued + Dispatched (2026-04-29, Lyra / Mira)
- [x] Mira Packet B Token Discipline + Typography + Truthful Budget UI Queued (2026-04-29, Lyra / Mira)
- [x] Mira Packet A Interaction Affordance + Disabled-State Semantics Accepted - PASS (2026-04-29, Lyra / Mira)
- [x] Flux Packet A Affordance Verification Issued + Dispatched (2026-04-29, Lyra / Flux)
- [x] Nimbus Read-Model Repositories Accepted - PASS (2026-04-29, Lyra / Nimbus)
- [x] Nimbus Artifact Read Models + Dual-Key Filters Packet Issued + Dispatched (2026-04-29, Lyra / Nimbus)
- [x] Baseline Commit Before Frontend Refactor (`1bf60be`) (2026-04-29, Lyra)
- [x] Frontend Refactor Ownership Transfer to Aegis + Mr. Zhang; Mira Paused (2026-04-29, Lyra / Aegis / Mr. Zhang)
- [x] Nimbus Infrastructure-Only Lane Reset + Foundation Quality Automation Packet Issued (2026-04-29, Lyra / Nimbus)
- [x] Nimbus Foundation Quality Automation Acceptance Issued - CONDITIONAL PASS (2026-04-29, Lyra / Nimbus)
- [x] Nimbus Rust Foundation Hardening Packet Issued + Dispatched (2026-04-29, Lyra / Nimbus)
- [x] Flux Rust Foundation Hardening Verification Packet Issued + Dispatched (2026-04-29, Lyra / Flux)
- [x] Aegis Frontend Refactor Phase 1 Progress Logged (2026-04-29, Lyra / Aegis)
- [x] Flux Rust Foundation Hardening Verification Accepted - PASS (2026-04-29, Lyra / Flux)
- [x] Nimbus Rust Foundation Hardening Accepted - PASS (2026-04-29, Lyra / Nimbus)
- [x] Commit-Pinned Infrastructure Baseline Published (`track/infra-foundation` @ `d007721`, advanced to `a658086`) (2026-04-29, Lyra / Nimbus)
- [x] Nimbus Real Collaboration PostgreSQL Baseline Accepted - PASS (2026-04-29, Lyra / Nimbus / Flux)
- [x] SG-A §A1 (Nimbus V1 white-screen diagnosis) **ACCEPTED PASS (no-fix)** — Flux closure confirms Layer A+B PASS, Mr. Zhang live-run clean (2026-05-09, Lyra / Nimbus / Flux)
- [x] SG-A v01 (Mira sessions live wiring §A/§B/§C) **ACCEPTED PASS** — deliverables 578ff7c / 4651bb4 / 2f83624, factual-record takeover e864392; Flux Layer A+B PASS, Mr. Zhang live-run incl. podman stop/start error-horizontal recovery (314 scanned) (2026-05-09, Lyra / Mira / Aegis / Flux)
- [x] SG-A §A4-α (Mira SessionsWorkspace mock-first attach UI) **ACCEPTED PASS** — delivery bdac54b, Flux Layer A+B PASS (code-verified via Vite)
- [x] SG-A §A2 (Mira NavRail Logo + ProjectSwitcher Portal) **ACCEPTED PASS** — delivery 3b7ac17, Flux Layer A+B PASS (code-verified via Vite)
- [x] SG-A §A3 (Nimbus tmux pipe-pane attach read path) **ACCEPTED UNCONDITIONAL PASS** — acceptance doc `docs/coordination/acceptance/LYRA-2026-05-09-nimbus-a3-tmux-attach-read-path-acceptance-v1.md` against commit `fca4fe0`; promoted from PROVISIONAL on 2026-05-09 evening when (a) Flux written delivery doc landed at `FLUX-2026-05-09-a3-tmux-mirror-verification-delivery-v1.md` with 13/13 static-invariant matrix PASS (R3 line-pinned at `mod.rs:356`) and (b) SG-A closure doc `FLUX-2026-05-09-sg-a-stage-gate-closure-v1.md` cleared transitive UI-integration chain via A4-α + Mr. Zhang live-run. Lyra spot-checked A.1–A.9 — no deviation from Aegis interim relay (2026-05-09, Lyra / Nimbus / Aegis / Flux)
- [x] SG-A Hotfixes (Aegis) c43c9af + 4793d04 — React infinite-loop fixes (SupervisorPanel FALLBACK module-scope + DagWorkflow useCallback+shallow-equal); Flux Layer A+B PASS
- [x] **SG-A STAGE-GATE UNCONDITIONAL PASS** — Flux closure doc `docs/coordination/tasks/flux/FLUX-2026-05-09-sg-a-stage-gate-closure-v1.md` records 11 commits verified, 20/20 Layer B runtime observations PASS, Mr. Zhang live-run complete. **SG-B unlocked.** (2026-05-09, Aegis directive / Flux verification)
- [x] SG-A §A4-β (Mira real-API wire-up) **ACCEPTED — UNCONDITIONAL PASS (Layer A + Layer B)** 2026-05-09 late-evening @ commit `ab672e5`. Three-layer gate cleared: Mira delivery (3 files, 56/-42, tsc=0, build green) → Lyra pre-flight clean (scope + wire shape verified against Rust source) → Flux Layer A verify PASS (13/13 static invariants, 5/5 R-rules; `docs/coordination/tasks/flux/FLUX-2026-05-09-a4b-sessions-real-api-wire-verify-delivery-v1.md`) → Mr. Zhang Layer B live-run PASS relayed by Aegis (cmd_list_tmux_sessions returned 5 real sessions; cmd_attach_tmux_session returned `{id, tmuxSessionName, fifoPath, runtime:'tmux-mirror'}` DTO). Lyra acceptance at `docs/coordination/acceptance/2026-05-09-lyra-mira-a4b-sessions-real-api-wire-acceptance.md`. SG-A UI surface now end-to-end real — no mocks, no deferred items for A4-β. Three carry-forwards logged as non-blockers (closeTab mirror leak → B1 / cleanup; unused optional fields reserved for seat attribution; lockfile clean). (2026-05-09, Lyra / Mira / Flux / Aegis / Mr. Zhang)
- [~] SG-B §B1 (Nimbus tmux send-keys bidirectional write path) **DISPATCHED** 2026-05-09 late-evening on A4-β acceptance. Packet `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-b1-pty-write-send-keys-v1.md` flipped status: draft → dispatched. Scope unchanged (5 files: pty/mod.rs + session_cmds.rs + tmux_mirror_smoke.rs + SessionTerminal.tsx + SessionsWorkspace.tsx). Lyra binds behavior (arbitrary bytes incl. NUL/Ctrl-C must round-trip, R3 must still print 'R3 OK' at smoke Step 12); Nimbus owns argv-vs-buffer-path decision with round-trip evidence. Pre-flagged coupling: if joint supplement review elevates 006 multi-project schema ahead of B1 landing, PtySession construction may need project_id threading — Nimbus will hold + flag rather than wire schema changes silently. (2026-05-09, Lyra / Nimbus)
- [~] **Joint review UNBLOCKED** 2026-05-09 late-evening: Lyra product supplement (`3ccbd6d`) × Nimbus arch supplement (`dd8758c`) cross-point review now runnable in parallel with B1. Aegis 2026-05-09 ruling held the gate at A4-β close; A4-β is now closed. Cross-points to align: schema 008 vs 006 numbering; realtime tier split; multi-project project_id migration timing. Lyra coordinates with Aegis on kickoff. (2026-05-09, Lyra / Aegis / Nimbus)
- [x] **Orphaned Tauri capabilities fix RECOVERED** 2026-05-09 late-evening by Aegis: commit `a329f5b` re-lands the capability config on `track/infra-foundation` with Mr. Zhang's original authorship + commit message preserved (the lost `f9b12d4` body). Sibling fix `3de1dbd` adds `withGlobalTauri` for DevTools IPC testing (tauri.conf.json, 1 file / 2-1). Both pushed to origin. Working-tree drift on `src-tauri/gen/schemas/capabilities.json` is a Tauri build-regenerated file — left uncommitted by Aegis intentionally (regenerated from `src-tauri/capabilities/default.json` source at build time). Tauri 2 `core:event:listen` + `core:window:*` permissions now permanently recorded on branch; frontend `event.listen` subscriptions (session:output, canonical:appended) are unblocked at runtime. (2026-05-09, Aegis / Mr. Zhang)
- [x] Aegis joint-review ruling 2026-05-09 late-evening (multi-stage):
  - **Decision 1**: Migration numbering binding — 006 `plan_mode_authority` → 007 `seats_budget` → 008 `project_isolation`. No insertion. Lyra amended product supplement §3.3.1 + §7.3 + §8 in follow-up commit.
  - **Decision 2 (CORRECTED)**: Initial ruling said "008 standalone before B1; B1 hold" — Nimbus correctly flagged that B1 bytes-only scope writes zero `canonical_events` rows, so 008's `project_id` column is not a B1 dependency. Lyra cross-point (c) was factually wrong; retracted. **Final ruling**: B1 unblocked + 008 dispatched in parallel, both target v0.0.2 milestone, 008 lands before B2 onward (which does write canonical_events). B1 packet status restored draft → dispatched; 008 packet reframed parallel-not-blocking.
  - **Decision 3**: Lyra product §1.3 framing — "FS = artifact truth, PG = projection authority" — ratified as **BINDING** architectural principle for v0.1+. Pending inline ratification edit in product supplement.
  - **Decision 4**: 7 amendment items + 1 open question consolidated into single adjudication doc to avoid fragmentation.
  - Realtime tier mapping (b) — Lyra self-handles framing note in product §2 (T-Live = user-perceived ≤1s budget, 16ms throttle = implementation), no Aegis adjudication needed.
- [~] **Schema 008 dispatched (parallel with B1, not blocking)** — `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-schema-008-project-isolation-v1.md`. Owner: Nimbus. Scope: `infra/postgres/schema/008_project_isolation.sql` adds `project_id TEXT NOT NULL REFERENCES projects(id)` to workitems / sessions / handoffs / canonical_events with conservative `'seatloom'` backfill heuristic; repo + DTO + Tauri command surface updates; idempotent migration; ~600-900 LOC per arch §3. Backfill verification queries (NULL count must be zero; GROUP BY must show only `'seatloom'`) embedded in §2.3. Lands in v0.0.2 before B2 onward (which write canonical_events). Does NOT gate B1 acceptance per Aegis correction. (2026-05-09 late-evening, Lyra → Nimbus)
- [~] **B1 RESTORED dispatched** (status: draft → dispatched, after brief held-then-released cycle) — Aegis correction confirmed B1 bytes-only writes zero events. B1 runs parallel with 008. Original scope unchanged (5 files: pty/mod.rs + session_cmds.rs + tmux_mirror_smoke.rs + SessionTerminal.tsx + SessionsWorkspace.tsx). Nimbus cleared to resume implementation. (2026-05-09 late-evening, Lyra / Nimbus)
- [~] **B1 DELIVERED** @ commit `0c0f425` (2026-05-09 late-evening, post-correction). Nimbus shipped 5 files / +209/-30: pty/mod.rs (real `PtySession::write` via `tmux load-buffer -b seatloom - ; paste-buffer -b seatloom -t <target>` single invocation, `;`-chained atomicity = no Mutex needed, NUL-byte fidelity via stdin), session_cmds.rs (cmd_pty_write + cmd_pty_write_bytes wired), tmux_mirror_smoke.rs (Steps 10-12 added: write round-trip + Ctrl-C SIGINT + R3 reassertion), SessionTerminal.tsx (disableStdin removed, cursorBlink on, term.onData → api.ptyWriteBytes via TextEncoder), SessionsWorkspace.tsx (banner amber→blue, pill v0.0.1→v0.0.2 with attach-only language preserved per R1). All 9 self-acceptance criteria line-pinned in delivery §4. Self-checks: cargo check core+tauri green, cargo test pty:: 6/6 (4 existing + 2 new write_noop_*), pnpm tsc 0 errors, pnpm build success, smoke 12/12 PASS (SMOKE_PASS + SMOKE_B1_PASS + R3 OK reasserted post-write). Two flag-ups: (a) cargo clippy -D warnings fails on 3 pre-existing doc_lazy_continuation lints in db/repositories.rs (validated NOT a B1 regression via stash test), (b) stale 008 packet file caught + unstaged before push (final commit 5 files exact). Architectural choice (buffer path over argv send-keys) rationale in delivery §3 with round-trip evidence. Delivery doc commit `b1d4b4b`. (2026-05-09 late-evening, Lyra ← Nimbus)
- [~] **Flux Layer A verify dispatched for B1** — `docs/coordination/tasks/flux/FLUX-2026-05-09-b1-tmux-write-path-verify-v1.md`. 13-row static invariant matrix + 5-row R-rule matrix + smoke reproduction. Independent stash test for clippy non-regression confirmation requested. Layer B (Mr. Zhang runtime) rolls into Lyra acceptance window same as A4-β pattern. (2026-05-09 late-evening, Lyra → Flux)
- [x] **Aegis consolidation adjudication COMPLETE** at commit `ebc0e68`. Seven verdicts:
  - §A.1 APPROVED (already applied)
  - §A.2 APPROVED + micro-correction (remove "before B1 enters Flux verify" stale clause)
  - §A.3 MODIFY (Option C event-first → PROVISIONAL not BINDING; needs own design packet)
  - §A.4 APPROVED verbatim (§1.3 BINDING ratified)
  - §A.5 APPROVED with per-section tags: §1.3 BINDING / §1.4 PROVISIONAL / §1.5 BINDING / §1.6 PROVISIONAL
  - §A.6 APPROVED (Nimbus applies arch §3 B1-parallel note himself)
  - §A.7 MODIFY (strike sprint-pattern self-critique; Aegis provided final retraction wording)
  - Open question RESOLVED (Nimbus reciprocal already filed at `b916912`; N1-N10 cross-points logged in product supplement §Review)
  Lyra applied §A.1/A.2/A.3/A.4/A.5/A.7 amendments inline; Nimbus applies §A.6 himself. (2026-05-11, Aegis adjudicated → Lyra/Nimbus applying)
- [x] **B1 ACCEPTED PROVISIONAL** at `0c0f425` — Layer A PASS (Flux 13/13 static + 5/5 R-rules + tsc/build green; cargo/smoke STATIC-VERIFIED with line-anchored confirmation). Acceptance doc at `docs/coordination/acceptance/2026-05-09-lyra-nimbus-b1-tmux-write-path-acceptance-v1.md`. Three-layer evidence chain solid. PROVISIONAL pending Mr. Zhang Layer B (6-step bidirectional smoke per acceptance §3); promotes to UNCONDITIONAL on Layer B PASS. SG-B entry on track for closure. (2026-05-11, Lyra / Nimbus / Flux)
- [~] **008 DELIVERED** at `526d25a` — Nimbus shipped 12 files / +308/-22. Migration applied + idempotent re-run proven against seatloom-postgres (PG 16): backfill counts 6/5/3/35 (workitems/sessions/handoffs/canonical_events), all 4 NULL-counts zero, all 4 GROUP BYs scoped to `seatloom`. cargo test 60/60. Three §10 pre-impl corrections applied (event_object_refs columns ref_type/ref_id; ref_type=seat only emitted today; ADD COLUMN IF NOT EXISTS supported PG 9.6+). Four new project-mode commands added (`cmd_list_*_for_project` × 4); existing un-scoped commands preserved for AD-013 v2 Global mode. B1 file scope untouched. Two flag-ups: (a) clippy doc_lazy_continuation pre-existing same as B1, validated NOT a 008 regression via stash-check; (b) future create-project UI must INSERT projects row before any FK-bearing reference, out of 008 scope. Delivery doc at commit `f681251`. (2026-05-11, Lyra ← Nimbus)
- [~] **Flux Layer A verify dispatched for 008** — `docs/coordination/tasks/flux/FLUX-2026-05-09-schema-008-project-isolation-verify-v1.md`. 15-row static invariant matrix + 4-row R-rule matrix + migration apply + idempotent re-run + backfill counts reproduction + clippy stash-test for non-regression. Parallel with B1 verify; independent acceptance windows. (2026-05-11, Lyra → Flux)
- [x] Aegis ruling 2026-05-09 evening: (1) No retroactive packet for SG-A hotfixes c43c9af + 4793d04 — commit messages are sufficient record. (2) Joint review of Lyra product supplement (3ccbd6d) + Nimbus arch supplement (dd8758c) deferred until A4-β closes, to avoid fragmented review. (Aegis / Lyra)
- [x] Product Design Supplement issued — `docs/coordination/reviews/2026-05-09-lyra-seatloom-full-product-design-v1.md` @ `3ccbd6d` (pushed to origin). T4 design_proposal extending PRD v0.5 (not superseding). Five sections per Mr. Zhang directive: data lifecycle (recommends event-first projection Option C), real-time tiers (T-Live ≤1s / T-Near ≤5s / T-Batch manual), multi-project (verified schema 10 tables missing `project_id`; schema 006 ALTER recommended), historical back-fill (synthetic=true flag on retroactive events), L1 concrete flows (Supervisor IM + plan-mode + WorkflowPanorama). AD-007 supersession noted. Five open questions in §7 flagged for Aegis sign-off. Hand-off to Nimbus technical-architecture supplement defined in §8. (2026-05-09, Lyra / Aegis pending review)
- [ ] Nimbus Technical Architecture Supplement issued at `docs/coordination/reviews/2026-05-09-nimbus-seatloom-full-arch-design-v1.md` (committed at `dd8758c`); 9 sections covering data flow PTY → canonical_events, file watcher, multi-project schema strategy (recommends migration 008), realtime PG LISTEN/NOTIFY, historical back-fill (recommends hybrid bootstrap + steady-state), A3 known limits, sequencing. Cross-link with Lyra product supplement §3/§4 noted; one timing question (Nimbus 008 vs Lyra 006 schema number) flagged for Aegis. Pending Aegis joint review with Lyra supplement. (2026-05-09, Nimbus / Aegis pending)
- [ ] Process correction 2026-05-09: Flux commit-pinned verify is a mandatory layer between delivery and Lyra acceptance; Aegis pre-flight review does not substitute. Sequence: `delivery → Aegis pre-flight (scope/drift) → Flux verify (commit-pinned build/tsc/smoke + static invariants + runtime) → Lyra acceptance`. Recorded after Mr. Zhang flagged missed gate on A1/v01/A4-α/A2. (Lyra / Aegis / Mr. Zhang)
- [ ] MVP Implementation Start (re-scoped 2026-05-09 per Mr. Zhang directive — not MVP framing; full product design + technical architecture supplements must land and pass Aegis review before any v0.1 implementation packet beyond SG-A)

---
*Last Updated: 2026-05-11 (B1 ACCEPTED provisional, Layer B pending Mr. Zhang; Aegis consolidation adjudication complete with all 7 verdicts applied; 008 DELIVERED at 526d25a, Flux Layer A verify dispatched; product supplement §1.3 BINDING ratified, §1.4/1.6 PROVISIONAL, §1.5 BINDING tags applied; Nimbus arch supplement §A.7 retraction inlined per Aegis-revised wording)*
