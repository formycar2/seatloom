# MIRA-2026-04-28-Theme-Preset-Closeout-v1

| Field | Value |
|---|---|
| template | T2 |
| subtype | task_packet |
| id | MIRA-2026-04-28-theme-preset-closeout-v1 |
| status | active |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| depends_on | `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-system-acceptance.md` |
| supersedes | none |
| tags | mira, ui, theme, closeout, selector, persistence |
| Owner | Mira |
| Issued by | Lyra |
| Acceptance owner | Lyra |
| Stage | Theme preset closure micro-packet |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Concurrency rule | One active packet only for this close-out |

## 1. Purpose

Close the two remaining acceptance gaps in the preset-theme system without reopening palette work or product behavior.

This is a **finish-line micro-packet** only.

## 2. Start condition

Start now.

Read the acceptance artifact first:

- `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-system-acceptance.md`

Do not start any other new UI packet in parallel.

## 3. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/coordination/tasks/mira/MIRA-2026-04-28-theme-preset-system-v1.md`
3. `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-system-acceptance.md`
4. this packet

## 4. Required fixes

### 4.1 Make preset identity visible in the selector

Current issue:

- the selector shows anonymous bars only;
- the preset names are hidden in `title` text.

Required outcome:

- render a compact visible label for each preset in the selector;
- keep the selector quiet and shell-level, not a settings workflow;
- keep the shell Chinese-first.

Use these visible labels:

- `纸账本` for `paper-ledger`
- `港湾蓝图` for `harbor-blueprint`
- `鼠尾档案` for `sage-archive`

Implementation rule:

- move selector-facing preset metadata into a shared exported registry in `ui/src/styles/theme.ts`;
- `ui/src/layouts/Sidebar.tsx` must render from that registry instead of an inline hardcoded array.

You may keep the exact preset IDs as tooltip or muted secondary metadata, but the user must be able to distinguish the three presets visually without hover.

### 4.2 Add safe persisted-value fallback

Current issue:

- persisted `themePreset` values are not validated on rehydrate.

Required outcome:

- any missing or invalid stored value must normalize to `paper-ledger`;
- the UI must read the normalized value after rehydrate;
- the selected theme must still persist normally for valid values.

Implementation rule:

- solve this in `ui/src/stores/useAppStore.ts`;
- keep the logic small and deterministic.

## 5. Guardrails

Do not:

- change palette values,
- add more presets,
- reopen component-by-component recolor work,
- redesign shell layout,
- change product flows,
- introduce a settings page.

## 6. Required file scope

Edit only if needed:

- `ui/src/styles/theme.ts`
- `ui/src/stores/useAppStore.ts`
- `ui/src/layouts/Sidebar.tsx`

Touch other files only if blocked by typing/runtime correctness.

## 7. Acceptance criteria

All must be true:

1. the selector visibly exposes all three presets;
2. the selector remains compact and low-noise;
3. the selector is rendered from shared preset metadata, not an inline duplicated preset list;
4. invalid or missing stored values normalize to `paper-ledger`;
5. valid selection still persists across reload;
6. build passes;
7. tmux reply is sent to Lyra;
8. stop and wait for acceptance.

## 8. Validation

Run:

```bash
cd ui && pnpm build
```

## 9. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] Theme Preset Close-out
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-theme-preset-closeout-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
