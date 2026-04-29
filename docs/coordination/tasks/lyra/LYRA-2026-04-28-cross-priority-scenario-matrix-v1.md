# LYRA-2026-04-28-Cross-Priority-Scenario-Matrix-v1

| Field | Value |
|---|---|
| Owner | Lyra |
| Status | Active baseline artifact |
| Purpose | Make P0/P1/P2 user stories explicit before implementation starts |
| Contract basis | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/interaction-spec-v1.0.md`, `docs/archive/product-history/acceptance-spec-v1.0.md` |
| Constraint references | `docs/archive/product-history/mvp-scenarios.md`, `docs/architecture-decisions.md`, `docs/coordination/reviews/2026-04-28-lyra-review-response.md` |

## 1. Scenario precedence note

This artifact is operationally normative for implementation planning until the older scenario and UX files are updated.

It resolves four current mismatches:

1. Adds a formal multi-project switching story missing from `docs/archive/product-history/mvp-scenarios.md`.
2. Treats Handoff as `P0`, not a later-phase-only flow.
3. Removes clipboard-full-pack fallback from runtime-switch/rehydrate stories.
4. Separates baseline Timeline replay (`P0`) from deep decision-retrospection expansion (`P2`).

## 2. P0 scenario matrix

| ID | Story | Trigger | Happy path | Failure / exception path | Deferred / guardrail | Acceptance anchor |
|---|---|---|---|---|---|---|
| P0-S01 | Initialize a new project | Open repo with no `.seatloom/` | Confirm init, create base files, land in guided empty state | Init fails -> visible error + retry | No hidden auto-init | `P-01`, `U-03`, `E-04` |
| P0-S02 | Switch between projects safely | Project Switcher, `Cmd/Ctrl+K`, All Projects | Pick target project, pass protection checks, restore prior UI state | Missing path, permission denied, running session/unsaved-input protection dialog | All Projects is summary only; no cross-project object editing | `P-00`, `U-00`, `E-00`, `Q-00` |
| P0-S03 | Add an accountable seat | Sidebar `SEATS +` | Create seat with name + role, see it immediately in Sidebar | Duplicate name or invalid role is blocked with clear validation | No social/profile-heavy seat system in baseline | `U-01`, `E-01` |
| P0-S04 | Attach a running session | `Attach` from Sessions area | Choose process, assign seat, open session in shell and Terminal | Attach unsupported -> `Launch with wrap` fallback | Best effort only; environment may limit attach | `P-05`, `E-03`, `Q-01` |
| P0-S05 | Launch a new tracked session | `New Session` from Terminal | Select seat/runtime/workdir, start session, auto-focus terminal tab | Launch failure -> reason + retry | Must preserve native-like terminal feel | `P-05`, `E-05`, `Q-04` |
| P0-S06 | See and act on actionable work in Inbox | App opens or new actionable item arrives | Inbox orders by priority, Detail shows next action, action mutates the linked object | Invalid action is disabled; unresolved object stays pending | Inbox stays human-only; no generic notification feed | `P-02`, `U-01`, `E-02`, `Q-01` |
| P0-S07 | Replay recent work with evidence | Open Timeline / apply filters | Read human labels, filter by seat/workitem/type/time, open linked detail | Unknown event still renders with readable fallback label and payload | Deep transcript analysis is not baseline-required | `P-03`, `E-02`, `Q-03` |
| P0-S08 | Create and progress a WorkItem | `Ctrl+N` or WorkItems `+` | Quick create to `draft`, enrich with goal/AC/owner, progress through gates | Missing AC/owner/evidence blocks transition with explicit reason | No hidden state jumps | `P-01`, `P-02`, `E-01`, `Q-02` |
| P0-S09 | Send and close a Handoff | `Ctrl+H` or WorkItem Detail action | Create with purpose/expected outcome/artifacts, send, accept or return, then complete | Missing required fields block send; return creates sender-visible consequence | Automatic expiry is deferred; manual stale handling only | `P-04`, `P-02`, `Q-01`, `Q-02` |
| P0-S10 | Continue work in another runtime | Session Detail `Switch Runtime` | Generate LaunchPack, preview it, launch target runtime, continue without re-explaining core context | Injection fails -> file path + short instruction fallback | No clipboard-pack standard path | `P-05`, `E-03`, `E-04`, `Q-01` |
| P0-S11 | Recover an interrupted session | Session enters `interrupted` | Offer native resume, rebuild, or minimal recovery in fixed order | Each failed recovery path yields visible next-step guidance | Best-effort native resume only when supported | `P-05`, `E-03`, `Q-01` |
| P0-S12 | Re-anchor at startup or before risk | App launch, manual reconcile, pre-pipeline reconcile | Reconcile, produce Morning Digest, route actionable items to Inbox | Reconcile failure is visible and retryable | Full pipeline operations remain outside baseline-critical scope | `P-06`, `P-02`, `Q-04` |

## 3. P1 scenario matrix

| ID | Story | Why it exists | Required P0 foundation | Not a baseline blocker |
|---|---|---|---|---|
| P1-S01 | Edit frequent objects inline and navigate faster | Reduce friction once the contract is stable | Stable object IDs, focus model, Detail actions, keyboard map | Baseline may still use dialogs/forms. |
| P1-S02 | Import external evidence into the work graph | Capture screenshots, design notes, or manual findings as first-class artifacts | Artifact model, attachment refs, Detail evidence surfaces | No manual upload/import needed for initial go/no-go. |
| P1-S03 | Inspect runtime health before starting work | Increase trust in launch/attach/switch decisions | Runtime identity and capability truth already visible at execution points | No dedicated runtime-health dashboard needed at baseline. |
| P1-S04 | Follow live activity without losing replay quality | Watch ongoing work while preserving audit quality | Event stream, filters, linked objects, Terminal/session evidence | Timeline remains replay-first by default. |
| P1-S05 | Recover richer context through timed checkpoints or smarter packs | Reduce recovery effort on long-running work | Checkpoint model, LaunchPack file-first flow, capability fields | P0 stays deterministic and rule-based. |
| P1-S06 | See background automation with visible history | Make recurring checks/pipelines trustworthy and inspectable | Ledger append, Inbox/Timeline projection, artifact evidence | No invisible daemon-only behavior is acceptable. |

## 4. P2 scenario matrix

| ID | Story | Future value | Control rule |
|---|---|---|---|
| P2-S01 | A seat drafts an artifact or handoff for human approval | Reduce ceremony while keeping governance visible | Draft/proposal must be explicitly accepted before it becomes active truth. |
| P2-S02 | A seat proposes child work from execution evidence | Let decomposition emerge from real work without silent scope mutation | Child work remains a proposal until a human promotes it. |
| P2-S03 | Temporary scratch collaboration supports thinking without corrupting the ledger | Give operators room for exploration while protecting traceable delivery | Accepted outcomes must still be attached back to WorkItem/Handoff/Artifact records. |

## 5. Execution use of this matrix

- Mira should design every `P0` row as a fully supported story with explicit empty/error states.
- Nimbus should wire only `P0` rows into the first implementation wave unless a later gate promotes a `P1` row.
- Flux should structure runbooks and evidence around `P0-S01` through `P0-S12` first.
- Any new idea that does not fit this matrix must be assigned to `P1`, `P2`, or explicitly rejected before implementation starts.
