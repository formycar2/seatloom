# Interaction Design Comprehensive Review

| Item | Content |
|------|---------|
| Document | Interaction Design Review |
| Status | Pending Lyra Review |
| Author | Aegis |
| Date | 2026-04-28 |
| Scope | interaction-spec-v1.0, ux-spec-v1.0, prd-v0.4, mvp-scenarios-v2.0 |

---

## 0. Overall Assessment

The current interaction design has an implementable foundation for **layout skeleton, core flow coverage, and status expression**, but significant gaps remain in **flow depth, exception branch completeness, cross-document consistency, and user cognitive load control**.

**One-line verdict: Skeleton 70/100, Details 40/100. Can scaffold, cannot ship.**

---

## 1. Strengths (What Is Done Well)

1. **Clear layout skeleton**: Sidebar / Main / Detail / Terminal / Status — five zones with distinct responsibilities
2. **Complete status indicator system**: Color + dot + text triple expression, not single-dimension dependent
3. **Inbox defined as action queue**: Enter/leave/priority rules are contractualized
4. **Detail Pane templated**: 6 object types each have a structured template
5. **LaunchPack preview flow**: Select runtime → preview → confirm → launch, steps are clear
6. **Morning Digest design is reasonable**: Auto reconcile on launch + digest banner

---

## 2. Critical Issues (Must Fix)

### C1. Multi-project switching lacks layout definition in UX Spec

PRD v0.4 §13 defines the capability, interaction-spec §3.0 defines the flow, but **ux-spec has no screen spec for All Projects view or Project Switcher**. Title Bar only mentions "app name + window controls" with no project name or switch entry point. Mira cannot implement this.

**Recommendation**: UX Spec must add Project Switcher component spec (position within Title Bar, dropdown structure, Recent/Pin list, switch protection dialog) and All Projects view spec.

### C2. Timeline still displays internal event names

PRD v0.4 §7 explicitly prohibits "showing only `handoff.sent` without a readable summary," but ux-spec §5.1 and mvp-scenarios §4 examples still directly show `session.started`, `artifact.created`, `handoff.sent`.

**Recommendation**: Add an "event type → human-readable template" mapping table covering at least 20 core events. Examples:
- `session.started` → "Nimbus started a new Codex session"
- `handoff.sent` → "Lyra sent a handoff to Nimbus for WI-012"
- `artifact.created` → "Nimbus produced a diff summary for OAuth callback"
- `pipeline.completed` → "Verification pipeline finished with FAIL result"
- `reconcile.drift_detected` → "WI-010 marked done but branch not merged"

### C3. Inbox action downstream state linkage undefined

Interaction-spec §3.5 only states "item status updates and removes from list," but does not define:
- Does Accept on a Handoff change the associated WorkItem status?
- After Return, what state does the Handoff revert to? Does a new Inbox item appear for the sender?
- Does Dismiss write a Ledger event?
- After Resolve on a blocked WorkItem, what is the next status?

**Recommendation**: Add a downstream state linkage table for each Inbox action:

| Action | Object | State Change | Ledger Event | Downstream Side Effects |
|--------|--------|-------------|-------------|------------------------|
| Accept (Handoff) | Handoff | sent → accepted | `handoff.accepted` | WorkItem stays active; sender gets receipt |
| Return (Handoff) | Handoff | sent → returned | `handoff.returned` | Sender Inbox gets "Handoff returned" item |
| Dismiss (Drift) | WorkItem | drifted (unchanged) | `inbox.dismissed` | No state change, suppresses re-entry for 24h |
| Resolve (Blocked) | WorkItem | blocked → active | `workitem.unblocked` | Removes blocking dependency |

### C4. Detail Pane defaults to blank, wasting 360px

UX Spec §7 defines Detail Pane as empty when nothing is selected. After opening the app, the right 360px is completely blank — extremely low information density.

