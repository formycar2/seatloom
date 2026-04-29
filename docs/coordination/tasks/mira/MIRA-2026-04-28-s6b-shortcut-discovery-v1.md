# MIRA-2026-04-28-S6B-Shortcut-Discovery-v1

| Field | Value |
|---|---|
| template | T2 |
| subtype | task_packet |
| id | MIRA-2026-04-28-s6b-shortcut-discovery-v1 |
| status | active |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| depends_on | `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md` |
| supersedes | none |
| tags | mira, ui, shortcut, help, discoverability, shell |
| Owner | Mira |
| Issued by | Lyra |
| Acceptance owner | Lyra |
| Stage | Post-theme interaction baseline closure |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Concurrency rule | May run in parallel with `S6A` and `S6C`; do not exceed 3 active packets total |

## 1. Purpose

Close the accepted shortcut discoverability gap.

The shortcut system exists, but it is still too hidden for baseline usability. We need a small, quiet discovery surface rather than a power-user secret.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/ux-spec-v1.1.md`
3. `docs/coordination/reviews/2026-04-28-lyra-review-response.md`
4. `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md`
5. this packet

Focus clauses:

- `docs/coordination/reviews/2026-04-28-lyra-review-response.md:40`
- `docs/coordination/reviews/2026-04-28-interaction-design-review.md:110`

## 3. Scope

Primary write targets:

- `ui/src/hooks/useGlobalShortcuts.ts`
- `ui/src/layouts/StatusBar.tsx`
- `ui/src/components/ShortcutHelpDialog.tsx` (new file allowed)

## 4. Required outcome

Implement a small shortcut discovery surface.

Minimum required behavior:

1. user can open shortcut help via `?` / `Shift+/` or `F1`;
2. help surface lists only real currently supported shortcuts;
3. Status Bar includes a lightweight discoverability hint;
4. the help surface is compact, readable, and Chinese-first.

Current supported shortcuts should at minimum include the real implemented ones:

- `Esc`
- `Cmd/Ctrl + K`
- `Cmd/Ctrl + \``
- any additional shortcut only if you actually implement it in this packet.

Implementation rules:

- Prefer a lightweight dialog, popover, or overlay.
- Do not invent a large command system.
- Do not list fake or future shortcuts.
- If you add `?` open behavior, wire it through the real shortcut hook.

## 5. Guardrails

Do not:

- redesign the Status Bar,
- add onboarding flows,
- add unsupported shortcut entries,
- create a settings page,
- change unrelated shell behavior.

## 6. Acceptance criteria

All must be true:

1. shortcut help is discoverable from the Status Bar;
2. keyboard help opens from `?` / `Shift+/` or `F1`;
3. the help surface lists only supported shortcuts;
4. help copy is Chinese-first and visually legible;
5. no unrelated shell regressions appear;
6. build passes.

## 7. Validation

Run:

```bash
cd ui && pnpm build
```

## 8. Done definition

All must be true:

- code is updated,
- build result is recorded,
- a delivery note is written,
- tmux reply is sent to Lyra,
- stop and wait for acceptance.

## 9. Required delivery artifact

Write:

- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6b-shortcut-discovery-delivery-v1.md`

Required sections:

1. Scope completed
2. Changed files
3. Shortcut list exposed
4. Build result
5. Blockers
6. Evidence paths

## 10. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s6b.txt
[Mira -> Lyra] S6B Shortcut Discovery
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-s6b-shortcut-discovery-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s6b /tmp/mira_to_lyra_s6b.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s6b
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
