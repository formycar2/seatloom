# SeatLoom top-level design — one record, one interpretation, governed action

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-19-top-level-design-v1 |
| status | delivered |
| author | lyra |
| date | 2026-10-19 |
| version | v1 |
| tags | top-level-design, layers, authority, first-screen |
| depends_on | LYRA-2026-10-19-whole-product-audit-v1, disp-hloGX15yBiBBhIFivlxzj |

## 1. Product thesis and scope

SeatLoom is a **governed work system with recoverable evidence**, not a terminal dashboard plus
separate collections of tasks, chat, files and memories. Its central loop is:

**observe work → record it → explain the current situation → obtain the authorized decision →
execute it → record the actual result**.

This design lifts the delivered capability audit into a vertical architecture. It does not repeat
the eleven-tab inventory, change the user-approved direction, prescribe implementation steps,
route work, or authorize any interface redesign. A layer is a responsibility, not a new page,
service or database. One existing component may host several responsibilities, but its commands
and reads must remain separable.

The audit established two facts that shape the design: current progress presentations observe
both dispatches and workitems without a unified meaning, and the existing acceptance area is the
closest real decision surface. The design preserves the latter and resolves the former by assigning
metric authority to a single interpretation contract, rather than to individual widgets.

## 2. Five layers, with exclusive responsibilities

Five layers are needed because activity, durable evidence, interpretation, authorization and
presentation have different guarantees. Combining activity with authorization lets an agent approve
itself; combining interpretation with recording turns an inference into history; combining
presentation with either makes each screen invent its own truth.

| Layer | User question | Inputs | Outputs / consumers | What it does NOT own |
|---|---|---|---|---|
| **A. Execution and observation** | What actually ran, changed, or was said? | Authorized work commands; harness/process output; source stores; actual action receipts | Raw observations, produced objects and actual results to B | Does not decide who may approve, interpret a complete project status, or equate a lively terminal with delivery progress. |
| **B. Authoritative record and evidence** | What was recorded, by whom, and what supports it? | Validated observations/results; admitted lifecycle events and references; source/archive manifests | Immutable event history, contextual identities and resolvable evidence; maintained current-state projections to C and checks for D | Does not invent missing observations, classify an inference as original evidence, count user obligations, or own source files merely because they are indexed. |
| **C. Work interpretation and query** | What does that record mean now, and what do we know or not know? | B's history/current projections and explicit object relationships; recorded governing policy | One scoped, timestamped work/attention/lineage read model to E; facts/candidates/evidence to D | Does not authorize or execute an action; does not manufacture a link between dispatch and workitem or infer a user decision from every blocked row. |
| **D. Governance and decision** | Who must decide, what choices are legal, and what action may follow? | C's explained candidate; B's current facts/evidence; actor identity, authority and policy; explicit human or authorized supervisor decision | Validated decision record to B; authorized command to A; refusal/rework to C/E | Does not change a verdict, silently delegate a human choice, report an authorized command as already executed, or supply its own competing progress counters. |
| **E. Presentation and interaction** | What should I look at or do here? | C's read models; D's allowed actions/requirements; evidence views resolved through B/C | Clear situation and explanation to the user; explicit user requests/decisions to D | Does not locally decide policy, write durable state directly, invent freshness/health, or treat a disabled backend capability as a successful local save. |

### No overlap, no vacuum

A owns **effects**, B owns **what is durably recorded**, C owns **meaning/query**, D owns
**permission/decision validity**, E owns **communication and input**. A decision's meaning and
allowed actions are separate: C can describe a pending acceptance; only D validates and applies a
human's acceptance. B persists it; E only presents it.

The archive browser, file browser and knowledge views sit in E; their source pointers and records
are in B; search, source resolution and memory relationships are C. Existing monitoring/task views
also sit in E. The supervisor dialog is E input backed by D, not a sixth supervisor-shaped data
layer. The terminal mirror is E's window into A, not C's authority on progress. This accounts for
the prior audit's full surface without assigning every tab its own architecture.

## 3. What “one source of truth” means

**B is the only authority boundary for recorded activity and evidence. C is the only authority for
published interpretations and metrics. E is never either.** This does not mean “put every byte in
one SQL table” or “every current object table is already rebuildable from the ledger”.

| Claim | Authority | Consequence |
|---|---|---|
| An activity/decision occurred, attributed to an actor | Canonical event history and explicit references in B | Current projections or terminal text must not rewrite historical meaning. |
| Source code/document/raw transcript contained these bytes | Its designated source store or recorded immutable version/digest, represented through B | A database index is a locator, not a replacement source. A missing/mutated source is reported, not reconstructed from a summary. |
| This is the current governed dispatch/acceptance state | Its maintained domain projection in B, read with dependencies/policy through C/D | Never derive completion from an issued row's existence or a branch name alone. |
| This memory says X | Versioned/status-bearing memory record in B | Its provenance and quoted/inferred status remain distinct from the evidence that supports it; a memory is not a verdict. |
| These are the user's pending decisions / this is work progress | C's documented metric/read-model contract over B | Every location displays the same result for the same scope/as-of time. No widget-local competing formula. |

