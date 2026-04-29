# SeatLoom Interaction Spec v1.1

| Field | Value |
|---|---|
| Document | Interaction Spec v1.1 |
| Status | Draft - Pending review |
| Updated | 2026-04-28 |
| Language | English |
| Depends on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/ux-spec-v1.1.md` |

## 0. Purpose

This document defines the trigger, success path, failure path, and visible system response for the expanded v0.5 contract.

A flow is implementation-ready only when the user can answer all of these questions:

- What triggers the flow?
- What does SeatLoom show immediately?
- What can the user confirm, edit, or cancel?
- What changes after success?
- What happens if the operation fails or is incomplete?

## 1. Global interaction rules

### 1.1 Confirmation rule

- Any state-changing action proposed by the Supervisor must present a confirmation card before commit in P0 and P1.
- The confirmation card must support `Confirm`, `Edit`, and `Cancel`.

### 1.2 Deterministic-first rule

- Inbox labels, Timeline summaries, gate reasons, search hits, and route outcomes must appear without waiting for an LLM round trip.
- If an LLM-assisted enhancement is still loading, the deterministic baseline remains visible.

### 1.3 Focus rule

- `Cmd/Ctrl+K` opens the Supervisor Command Bar from any screen.
- `Esc` closes the top-most sheet, card, or drawer.
- `Enter` confirms the current highlighted action when validation passes.

### 1.4 Audit rule

- Every accepted state change must show a visible success result and create a linked object update that is discoverable from Timeline and Detail.

### 1.5 Isolation rule

- Worker seats can only access pack-included evidence or evidence the user explicitly loads through a visible action.
- Supervisor-only views must not leak automatically into worker continuity packs.

### 1.6 Budget rule

- Session, handoff, and pipeline budgets must be visible before launch and enforced after launch.
- Budget exhaustion must produce a structured stop state with retry, escalate, or partial-result options.

### 1.7 Artifact typing rule

- Coordination artifacts use `template+subtype` as the canonical classification key. `template` selects the broad family badge and base layout; `subtype` selects the exact summary block, primary actions, and automation contract.
- Extracted header metadata must include `template`, `subtype`, `id`, `status`, `author`, `date`, `version`, `depends_on`, `supersedes`, and `tags`, and `subtype` must validate against `docs/coordination/DOCUMENT_TEMPLATES.md` Section 11.1.
- Timeline, Inbox, and Detail evidence projections must render typed artifact references when the dual key validates, so users open artifact objects directly instead of reading raw document paths.
- If either field is missing or invalid, SeatLoom must fall back to a generic reader with a visible warning and disable subtype-specific routing or gating assumptions.

### 1.8 Review change tier rule

- Any review-driven follow-up edit triggered from WorkItem return, delivery rejection, artifact review, or gate correction must be classified as `L1`, `L2`, or `L3` before execution starts.
- SeatLoom must persist a structured `change_tier_record` with `tier`, `reason`, `changed_clauses`, `impact_level`, `executor`, `reviewer`, `ack_mode`, and `evidence_refs` before the follow-up path is considered active.
- `L1` allows reviewer direct patch, `L2` dispatches worker patch with compact acknowledgment only, and `L3` dispatches worker patch plus a full gate review before closure.
- Missing tier data, missing evidence references, invalid changed-clause anchors, or an `L2` response that expands beyond the compact acknowledgment contract must keep the review loop open and block acceptance.

### 1.9 Mobile channel consistency rule

- Mobile companion surfaces are required for monitoring, approval/takeover, and short feedback return loops, but they must not introduce separate workflow objects or mobile-only state branches.
- Any mobile `Approve`, `Reject`, `Escalate`, `Return`, `Reserve desktop takeover`, or `Send feedback` action must mutate the same canonical objects, run through the same gate logic, and emit the same audit-visible outcome that desktop would create.
- Mobile may add channel metadata such as `source=mobile`, notification receipt markers, or cached-sync warnings, but desktop Timeline, Inbox, Detail, and review surfaces remain the system of record after projection.

## 2. Flow index

| ID | Flow | Priority | Primary modules |
|---|---|---|---|
| `INT-01` | Start day and review action queue | P0 | Supervisor, Data Engine |
| `INT-02` | Create and route work through the Supervisor | P0 | Supervisor, Seat, Data Engine |
| `INT-03` | Reuse seat identity and manage project role, capability, and delegation | P0 | Seat, Data Engine |
| `INT-04` | Deterministic Inbox and Timeline projection | P0 | Data Engine |
| `INT-05` | Fail review and reissue work with evidence | P0 | Data Engine, Supervisor, Artifact review |
| `INT-06` | Switch runtime, suspend sessions, and recover continuity | P0 | Data Engine, Playbook |
| `INT-07` | Open artifact and add manual review comments | P0 | Artifact review |
| `INT-08` | Ask Supervisor to draft review comments | P1 | Supervisor, Artifact review |
| `INT-09` | Publish and apply playbooks / supervisor continuity | P1 | Playbook, Data Engine, Supervisor |
| `INT-10` | Clone collaboration template from another project | P1 | Seat |
| `INT-11` | Review bounded automation proposals | P2 | Supervisor, Seat, Playbook |
| `INT-12` | Track handoff progress and live activity | P1 | Seat, Data Engine |
| `INT-13` | Search exact project evidence and load deeper context | P0 | Data Engine, Artifact review, Supervisor |
| `INT-14` | Ask why a past decision happened | P1 | Data Engine, Supervisor, Playbook |
| `INT-15` | Compare reuse, cost, and routing outcomes | P2 | Playbook, Data Engine, Seat |
| `INT-16` | Handle interactive prompts in wrapped runtimes | P0 | Data Engine, Supervisor |
| `INT-17` | Monitor project health from mobile | P0 | Data Engine, Supervisor |
| `INT-18` | Approve, reject, or escalate urgent decisions from mobile | P0 | Supervisor, Data Engine, Artifact review |
| `INT-19` | Send mobile feedback back into the work loop | P0 | Supervisor, Artifact review |
| `INT-20` | Triage prompt, gate, and handoff interrupts from mobile | P0 | Supervisor, Data Engine, Seat |
| `INT-P3-01` | Organize module-parallel work before replicating core seats | P3 | Supervisor, Seat, Data Engine |

## 3. Flow definitions

### INT-01 Start day and review action queue

Trigger:

- app launch into an initialized project,
- manual `Reconcile now`, or
- project switch into a project with existing state.

User sees immediately:

- the Project Overview in Main,
- Morning Digest if reconcile found new interruptions, drift, comments, or pending handoffs,
- an Inbox list sorted by priority,
- a suggested next action chip from the Supervisor,
- reconcile freshness and budget-health warnings in the shell status area.

Success path:

1. App loads project state.
2. Data Engine projects canonical object state into Morning Digest and Inbox rows.
3. User selects an Inbox item or accepts a suggested next action.
4. Detail opens with linked evidence and allowed actions.
5. Selected action updates the underlying object and removes or updates the Inbox row.

Failure path:

- reconcile fails: show `Reconcile failed` with reason and `Retry`, `Open logs`, `Continue with cached state`;
- project data missing: show `Project data incomplete` with `Repair project` and `Open folder`.

### INT-02 Create and route work through the Supervisor

Trigger:

- `Cmd/Ctrl+K`,
- `New Work` button,
- context menu action such as `Create follow-up from comment`.

User sees immediately:

- command bar input,
- after submit, a suggestion card with proposed WorkItem, owner, acceptance criteria, linked artifacts, optional handoff, seat rationale, and budget estimate.

Success path:

1. User enters natural-language intent.
2. Supervisor parses intent using current project context, seat constraints, and capability data.
3. SeatLoom shows a suggestion card.
4. User may `Edit` fields inline or `Confirm` directly.
5. Data Engine writes the WorkItem, optional Handoff, and audit events.
6. Sidebar, WorkItems view, Inbox, and Timeline update.

Failure path:

- no capable seat: show `No eligible seat found` and list blocked constraints;
- ambiguous owner: show `Owner unclear` and present seat choices;
- missing authority: show `Role not permitted` and require reassignment;
- budget violation: show `Pack exceeds budget` and highlight the cause before confirmation;
- validation failure: highlight missing AC, missing linked evidence, or prohibited routing.

### INT-03 Reuse seat identity and manage project role, capability, and delegation

Trigger:

- `Add seat` from project setup,
- `Manage seats` from project settings,
- `Delegate this work` from WorkItem detail,
- `Open seat card` from a suggestion card or seat row.

User sees immediately:

- Global Seat Registry picker or creator,
- Seat Card panel with capabilities, accepted input types, output types, budgets, constraints, and attached seat skills,
- Project Role Drawer with role, authority docs, constraints, and active collaboration template,
- delegation overlay editor when assigning temporary responsibility,
- in P1, a read-only Execution Template inspector.

Success path:

1. User selects an existing global seat or creates a new identity.
2. User completes or reviews Seat Card information.
3. User binds that seat to a project role.
4. Seat row updates with role badge, runtime hints, and capability truth.
5. For delegation, user chooses a delegate seat, scope, issuer, and expiry.
6. Timeline records the delegation event; seat detail shows the delegation badge.

Failure path:

- duplicate seat name in global registry: inline validation;
- Seat Card missing mandatory runtime or budget data for a routable seat: show `Seat card incomplete` and block assignment;
- project role missing required authority doc: block save and show required docs;
- delegation without scope or expiry: block confirmation.

### INT-04 Deterministic Inbox and Timeline projection

Trigger:

- object state change in WorkItem, Session, Handoff, Comment, or Pipeline,
- manual rehydrate or reconcile,
- accepted Supervisor action.

User sees immediately:

- new or updated Inbox row when human action is required,
- new Timeline row with readable label and summary,
- typed artifact evidence chips in Inbox or Timeline whenever linked coordination documents have valid `template+subtype` metadata,
- gate reason or route reason in Detail when applicable,
- budget warning chips when a state change affects cost risk.

Success path:

1. Canonical object changes.
2. Data Engine evaluates routing, template, gate, and budget rules.
3. SeatLoom projects the change into Inbox and Timeline and resolves linked artifact evidence into typed references where metadata is valid.
4. User can click either row to open linked object detail.

Failure path:

- missing template mapping: show fallback readable label `Action required` plus object refs and log a template-gap diagnostic;
- linked artifact exists but metadata is missing or invalid: keep a visible raw-path fallback warning and disable subtype-specific actions for that reference;
- projection write failure: keep canonical object state, show retry toast, and mark projection status in Detail.

### INT-05 Fail review and reissue work with evidence

Trigger:

- user selects `Return`, `Reject delivery`, or `Reissue` from WorkItem, Handoff, or Artifact review context.

User sees immediately:

- review verdict card,
- required reason field,
- linked artifact, checkpoint, or comment evidence,
- suggested reissue path and assignee,
- recommended follow-up tier (`L1`, `L2`, or `L3`) with execution mode,
- a `change_tier_record` preview showing changed clauses, impact level, reviewer, executor, and evidence references.

Success path:

1. User opens a delivered work item or linked artifact.
2. User chooses `Reject delivery` or `Return`.
3. SeatLoom requires verdict reason and evidence links.
4. Data Engine records the review verdict and scope-change event.
5. SeatLoom classifies the requested follow-up as `L1`, `L2`, or `L3` and drafts the structured `change_tier_record`.
6. Supervisor suggests the reissue target, updated acceptance criteria, and tier-appropriate execution path.
7. If the user confirms an `L1` follow-up, the reviewer enters a direct-patch path and the record is logged immediately.
8. If the user confirms an `L2` or `L3` follow-up, the WorkItem re-enters the active path through existing lifecycle rules plus the stored `change_tier_record`; `L2` expects compact acknowledgment only, while `L3` stays open for a full gate review after delivery.

Failure path:

- missing reason or evidence: block submit;
- missing tier, missing `changed_clauses`, or missing `evidence_refs`: block submit;
- invalid reissue target by role constraints: require seat change;
- `L2` acknowledgment exceeds the compact schema: show `Ack format invalid`, keep review open, and require a compact resubmission;
- user cancels after verdict draft: no state mutation, draft remains local only.

### INT-06 Switch runtime, suspend sessions, and recover continuity

Trigger:

- `Switch runtime` from Session detail,
- `Recover session` from interrupted Session,
- `Suspend session` or `Resume session` from active/suspended Session,
- supervisor continuity rebuild request in P1.

User sees immediately:

- target runtime picker or resume choice,
- continuity pack preview split into context tiers,
- separate seat-skill and playbook sections,
- budget estimate and remaining budget indicator,
- ordered recovery options where applicable,
- `Load more evidence` when deeper context exists.

Success path:

1. User selects target runtime, suspend/resume mode, or recovery mode.
2. Data Engine assembles the pack from structured checkpoints, structured index hits, and full-text evidence.
3. SeatLoom previews Tier 0 identity, Tier 1 state, Tier 2 decisions, plus optional deeper evidence.
4. User reviews matched seat skills, matched playbooks, and budget impact.
5. User confirms launch, recovery, suspend, or resume.
6. If resuming a suspended session in P1, SeatLoom injects only delta context since the last checkpoint.
7. Timeline records the switch, suspend, resume, or recovery event.

Failure path:

- injection unavailable: show pack file path plus short instructions;
- pack too large: show truncation summary and allow open-file inspection;
- native resume unavailable: fall back to rebuild and then minimal recovery;
- budget exceeded: show which tier or evidence block caused overflow and require trim or escalation before confirmation.

### INT-16 Handle interactive prompts in wrapped runtimes

Trigger:

- a wrapped session enters a blocked interactive state (menu, wizard, freeform input request, or sensitive prompt),
- the system detects `stdin required` and a prompt-classifier match, or
- the runtime emits an explicit structured prompt frame (recommended).

User sees immediately:

- a visible `Prompt blocked` banner on the Session Panel and a chip on the Session row,
- prompt classification: `deterministic`, `wizard/menu`, `freeform`, or `sensitive`,
- a policy badge: `auto allowed`, `needs approval`, or `human required`,
- action choices: `Approve`, `Human takeover`, `Supervisor assist` (when allowed), `Stop`.

Success path (deterministic prompts):

1. Data Engine classifies the prompt from a bounded terminal window plus structured session state.
2. If policy allows auto-complete, SeatLoom injects the deterministic response (e.g., `y`, `n`, a numeric choice) and records an audit event.
3. The session resumes; Timeline logs `prompt.resolved` with evidence.

Success path (complex prompts with Supervisor assist):

1. Data Engine classifies the prompt as `wizard/menu` or `freeform`.
2. SeatLoom requests explicit confirmation before injecting any non-trivial input.
3. Supervisor assist uses only a bounded prompt window plus structured context tiers 0-2; it must not require rereading long terminal logs.
4. SeatLoom shows a confirmation card with the proposed keystrokes/text input, risk level, expected next prompt, and a hard step budget.
5. User confirms; SeatLoom injects the input.
6. The next prompt must match the expected pattern or SeatLoom stops and escalates to `Human takeover`.

Failure path:

- classifier uncertain: show `Prompt unclear` and require `Human takeover` or `Stop`;
- sensitive prompt detected (password/OTP/sudo/secret): show `Sensitive prompt` with `Human required` and recommended safe path (credential helper / askpass / keychain);
- assist budget exceeded: show `Assist budget exceeded` and offer `Human takeover` or `Stop` plus a playbook-candidate note.

Audit requirements:

- Emit `prompt.detected` with prompt type, risk, and bounded evidence window reference.
- Emit `prompt.input_injected` with operator (user/supervisor), scope (once/session/project), and result.
- If Supervisor assist is used, record token budget, step budget, and confirmation evidence.

### INT-17 Monitor project health from mobile

Trigger:

- user opens the mobile companion directly,
- user opens a deep link from a high-priority notification, or
- user returns to a cached mobile session while away from the desktop.

User sees immediately:

- Mobile Overview with project health, open blockers, waiting approvals, and budget warnings,
- Mobile Inbox limited to high-priority and approval-worthy items by default,
- interrupt cards for `Prompt blocked`, `Gate needed`, `Handoff pending`, or returned review items,
- sync freshness, last successful reconcile time, and current project selector.

Success path:

1. Mobile companion loads the same canonical project projection used by desktop Overview and Inbox.
2. Data Engine returns ranked mobile-safe cards for monitoring, approval, and short-feedback follow-up.
3. User opens an item to read an action card or quick view.
4. If the user acts, SeatLoom routes the action through the same canonical object transition and audit flow used on desktop.
5. Desktop Timeline, Inbox, and Detail refresh to the new state after projection.

Failure path:

- sync snapshot unavailable: show last cached state, `Retry sync`, and a visible `Read-only until sync recovers` banner;
- project no longer available on the mobile device: show `Project unavailable on this device` with `Choose another project`;
- projection stale beyond policy threshold: keep read-only summary visible and require refresh before approval actions.

Empty state:

- no urgent items: show `All caught up on mobile` plus `View recent activity`.

### INT-18 Approve, reject, or escalate urgent decisions from mobile

Trigger:

- user taps a pending approval from Mobile Inbox,
- user opens a `Gate needed` / `Handoff pending` / `Prompt blocked` notification, or
- user opens an action card from Mobile Overview.

User sees immediately:

- a Mobile Action Card with object ref, owner, wait time, summary, and current requested decision,
- quick-view links for the most relevant event or artifact evidence,
- action buttons such as `Approve`, `Reject`, `Escalate`, and `Open on desktop`, with the allowed set determined by object type and policy.

Success path:

1. User opens the pending decision on mobile.
2. SeatLoom loads the same governing object state, gate reason, prompt policy, or handoff summary that desktop would show.
3. User selects `Approve`, `Reject`, or `Escalate` and may add a short bounded note.
4. SeatLoom applies the same underlying state mutation that desktop would apply for that decision: approval clears the waiting state, rejection reopens or returns the work loop, and escalation creates or updates the appropriate follow-up object.
5. Audit history records the actor, channel=`mobile`, note, and linked evidence; Timeline and Inbox update on all surfaces.

Failure path:

- decision already resolved elsewhere: show the final state, actor, and timestamp in read-only mode;
- evidence unavailable within the mobile-safe window: show `More context required` and offer `Open on desktop`;
- policy forbids mobile approval (for example, sensitive prompt requires desktop takeover): disable the action and explain why.

Empty state:

- no mobile approvals pending: show `No approvals waiting on mobile`.

### INT-19 Send mobile feedback back into the work loop

Trigger:

- user taps `Add feedback` from a Mobile Action Card,
- user opens a WorkItem or Handoff quick view and chooses `Send feedback`, or
- user responds to a mobile prompt asking for short reviewer input.

User sees immediately:

- a Mobile Feedback Composer with target object ref and current status summary,
- mode chips such as `Needs change`, `Question`, and `FYI`,
- a bounded text input,
- optional quick link to the latest related event or artifact excerpt.

Success path:

1. User selects the target object and feedback mode.
2. User enters short feedback and submits it.
3. SeatLoom stores the result as the same comment or feedback object family used by desktop review and work-detail surfaces, linked to the WorkItem, Handoff, or artifact anchor when available.
4. Timeline and Inbox project the new feedback to the responsible owner and any reviewer who must follow up.
5. When the user or owner later opens desktop, the feedback already appears in the same canonical work loop rather than in a separate mobile inbox.

Failure path:

- target already closed or superseded: block submit and show `Target already closed`;
- feedback exceeds bounded mobile limits or anchor validation fails: keep the draft local and highlight the field that needs correction;
- sync write fails: keep a local draft with `Pending send` status and retry controls.

Empty state:

- no eligible open objects: show `No open work can receive mobile feedback right now`.

### INT-20 Triage prompt, gate, and handoff interrupts from mobile

Trigger:

- mobile receives a high-priority interrupt for `Prompt blocked`, `Gate needed`, or `Handoff pending`,
- user opens the interrupt queue from Mobile Overview, or
- user follows an interruption deep link from notification.

User sees immediately:

- an interrupt triage card with type badge, owner seat, elapsed wait time, and current risk or policy chips,
- for prompt interrupts, a bounded preview plus prompt classification and policy chips,
- only the actions allowed for that interrupt: `Approve`, `Reject/Return`, `Escalate`, `Reserve desktop takeover`, or `Stop`.

Success path:

1. Mobile companion opens the interrupt card from notification or Mobile Inbox.
2. SeatLoom loads the current interrupt state and bounded supporting evidence.
3. User selects a permitted action. If the user chooses `Reserve desktop takeover`, SeatLoom pauses any automated assist, marks the interrupt as reserved for desktop continuation, and keeps the session waiting safely.
4. SeatLoom applies the same canonical transition and audit-visible event that desktop would have applied for the same action.
5. The interrupt clears or reprioritizes across mobile and desktop projections.

Failure path:

- interrupt already handled: show who resolved it and when;
- chosen action is blocked by policy: disable the control and explain the required desktop path;
- waiting context aged out of the mobile-safe window: require `Open on desktop` before continuing.

Empty state:

- no urgent interrupts: show `No urgent interrupts on mobile`.

### INT-P3-01 Organize module-parallel work before replicating core seats

Trigger:

- user selects `Organize by module` from Project Overview, active work surfaces, or the Supervisor Command Bar,
- user opens `Module topology` in a project with several active workstreams, or
- the system detects repeated queue pressure on the core PO/architect lane and surfaces a topology suggestion.

User sees immediately:

- a Module Topology view with module session groups,
- a central core-lane strip showing the seats that still own final gate truth,
- per-group counts for sessions, WorkItems, Handoffs, Artifacts, and blockers,
- a delegation overlay drawer for assigning a temporary module lead,
- a replication-justification panel with throughput and drift metrics.

Success path:

1. User groups related active work under a named module session group.
2. SeatLoom links the relevant sessions, WorkItems, Handoffs, Artifacts, and progress rows to that group without changing their canonical ownership.
3. User optionally assigns a temporary module lead through a delegation overlay with explicit scope, issuer, expiry, and authority limits.
4. Timeline and Detail show the module-group context and delegation badge, while final gate ownership remains on the central core seat.
5. If queue pressure stays high, SeatLoom shows replication-justification metrics and a review-only recommendation about whether duplicating a core seat is warranted.
6. Any later replication request goes through a separate explicit review decision rather than happening automatically.

Failure path:

- group has no linked active work: block save and require at least one session or WorkItem;
- delegation lacks scope, expiry, or authority limit: block confirmation and highlight the missing field;
- user attempts to transfer final gate authority to a module lead: block and explain that delegation may not replace centralized gate truth;
- replication request lacks evidence metrics or active drift remains unresolved: show `Replication not justified yet` and keep the team in grouped mode.

### INT-07 Open artifact and add manual review comments

Trigger:

- `Open full content` from Artifact detail,
- click artifact reference from Timeline, WorkItem, or Inbox.

User sees immediately:

- full-content reader in Main,
- human-readable artifact family badge plus subtype chip and metadata summary strip,
- a family/subtype-specific summary block for task packet, review, acceptance, memory, or governance context when metadata exists,
- comment counts on headings or anchors,
- review thread rail on the right,
- exact-match search inside the artifact,
- actions: `Add comment`, `Request change`, `Reply`, `Resolve`, `Dispute`, `Open in editor`.

Success path:

1. User opens an artifact.
2. SeatLoom extracts `template` and `subtype`, validates the subtype allow-list, renders the family badge plus subtype chip, and shows the matching summary block if the artifact is a coordination document.
3. User searches, selects text or a heading, or chooses document-level comment.
4. Comment composer appears.
5. User submits the comment.
6. SeatLoom creates a comment object, Timeline row, and sender-side Inbox item when follow-up is required.
7. If the reviewer escalates the thread into a change request, SeatLoom opens the same `change_tier_record` flow defined in `INT-05` so the execution path is tiered and auditable before work resumes.

Failure path:

- artifact file missing: show `File not found` with `Locate file` and `Open artifact detail`;
- `template` or `subtype` missing, malformed, or invalid: show `Metadata unavailable` and fall back to a generic document reader;
- invalid anchor after file changed: preserve comment as section-level with warning;
- comment save failure: keep draft locally and offer retry.

### INT-08 Ask Supervisor to draft review comments

Trigger:

- `Ask Supervisor to comment` in artifact review,
- natural-language input such as `Comment on section 2 and say the scope is still unclear`.

User sees immediately:

- comment proposal card with anchor, draft text, and destination thread,
- options: `Confirm all`, `Edit`, `Cancel`.

Success path:

1. User invokes review assistance.
2. Supervisor drafts one or more structured comments.
3. User edits or confirms.
4. Confirmed comments are saved through the same comment object model as manual comments.

Failure path:

- ambiguous anchor: user must choose section or text range;
- proposal generation failure: fallback to manual comment composer.

### INT-09 Publish and apply playbooks / supervisor continuity

Trigger:

- resolved blocker,
- session end with reusable lesson,
- explicit `Save as playbook`,
- supervisor start after a gap in P1.

User sees immediately:

- playbook candidate card with trigger, resolution, scope, source evidence, and estimated savings,
- for supervisor continuity, a continuity preview showing unresolved blockers, open work items, pending handoffs, last gate decision, and delta changes since the last supervisor checkpoint.

Success path:

1. SeatLoom detects or receives a reusable lesson.
2. User reviews the playbook candidate and chooses project or global scope.
3. Playbook is published into the library.
4. On a later matching flow, Data Engine attaches the playbook into the pack preview separately from any seat skill.
5. For supervisor continuity, SeatLoom assembles the continuity pack and shows delta changes before launch.

Failure path:

- insufficient evidence: block publish and request source link;
- duplicate playbook candidate: offer merge or increment version;
- continuity pack assembly failure: fall back to Morning Digest plus active work summary.

### INT-10 Clone collaboration template from another project

Trigger:

- project initialization,
- project settings `Change collaboration template`.

User sees immediately:

- template picker with built-in presets and recent project templates,
- preview of routing rules, prohibited paths, escalation model, and seat expectations.

Success path:

1. User picks a built-in or source-project template.
2. SeatLoom previews key rules.
3. User confirms.
4. Current project stores the active collaboration template and updates seat detail surfaces.

Failure path:

- source project unavailable: show `Template source not found`;
- incompatible seat roles: show conflicts and require remapping.

### INT-11 Review bounded automation proposals

Trigger:

- seat draft ready for approval,
- Supervisor detects a repeatable next step,
- user opens `Proposals` tray.

User sees immediately:

- proposal card with source seat, affected objects, evidence, expected impact, and explicit approval requirement,
- actions: `Accept`, `Edit`, `Reject`.

Success path:

1. Proposal is created as a draft object.
2. User reviews the proposal.
3. Accepted proposal is promoted into canonical state through normal audit flow.

Failure path:

- stale source context: mark proposal stale and block accept;
- missing evidence: require evidence before accept.

### INT-12 Track handoff progress and live activity

Trigger:

- a Handoff is accepted by the receiving seat,
- user opens Handoff detail,
- user toggles `Live activity` from Timeline or Session detail.

User sees immediately:

- a visible Handoff state strip with `sent`, `accepted`, `working`, `completed`, and `returned`,
- optional live activity stream linked to the active session,
- latest artifact or checkpoint references,
- stale-progress warning if no event has arrived within policy.

Success path:

1. Receiving seat accepts a handoff.
2. Handoff enters `working`.
3. SeatLoom projects progress events, checkpoints, or artifact outputs into the live stream.
4. User watches progress without leaving replay surfaces.
5. Completion or return updates the Handoff state strip and Timeline.

Failure path:

- no live events within threshold: show `Progress stale` and `Ping owner`, `Open session`, `Mark blocker`;
- live stream unavailable: fall back to replay-only mode and keep canonical Handoff state visible.

### INT-13 Search exact project evidence and load deeper context

Trigger:

- global `Find evidence` action,
- search inside artifact review,
- `Open supporting evidence` from Detail,
- Supervisor request that needs exact grounding.

User sees immediately:

- search input and filters,
- artifact-family, artifact-subtype, and document-status filters when artifacts are in scope,
- grouped results for `Objects`, `Checkpoints`, `Artifacts`, and `Text matches`,
- artifact result cards with family badges, subtype chips, and extracted metadata chips when available,
- an optional `Load related evidence` action when deeper context exists.

Success path:

1. User enters a query or opens evidence search from context.
2. Query router executes Layer 1 structured lookup across object fields and artifact metadata, including `template` and `subtype`, then Layer 2 full-text search.
3. SeatLoom returns grouped exact matches without waiting for LLM synthesis.
4. User opens a result or loads related evidence.
5. Selected evidence opens in Detail, Artifact Reader, or continuity preview.

Failure path:

- no exact match: show `No exact result found` plus filter hints and, in P1, `Ask why instead`;
- metadata index stale: show `Artifact metadata needs refresh` with `Retry indexing`;
- index unavailable: show `Search index unavailable` with `Retry indexing` and `Open source folder`.

### INT-14 Ask why a past decision happened

Trigger:

- natural-language question such as `Why did we abandon passport.js?`,
- `Explain decision` action from WorkItem, Handoff, Playbook, or Artifact detail.

User sees immediately:

- answer card with a concise explanation draft,
- source evidence list,
- confidence note,
- `Open sources` and `Pin to continuity` actions.

Success path:

1. User asks a why-question.
2. Query router resolves exact references first and then semantic retrieval in P1 if required.
3. Supervisor composes an evidence-backed answer.
4. User opens sources, copies the answer, or pins it into a continuity pack.

Failure path:

- insufficient evidence: show `Evidence incomplete` with searched sources listed;
- semantic index unavailable: fall back to exact sources only and mark the answer as partial.

### INT-15 Compare reuse, cost, and routing outcomes

Trigger:

- user opens ROI or operations view,
- user selects a seat, playbook, runtime, or date range for comparison.

User sees immediately:

- filters for project, date range, seat, runtime, and playbook,
- counters for saved tokens, reuse count, reissue rate, and average handoff turnaround,
- ranked tables or charts for comparison.

Success path:

1. User chooses filters.
2. Data Engine aggregates audit and budget records.
3. SeatLoom presents comparable outcomes across seats, playbooks, or flows.
4. User opens the underlying evidence for any metric.

Failure path:

- insufficient data: show `Not enough evidence yet` and explain what events must accumulate first;
- stale analytics cache: offer `Refresh metrics` and keep last generated snapshot visible.

## 4. Empty and error state contracts

- No project: show project switcher plus `Open Project` and recent projects.
- No seats yet: show `Create or import a seat identity`.
- No collaboration template: show `Choose how this project coordinates work`.
- No artifacts yet: show `Artifacts appear after sessions, uploads, or review documents are linked`.
- No playbooks yet: show `No reusable lessons published yet`.
- No search results: show `No exact result found yet` plus filter or source hints.
- Budget exhausted: show reason, consumed amount, and the next safe actions.
- All failures must provide reason plus next-step guidance. Console-only failure is not allowed.

## 5. Interaction acceptance checkpoints

- Every P0 flow must be runnable end-to-end without leaving the product for its primary action.
- Every P0 continuity flow must show context tiers and budget impact before launch or recovery.
- Every P0 evidence-search flow must return deterministic results without LLM dependency.
- Every P1 flow must visibly extend a defined P0 surface rather than introducing a disconnected concept.
- Every P1 why-answer flow must show evidence sources.
- Every P2 flow must preserve human approval and auditability.
- Every mobile P0 flow must reuse desktop object transitions and audit history rather than creating a notification-only side channel.
- Mobile flows must stay bounded to monitoring, quick approval/takeover, and short feedback; full terminal control and heavy editing remain desktop responsibilities.
- Every review-driven follow-up must expose `L1/L2/L3` tier, changed clauses, impact, executor/reviewer, evidence refs, and the correct acknowledgment mode before the loop can close.
- Every P3 topology flow must keep final gate truth centralized until evidence justifies core-seat replication.
