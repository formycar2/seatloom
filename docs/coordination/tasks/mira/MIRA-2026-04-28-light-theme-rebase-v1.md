# MIRA-2026-04-28-Light-Theme-Rebase-v1

| Field | Value |
|---|---|
| template | T2 |
| subtype | task_packet |
| id | MIRA-2026-04-28-light-theme-rebase-v1 |
| status | superseded |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| depends_on | `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5f-seeded-visibility-patch-v1.md` |
| tags | mira, ui, theme, light-mode, visual-rebase |
| Owner | Mira |
| Issued by | Lyra |
| Acceptance owner | Lyra |
| Stage | Post-S5 visual rebase |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Single-theme rebase / no behavior changes |

> Superseded by `docs/coordination/tasks/mira/MIRA-2026-04-28-theme-preset-system-v1.md`.

## 1. Purpose

Rebase the prototype from a dark-default visual language to a bright, readable, Chinese-text-friendly light theme.

This is not a cosmetic whim. The current dark default makes dense product content feel heavier than it is, reduces reading comfort, and makes the operating-console value of SeatLoom harder to understand at a glance.

The new direction must feel:

- calm,
- high-trust,
- operational,
- document-friendly,
- not neon SaaS,
- not black terminal-first,
- not pastel toy UI.

## 2. Start condition

Do **not** start this packet until:

1. `S5F` has at least a `CONDITIONAL PASS` verdict from Lyra, and
2. Lyra explicitly tells you to begin this packet.

## 3. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/interaction-spec-v1.1.md`
5. this packet

## 3A. Mandatory pre-start close-out from S5F

Before the visual rebase begins, close the small `S5F` acceptance delta inside the same packet.

Edit and verify these copy-only fixes first:

- localize the remaining operator-facing English strings in `ui/src/stores/useDataStore.ts` and `ui/src/mockData.ts` for:
  - the `ses-407` prompt preview,
  - the `ses-407` expected-next text,
  - the `ses-408` `current_blocker`,
  - the `ses-408` `seat_skills`,
  - the `ses-408` `playbook_matches`;
- correct the compatibility typo in `ui/src/mockData.ts`:
  - `不再出现明显中英混杂福利` -> `不再出现明显中英混杂`

This pre-start delta is copy hygiene only. It must not change behavior, structure, or seeded product meaning.

## 4. Non-goals

Do **not**:

- redesign layout hierarchy,
- change product behavior,
- add dark-mode toggle,
- change seeded product meaning,
- invent new features,
- re-open information architecture.

This is a **visual rebase**, not a product rewrite.

## 5. Design direction

### Theme name

**Paper Ledger Light**

### Emotional target

SeatLoom should feel like:

- a live operating ledger on paper,
- a supervisor desk with structured evidence,
- a product console that invites reading,
- a bright workspace for long-form Chinese content.

It should **not** feel like:

- a nighttime developer dashboard,
- a generic fintech black/glass template,
- a candy-colored AI toy.

## 6. Exact palette

Use these values as the canonical palette unless a tiny accessibility adjustment is required.

| Token | Hex | Usage |
|---|---|---|
| Canvas | `#F6F1E8` | app background |
| Surface | `#FFFDF8` | cards / detail panels |
| Surface Subtle | `#F0E8DC` | sidebar / meta panels / soft blocks |
| Border | `#DDD2C2` | default borders |
| Ink | `#1F2A37` | primary text |
| Ink Soft | `#5E6A75` | secondary text |
| Ink Faint | `#8C97A3` | tertiary labels |
| Primary | `#1F6B75` | primary action / key highlight |
| Primary Hover | `#175560` | primary hover |
| Primary Tint | `#DCECEF` | selected rows / soft emphasis |
| Success | `#2E8B57` | active / healthy |
| Warning | `#C77B18` | caution / input required |
| Error | `#C3513A` | destructive / blocked |
| Done | `#6C7A89` | completed / historical |
| Drifted | `#5F78B9` | drifted / reconciliation |
| Focus Ring | `#8DB9C0` | focus outline |
| Terminal Preview Only | `#20262E` | bounded code / terminal preview blocks only |

## 7. Global visual rules

