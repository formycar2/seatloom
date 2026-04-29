# MIRA-2026-04-28-Theme-Preset-System-v1

| Field | Value |
|---|---|
| template | T2 |
| subtype | task_packet |
| id | MIRA-2026-04-28-theme-preset-system-v1 |
| status | active |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| depends_on | `docs/coordination/acceptance/2026-04-28-lyra-mira-s5f-seeded-visibility-acceptance.md` |
| supersedes | `docs/coordination/tasks/mira/MIRA-2026-04-28-light-theme-rebase-v1.md` |
| tags | mira, ui, theme, preset-system, visual-rebase |
| Owner | Mira |
| Issued by | Lyra |
| Acceptance owner | Lyra |
| Stage | Post-S5 visual system rebase |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Theme-token refactor + 3 preset themes / no product-flow changes |

## 1. Purpose

Stop doing one-off per-component recoloring.

The prototype should move to a **preset theme system**:

- one semantic theme token layer,
- multiple pre-defined visual presets,
- a lightweight in-app selector,
- persistent user preference,
- no arbitrary color editing.

This gives us two benefits at once:

1. visual quality improves because color decisions are centralized instead of drifting component by component;
2. the team can evaluate multiple visual directions quickly without reopening product meaning.

## 2. Start condition

Start this packet now.

This packet **supersedes** `docs/coordination/tasks/mira/MIRA-2026-04-28-light-theme-rebase-v1.md`.

Do not continue the old packet as a single-palette hardcoded recolor pass.

## 3. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/interaction-spec-v1.1.md`
5. `docs/coordination/acceptance/2026-04-28-lyra-mira-s5f-seeded-visibility-acceptance.md`
6. this packet

## 4. Mandatory pre-start close-out from S5F

Before theme work is considered complete, close these small copy-hygiene deltas inside the same packet:

- localize the remaining operator-facing English strings in `ui/src/stores/useDataStore.ts` and `ui/src/mockData.ts` for:
  - `ses-407` prompt preview,
  - `ses-407` expected-next,
  - `ses-408` `current_blocker`,
  - `ses-408` `seat_skills`,
  - `ses-408` `playbook_matches`;
- correct the compatibility typo in `ui/src/mockData.ts`:
  - `不再出现明显中英混杂福利` -> `不再出现明显中英混杂`

This is copy hygiene only. Do not change the seeded product meaning.

## 5. Required outcome

Implement a small preset-theme system with **3 pre-defined themes**:

1. `paper-ledger` — default
2. `harbor-blueprint`
3. `sage-archive`

The user must be able to choose among these presets in the prototype.

The system must be token-driven, not hardcoded component by component.

## 6. Preset definitions

### 6.1 `paper-ledger` (default)

| Token | Hex |
|---|---|
| Canvas | `#F6F1E8` |
| Surface | `#FFFDF8` |
| Surface Subtle | `#F0E8DC` |
| Border | `#DDD2C2` |
| Ink | `#1F2A37` |
| Ink Soft | `#5E6A75` |
| Ink Faint | `#8C97A3` |
| Primary | `#1F6B75` |
| Primary Hover | `#175560` |
| Primary Tint | `#DCECEF` |
| Success | `#2E8B57` |
| Warning | `#C77B18` |
| Error | `#C3513A` |
| Done | `#6C7A89` |
| Drifted | `#5F78B9` |
| Focus Ring | `#8DB9C0` |
| Terminal Preview Only | `#20262E` |

### 6.2 `harbor-blueprint`

| Token | Hex |
|---|---|
| Canvas | `#EEF3F6` |
| Surface | `#FCFEFF` |
| Surface Subtle | `#E2EBF0` |
| Border | `#C8D4DC` |
| Ink | `#1B2C3A` |
| Ink Soft | `#556776` |
| Ink Faint | `#7C8B98` |
| Primary | `#245A7A` |
| Primary Hover | `#1D4760` |
| Primary Tint | `#D9E8F2` |
| Success | `#2F7D5A` |
| Warning | `#B7791F` |
| Error | `#C0543F` |
| Done | `#667788` |
| Drifted | `#4F6FA8` |
| Focus Ring | `#89AFC5` |
| Terminal Preview Only | `#20262E` |