**Recommendation**: When nothing is selected, default to a "Project Overview" card:
- Active Seats count + names
- Running Sessions count
- Open WorkItems count
- Pending Inbox count
- Last reconcile time

Alternatively, auto-select the first Inbox item on launch.

---

## 3. High Issues (Impact Experience Completeness)

### H1. Sidebar and Main Panel navigation overlap

Sidebar has an Inbox entry; Main Panel also has an Inbox Tab. Is clicking Sidebar Inbox identical to clicking Main Panel Inbox Tab? When clicking a Seat in Sidebar, Main Panel stays unchanged but Timeline auto-filters — this linkage rule is not intuitive.

**Recommendation**: Clarify "Sidebar = navigation + filter" vs "Main Panel Tabs = view switcher." Provide an explicit linkage matrix:

| Sidebar Click | Main Panel Behavior | Detail Pane Behavior |
|---------------|--------------------|--------------------|
| Inbox | Switch to Inbox tab | Clear selection |
| Seat | No tab change | Show Seat detail |
| Session | No tab change | Show Session detail |
| WorkItem | Switch to Timeline tab, filter by WI | Show WorkItem detail |

### H2. Too many dialogs, missing inline editing paths

Current design uses modal dialogs for: init, add seat, attach, wrap, workitem, handoff, switch runtime, launchpack preview. For frequent operations, dialog fatigue is severe.

**Recommendation**: High-frequency operations (create WorkItem, process Inbox items) should prioritize inline interaction (edit directly within Detail Pane). Reserve modal dialogs for low-frequency or multi-step operations (init, attach, switch runtime).

### H3. Session-to-Terminal relationship is not intuitive

Clicking a Session in Sidebar opens Detail Pane, but viewing live output requires manually switching to Terminal Panel. User expectation: "click Session = see what it's doing."

**Recommendation**: Clicking a running session should auto-expand Terminal Panel and switch to the corresponding tab. If Terminal is collapsed, auto-expand it.

### H4. Keyboard shortcut system lacks discoverability

10+ shortcuts defined but no discovery mechanism (no `?` help panel, no tooltips, no onboarding hints).

**Recommendation**: Add `Cmd/Ctrl+?` or `F1` shortcut help panel. On first use, show hint in Status Bar: "Press ? for keyboard shortcuts."

### H5. WorkItem state transitions lack visualization

PRD v0.4 §5.1 defines a complete state machine, but UI only shows dot color changes. User cannot see "what can I do next" or "why can't I do this."

**Recommendation**: In WorkItem Detail, add "available transition" button group. For example, in `active` state show `Mark as In Review` / `Mark as Blocked`. When gate is not satisfied, button is disabled + tooltip explains why (e.g., "Requires at least 1 artifact to move to in_review").

### H6. Artifact lacks creation/upload entry

Currently Artifacts only appear through Session output or Handoff attachment. Users have no way to manually create/upload an Artifact. In practice, users frequently need to attach design docs, screenshots, etc.

**Recommendation**: Add `+ Add Artifact` button in WorkItem Detail, supporting file system selection or content paste.

---

## 4. Medium Issues (Experience Polish)

### M1. Responsive rules only cover down to 1024px

No behavior defined for < 768px. Although this is a desktop app, windows can be resized small. Define minimum usable width (e.g., 800px) and behavior below that threshold.

### M2. Toast notification stacking rules undefined

§11.6 defines toast, but does not define stacking/queuing behavior when multiple toasts appear simultaneously. Recommend: max 3 visible, stack vertically with 8px gap, oldest auto-dismiss first.

### M3. Search only defined for Timeline

Inbox and WorkItems views have no search capability. At minimum, add WorkItems search (by title/id) and Inbox search (by summary text).

### M4. Handoff `expired` status has no trigger condition

PRD §5.3 lists `expired`, but defines no timeout duration or auto-expiry rules. Define: default expiry = 7 days after `sent`, configurable per-handoff. Expired handoffs appear as Low priority in sender's Inbox.