1. The app must open in **light theme by default**.
2. No full-page dark background remains anywhere in the default experience.
3. White or near-white surfaces must carry the reading load; color is for emphasis, not for filling the whole page.
4. Use saturated color mainly for:
   - primary CTA,
   - active selection,
   - status chips,
   - narrow emphasis bars.
5. Banner components should prefer **tinted backgrounds + dark text**, not solid saturated bars with large white text, unless severity requires it.
6. Sidebar must be lighter than today and closer to a paper/planner surface than a terminal rail.
7. Hover states must use warm tint / primary tint, not `bg-black/5` style darkening.
8. Preserve visual density. This is not a spacious marketing page.
9. Keep strong contrast for Chinese text blocks; secondary text must remain readable.
10. The only intentionally dark area allowed is the bounded terminal/prompt preview block.

## 8. Component rules

### 8.1 App shell

- Background becomes `Canvas`.
- Sidebar becomes `Surface Subtle`.
- Main detail region uses `Canvas`, with `Surface` cards floating above it.
- Active navigation row uses `Primary Tint` + `Ink`, not dark fill.

### 8.2 Cards and panels

- Standard cards use `Surface`.
- Meta panels use `Surface Subtle` or a very light tint.
- Borders should feel soft and editorial, not cold blue-gray.
- Use subtle shadows only; no heavy dark glows.

### 8.3 Buttons

- Primary button: `Primary` background, white text.
- Secondary button: `Surface` background, `Border`, `Ink`.
- Quiet button / ghost: transparent or `Surface Subtle`, no dark hover.
- Destructive button: soft error tint by default, solid `Error` only when emphasis is necessary.

### 8.4 Status language

- `Active`: green tint + dark green text.
- `Warning/Input required`: warm amber tint + dark amber text.
- `Error/Blocked`: soft terracotta tint + dark terracotta text.
- `Done`: cool slate tint + dark slate text.
- `Drifted`: muted steel-blue tint + dark steel-blue text.

Do not overuse bright status fills as generic decoration.

### 8.5 Prompt-blocked and continuity sections

- `Prompt blocked` should read as alert, but not as a black/yellow hazard wall.
- Use light warning tint background with dark text and compact emphasis chips.
- `Continuity Pack Preview` should feel trustworthy and structured:
  - light teal-tinted container,
  - white inner tier cards,
  - dark text,
  - restrained accent usage.

### 8.6 Typography-color relationship

- Long paragraphs must sit on light surfaces with `Ink` / `Ink Soft`.
- Avoid light gray text on tinted backgrounds.
- Avoid white text except on solid primary / solid destructive buttons or very small status pills.

## 9. Required implementation targets

At minimum, inspect and update:

- `ui/index.html`
- `ui/src/styles/globals.css`
- `ui/src/styles/theme.ts`

Then repair any hardcoded dark assumptions in:

- `ui/src/layouts/*`
- `ui/src/views/*`
- `ui/src/components/*`

only where needed to align with this theme.

## 10. Likely hardcoded dark assumptions to fix

Search and normalize cases like:

- `data-theme="dark"`
- `bg-black/...`
- `text-white`
- solid warning / primary bars with large white headings
- hover states that darken with black overlays
- shadows tuned for dark surfaces rather than paper surfaces

## 11. Acceptance criteria

All must be true:

1. First load is light by default.
2. No main shell area reads as dark-theme UI.
3. Sidebar, canvas, and cards form a clear light hierarchy.
4. Dense Chinese text is easier to read than before.
5. Statuses remain clearly distinguishable.
6. Prompt-blocked and continuity surfaces still stand out, but with lighter treatment.
7. No product behavior changes.
8. Build passes.

## 12. Validation

Run:

```bash
cd ui && pnpm build
```

## 13. Done definition

All must be true:

- the visual default is clearly light,
- the palette matches this packet closely,
- the default shell no longer feels dark,
- current repaired surfaces still read clearly,
- build result is reported,
- tmux reply is sent to Lyra,
- stop and wait for acceptance.

## 14. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra.txt
[Mira -> Lyra] Light Theme Rebase
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-light-theme-rebase-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra /tmp/mira_to_lyra.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