### 6.3 `sage-archive`

| Token | Hex |
|---|---|
| Canvas | `#F1F3EC` |
| Surface | `#FCFDF9` |
| Surface Subtle | `#E6EBDD` |
| Border | `#CFD7C5` |
| Ink | `#243128` |
| Ink Soft | `#5D6A60` |
| Ink Faint | `#879187` |
| Primary | `#4C6B4E` |
| Primary Hover | `#3D5540` |
| Primary Tint | `#DDE7DA` |
| Success | `#3E7C59` |
| Warning | `#B9852A` |
| Error | `#B85A46` |
| Done | `#6F7A70` |
| Drifted | `#6076A6` |
| Focus Ring | `#9CB7A0` |
| Terminal Preview Only | `#20262E` |

## 7. Visual rules

1. All three presets must remain **light-first**.
2. No preset may return the app to a black-shell / dark-dashboard first-load feeling.
3. The only intentionally dark area allowed is the bounded terminal / prompt preview block.
4. Dense Chinese text must remain easy to read in every preset.
5. Status distinction must remain clear in every preset.
6. Prompt-blocked and continuity surfaces must remain emphasized, but through light tinted treatment.

## 8. Architecture rules

1. Centralize theme definitions in the theme system.
2. Use semantic tokens / CSS variables; do not scatter new hardcoded hex values across components.
3. Existing components may be patched only to remove hardcoded dark assumptions or to consume semantic tokens properly.
4. If a component needs special treatment, make it token-based rather than one-off palette logic.
5. Persist the selected preset in local storage.
6. Invalid or missing stored value must fall back to `paper-ledger`.

## 9. Required implementation targets

At minimum, inspect and update:

- `ui/src/styles/theme.ts`
- `ui/src/styles/globals.css`
- `ui/src/App.tsx`
- `ui/src/layouts/Sidebar.tsx`

Then patch other files only where required to remove hardcoded dark assumptions.

## 10. Theme selector requirement

Add a **small, quiet preset selector** to the prototype.

Preferred placement:

- sidebar footer, or
- another similarly low-noise shell location.

Rules:

- it must expose the 3 preset names;
- it must be visually compact;
- it must not become a settings workflow;
- it must switch the preset immediately;
- it must persist the choice.

## 11. Non-goals

Do **not**:

- create an arbitrary color picker,
- add a full settings page,
- redesign layout hierarchy,
- change product behavior,
- change seeded product meaning,
- invent new features beyond the preset selector,
- reopen information architecture.

This is a **visual system upgrade**, not a product rewrite.

## 12. Acceptance criteria

All must be true:

1. The visual system is tokenized rather than implemented as scattered one-off recolors.
2. The prototype exposes exactly 3 preset themes: `paper-ledger`, `harbor-blueprint`, `sage-archive`.
3. `paper-ledger` is the default first-load theme.
4. The user can switch themes in-app.
5. The selected theme persists across refresh/reload.
6. The main shell no longer reads as dark-theme UI in any preset.
7. Dense Chinese text remains readable across the repaired surfaces.
8. Prompt-blocked and continuity surfaces still stand out appropriately.
9. The S5F copy-hygiene delta is closed.
10. Build passes.

## 13. Validation

Run:

```bash
cd ui && pnpm build
```

## 14. Done definition

All must be true:

- the theme system is preset-based,
- the old single-theme hardcoded pass is no longer the implementation approach,
- all 3 presets work,
- the selector works and persists,
- S5F copy cleanup is included,
- build result is reported,
- tmux reply is sent to Lyra,
- stop and wait for acceptance.

## 15. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] Theme Preset System
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-theme-preset-system-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