### M5. CJK/Latin mixed typesetting rules missing

Font spec only defines system sans-serif. No rules for CJK/Latin spacing, punctuation handling, or spacing between IDs and Chinese text. Actual rendering will look rough.

**Recommendation**: Define: 1 space between Latin/CJK boundaries when no punctuation; IDs (`WI-012`, `SES-005`) always use monospace font; punctuation follows CJK conventions in Chinese context.

### M6. Pipeline view lacks state preservation on "Back"

Pipeline view replaces Main Panel content. When user clicks "Back to WorkItems," whether the previous WorkItems view state (grouping/sorting/selected item) is preserved is undefined.

**Recommendation**: Treat Pipeline view as a stack push. "Back" pops the stack and restores previous view state.

---

## 5. Low Issues (Detail Optimization)

- **L1.** Status Bar information density can be improved: add active session count, today's Inbox processed count
- **L2.** Sidebar collapsed section default state undefined: all expanded? Only sections with active items expanded?
- **L3.** Color system lacks `info` semantic color (blue is used for in_review/launching; no independent info color for informational content like tooltips/hints)
- **L4.** Dialog Esc close may conflict with Detail Pane Esc close (if dialog is above Detail Pane); define z-index priority: Dialog Esc takes precedence

---

## 6. Cross-Document Consistency Issues

| Conflict | Location |
|----------|----------|
| LaunchPack fallback L3 (clipboard) exists in mvp-scenarios §8, but PRD v0.4 §9.3 already removed it | Unify: remove L3 from mvp-scenarios |
| ux-spec dependency declaration still references PRD v0.3, should be updated to v0.4 | ux-spec metadata row |
| mvp-scenarios Scene 7 (Handoff) is Phase 3, but PRD v0.4 treats it as a core flow | Confirm priority — if core, promote to Phase 1-2 |
| interaction-spec §3.0 multi-project switching has no corresponding scene in mvp-scenarios | Add Scene 14: Multi-project Switching |

---

## 7. Priority Fix List

| Priority | ID | Fix Item | Recommended Owner |
|----------|-----|----------|-------------------|
| Critical | C1 | Add Project Switcher + All Projects screen spec | Mira (UX) |
| Critical | C2 | Add event type → human-readable template mapping table | Lyra (define) + Mira (render) |
| Critical | C3 | Add Inbox action → downstream state linkage table | Lyra (define) |
| Critical | C4 | Detail Pane default content (not blank) | Mira (UX) |
| High | H1 | Sidebar ↔ Main Panel linkage matrix | Lyra (define) |
| High | H2 | Convert high-freq operations to inline interaction | Mira (UX) |
| High | H3 | Click running session auto-expands Terminal | Mira (UX) |
| High | H4 | Keyboard shortcut help panel | Mira (UX) |
| High | H5 | WorkItem available-transition button group | Mira (UX) |
| High | H6 | Artifact manual creation/upload entry | Mira (UX) |
| Medium | M1 | Define minimum window width behavior | Mira (UX) |
| Medium | M2 | Toast stacking rules | Mira (UX) |
| Medium | M3 | Search for Inbox + WorkItems views | Mira (UX) |
| Medium | M4 | Handoff expiry trigger condition | Lyra (define) |
| Medium | M5 | CJK/Latin typesetting rules | Mira (UX) |
| Medium | M6 | Pipeline view back-state preservation | Mira (UX) |

---

## 8. Recommended Next Steps

1. Lyra reviews this document and confirms/disputes each item
2. Confirmed items get converted to task packets for Mira (UX fixes) and Lyra (contract fixes)
3. Cross-document conflicts resolved before Baseline Freeze
4. After all Critical + High items resolved, re-review before implementation handoff to Nimbus

---

*This review is authored by Aegis. Lyra should review and respond with accept/dispute per item before actionable task packets are created.*