A count is identified by **meaning + entity population + scope + status predicate + as-of time +
source/coverage**. “Active dispatches” and “active workitems” are different counts and may differ;
calling both “team progress” without their relationship is forbidden. Deduplicate by recorded
identity/links, not titles. If dispatch-to-workitem links are absent, show separate qualified
populations; do not add them together or pretend one covers the other.

C may expose the same number in the first screen, a project overview or chat context. Each is a
presentation of ONE contract, not an independent authority. Incomplete coverage, stale snapshots,
failed reads, and genuinely empty results remain four different states.

## 4. Top-level diagram and direction of dependence

```text
       External sources / harnesses / repositories / runtime
                      | actual observation / result
                      v
                 A. EXECUTION <-------- authorized command -------- D. GOVERNANCE
                      |                                             ^       |
                      | fact + evidence                             |       | admitted decision
                      v                                             |       v
                 B. RECORD / EVIDENCE ---------- facts/current checks ------+
                      |
                      | immutable history + current projections + pointers
                      v
                 C. INTERPRETATION ---- explained candidate --------> D
                      |                                             ^
                      | situation / attention / lineage             | request or decision
                      v                                             |
                 E. PRESENTATION -----------------------------------+
                      |
                      v
              User: understand / inspect / choose

Evidence side path: E requests detail -> C resolves through B -> original source/version.
No direct durable write E -> B. No authority shortcut A -> user approval.
```

The arrows are contracts and dependencies, not a mandatory linear service chain. D can check B
again without trusting a stale C/E snapshot. A can emit observations without a preceding user
request. Source reads need not copy transcript bodies into the canonical ledger. Querying evidence
is not executing a command.

### 4.1 A seat's raw process to something the user sees

A observes a tool result or created artifact. B records the admitted typed fact, context and
source pointer. C attaches only recorded relationships and interprets status/lineage at a named
scope/time. E displays the work result or evidence zoom. A mirrored terminal can appear directly
as raw detail in E, but that observation does not count as work completion unless the governed
record supports it. Historical transcripts enter through B's indexed archive boundary and C's
read-only replay, not as fabricated past canonical events.

### 4.2 A verdict or product decision to executed work

- **Technical verdict:** reviewer A produces a conclusion with evidence. D validates who issued
  it and what work it concerns; B records the verdict. C explains it. The verdict is evidence for
  a later human decision, NOT that human decision itself.
- **Human acceptance/product choice:** E submits an explicit choice, identity, target and the
  shown version/context to D. D checks authority and current prerequisites, admitting or refusing
  it. The decision is durably recorded in B; authorized downstream work is sent to A. Actual action
  success/failure comes back through A/B and then C/E.
- **Acceptance with a git merge:** validation precedes the merge. The result is recorded only after
  the actual merge outcome is known. Git and the DB are different systems: this design does not
  claim one atomic transaction spans both. A crash between effect and receipt is an unknown outcome
  requiring reconciliation, not a fabricated success or an automatic repeated merge.
- **No repository:** a product decision can still be recorded and dispatched, but there is no git
  merge action to execute. The action capability is explicit rather than making git universal.
- **Rejection/correction:** record the refusal/reason and authorized rework; do not silently edit
  the historical verdict or original user choice. Retry and reissue preserve identity and provenance.

Every transition changes one thing: A changes the world; B records it; C explains it; D grants or
refuses authority; E lets the user understand and choose. A queued command, a successful operation,
a recorded result and an installed capability remain distinct states.

### 4.3 Evidence is shared, but its authority is not duplicated

All layers may **reference** evidence within their permission boundary. B owns admission,
provenance, retention and source resolution; C supplies bounded queries; D requires enough valid
support for the action; A consumes approved context; E shows progressively disclosed detail.
Do not copy evidence into each screen as separate truth. Indexes and summaries may be derived,
but point to the same source/version and show derivation. Unknown relationships stay unknown.
Sensitive raw content is not automatically available merely because a summary is visible.

## 5. Five executable placement and existence rules

1. **Effect/authority test:** if it changes durable state or external systems, it is an A command
   authorized by D with a B receipt. If it only reads/describes, it belongs to C/E. A local form
   mutation cannot satisfy a save contract. Tests must distinguish authorized, executed, recorded
   and unknown outcome.
2. **Metric identity test:** before adding a number, declare its population, predicate, scope,
   as-of time and authoritative C contract. If another view displays the same metric, compare their
   contract identities and results. A mismatched formula fails; a distinct population requires a
   distinct label. “Progress” without a declared denominator/coverage cannot be published.
