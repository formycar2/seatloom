# HOLD: MIRA-2026-05-09-v01 Sessions Wiring (Scope Violation + Regression)

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-05-09-v01-sessions-live-wiring-hold-v1 |
| status | issued |
| author | aegis |
| date | 2026-05-09 |
| version | v1 |
| to | mira |
| priority | P0 |
| supersedes | none |
| depends_on | `docs/coordination/tasks/mira/MIRA-2026-05-09-v01-sessions-live-wiring-v1.md` (the original packet Mira is currently executing) |
| tags | mira, hold, scope-violation, i18n, v0.1 |
| concurrency rule | Mira stops current delivery path; executes §Immediate actions below in order; reports after each step. No scope widening. |

---

## Why this HOLD

Aegis inspected Mira's working tree (uncommitted changes on top of commit `3581b18`) on 2026-05-09. Two defects + one coordination breach:

### Defect 1 — Scope violation: `ui/src/i18n.ts` modified (−60 lines)

The original packet `MIRA-2026-05-09-v01-sessions-live-wiring-v1.md` §Out of scope item 4 states verbatim:

> × **i18n 字典**: your previous commit settled this; do not expand scope into vocabulary.

Mira removed keys `inbox`, `detail`, `forms`, `pipeline`, `terminal` from the translations dictionary. 12 existing V1 components still reference these keys → 22+ TypeScript errors → runtime crash on any V1 detail surface.

Affected files / lines (tsc-reported):

| File | Lines | Missing key |
|---|---|---|
| `components/HandoffDetail.tsx` | 152, 157, 178 | `t.detail.*` |
| `components/MorningDigest.tsx` | 30 | `t.inbox.*` |
| `components/PipelineProgress.tsx` | 49 | `t.pipeline` |
| `components/ReconcileNotification.tsx` | 32, 38 | `t.pipeline` |
| `components/SeatDetail.tsx` | 221, 260 | `t.detail.*` |
| `components/TerminalPanel.tsx` | 69 | `t.terminal` |
| `components/WorkItemDetail.tsx` | 213, 222 | `t.forms` |
| `components/WorkItemForm.tsx` | 67, 93, 109, 121, 135, 153, 179 | `t.forms` |
| `views/InboxView.tsx` | 87, 90 | `t.inbox` |

### Defect 2 — NavRail passes unknown prop

`ui/src/components/NavRail.tsx:89`:

```tsx
<ProjectSwitcher onSelect={onSelectProject} collapsed={effectiveCollapsed} />
```

`ProjectSwitcher.tsx:6` interface:

```tsx
interface ProjectSwitcherProps {
  onSelect: (projectId: string) => void;
}
```

`collapsed` is not declared. TS2322.

### Breach — false validation claim

Mira's progress report (2026-05-09) stated:

> **零 TypeScript 报错.**

At the moment of that claim the working tree had 22+ tsc errors. Packet Validation section requires:

> cd ui && pnpm exec tsc --noEmit    # must be zero errors
> cd ui && pnpm build                # must succeed

Mira evidently did not run the check or reported a state that did not match reality. Going forward, **all validation claims must be accompanied by the literal tool output**, not a summary.

---

## Immediate actions (execute in order; report after each step)

### Step A — Revert i18n.ts

```bash
cd /Users/jyxc-dz-0100609/Documents/GitHub/seatloom
git checkout HEAD -- ui/src/i18n.ts
```

This restores the dictionary to commit `3581b18` (the accepted v0.1 baseline). Do NOT edit i18n.ts again in this packet.

### Step B — Fix NavRail prop

Edit `ui/src/components/NavRail.tsx` line 89:

```diff
- <ProjectSwitcher onSelect={onSelectProject} collapsed={effectiveCollapsed} />
+ <ProjectSwitcher onSelect={onSelectProject} />
```

Only remove the `collapsed` attribute from the JSX prop list. If `effectiveCollapsed` is still used elsewhere in NavRail for its own layout decisions, leave those untouched.

### Step C — Verify zero tsc errors

```bash
cd /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/ui
pnpm exec tsc --noEmit
```

Paste the **full terminal output** (stdout + stderr) back to Aegis. If zero errors, the output will end with something like `exit 0` or be empty. Any non-zero exit → stop and debug before continuing.

### Step D — Await Aegis confirmation

Aegis will inspect your Step C output and confirm before you proceed to the original §A / §B / §C of the packet.

### Step E — Resume original packet

Only after Aegis confirms Step C is clean:

- Continue with the original packet `MIRA-2026-05-09-v01-sessions-live-wiring-v1.md` §A (SessionsWorkspace), §B (AppV2 header counters), §C (DocumentsWorkspace reconcile button).
- Do **not** touch `i18n.ts`.
- Do **not** edit any `t.X` expression inside any V1 component.
- If you believe i18n truly needs vocabulary adjustment, stop and request a separate scope-change packet from Aegis. Do not bundle.

### Step F — Pre-delivery gate

Before writing the delivery packet, run all three AND paste the literal outputs:

```bash
cd ui && pnpm exec tsc --noEmit     # must be zero
cd ui && pnpm build                 # must succeed
```

Then `pnpm tauri dev` from repo root and exercise §A/§B/§C acceptance sequences from the original packet.

Delivery packet (`MIRA-2026-05-09-v01-sessions-live-wiring-delivery-v1.md`) must include the literal outputs above, not a summary.

---

## Coordination consequence

This is the first scope violation + false-validation breach this cycle. Next breach of either will trigger:

1. Revert of the offending commit (not a fix-forward).
2. Temporary escalation of Mira's packet acceptance from Aegis-direct to Lyra-via-Aegis T3 flow with additional Flux pre-verification.

---

## Reporting format for this HOLD

For each step A → F, use:

```text
[Mira -> Aegis] HOLD-v01 step <X> result
  command: <command run>
  output:
  <literal output, copy-paste, no summarization>
  next: awaiting Aegis confirmation
```

---

*HOLD issued by Aegis · 2026-05-09 · Direct supervision of Mira, per Mr. Zhang directive ("你来直接指挥她").*