3. **Decision obligation test:** a user-decision item requires an explicit target, allowed choices,
   authorized decider, pending/resolved state, evidence and downstream consequence in B/D. A message,
   failed read or blocked dispatch alone fails this test. Missing decider/relationship is reported,
   not automatically assigned to the human.
4. **Evidence and state test:** every claim displayed by E resolves through C to B/source or is
   explicitly marked inferred/unavailable with scope/freshness. Fault tests must distinguish empty,
   incomplete, stale and unavailable. A current render clock or a hardcoded green percentage fails.
5. **Entry justification test:** a proposed interface must either present the shared work/attention
   contract, accept a governed action, or provide a distinct evidence/context zoom with a return
   to that same work. If it duplicates the same contract without a distinct user task, reuse the
   existing presentation; if it bypasses B/C/D, reject it. A needed new navigation/interaction waits
   for the user's approval, not this architecture document.

## 6. The first screen's top-level position

**The user lands in E, looking at C's shared work-and-attention interpretation, with D-backed actions
alongside it.** The user does not land in raw execution A, the storage model B, or a competing
supervisor shell. This maps exactly to the already chosen direction:

- **What the whole team is doing:** current governed work with ownership/state/coverage from C.
  A terminal's liveness and a seat's membership are contextual, not substitute progress.
- **What is stuck:** evaluated prerequisites and blockers from C, with who owns the next step when
  explicitly known. Missing expected-producer data remains unknown; no guessing from an event id.
- **What needs my decision:** D-defined pending obligations interpreted by C, not all alerts,
  internal memory events or every blocked task. Acceptance is one class, not the entire set.

The prior audit's choice **dispatch page + acceptance area is the closest existing anchor** stands.
Its component placement is not the architecture: those existing sections should consume the common
contract instead of defining its meaning. Current project-scoped data cannot be silently relabelled
“whole fleet”; a fleet aggregate retains project attribution and declared coverage. History,
knowledge, files and raw terminals are contextual zooms from this situation. The supervisor dialog
is the input/discussion companion, not a second independent dashboard authority.

No new first-screen layout or navigation is specified here. Existing sections and information
architecture stay intact. The mirror is transitional as the main face and remains an end-state
raw diagnostic detail. Evidence replay is not transitional merely because its implementation lives
in an older directory.

## 7. User choices — kept separate from technical contracts

The following are not implementation assignments and do not reopen the already decided three
first-screen questions:

- **Default scope:** recommendation is a whole-team view retaining each project's identity, with
  the existing project selection narrowing the view. The current anchor is project-local. Confirm
  the default scope before changing what appears on opening the product; no invented cross-project
  obligations are allowed in the meantime.
- **Detail of internal bookkeeping:** whether memory recall/topic bookkeeping is collapsed by
  default in the event zoom. Recommendation: it should not displace pending work/decisions on the
  first screen, while remaining inspectable as evidence. Exact default disclosure is the user's.
- **Visible consolidation:** architecture recommends one progress meaning and one governed input
  path. Which existing duplicated entry remains visible, and ownership of the current shortcut,
  are user choices before any navigation removal or reassignment.

The single-source metric contract, evidence/decision distinction and no-direct-local-save rule
are architecture responsibilities, not questions the user must solve. Work routing, integration,
permission grants and gate decisions remain Aegis's; this document neither dispatches nor performs
any of them.

## 8. Evidence basis, current gaps and limits

The completed capability audit is the baseline, not a new inventory. It measured transitive
reachability, upstream event ordering, current scoped hydration, dashboard counter inconsistency,
mock global summary and retained local-only forms. Those are the concrete failures this layering
must prevent; they are not evidence the proposed shared interpretation is already implemented.

Additional source checks at clean `track@cfda740` confirm: the core event append path explicitly
maintains projections; dispatch lifecycle uses the transaction-aware append primitive; the
acceptance command checks verdict value/subject before its external merge; intent planning has a
single facts gatherer and returns wrote:false; archive manifests refer to original bytes on disk.
These checks support the responsibility boundaries, NOT complete coverage: legacy tables/read
paths coexist, supervisor-message delivery has a stubbed injection branch, and not every projection
is proven replayable today.

Earlier 2026-09-30 design reviews distinguish activity/object/context, source-owned bytes, typed
activity versus raw/process evidence, and historical ledger versus current projections. They are
labelled draft references: this document uses their useful distinctions without presenting every
proposal as an approved or shipped capability. The five responsibility layers here are NOT those
three activity-fidelity tiers; a trace or raw record belongs to B, not a new user-facing layer.

Validation is source-level plus consistency review: every responsibility and metric has one owner,
all three first-screen questions map to that model, effects and unknown outcomes have a route back
to the record, evidence remains available without copied authority. No rendered-app claim, code
change, live database change, gate emission, merge, acceptance card or sub-task issuance.
