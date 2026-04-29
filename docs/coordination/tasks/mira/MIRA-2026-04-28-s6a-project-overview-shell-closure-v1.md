# MIRA-2026-04-28-S6A-Project-Overview-Shell-Closure-v1

| Field | Value |
|---|---|
| template | T2 |
| subtype | task_packet |
| id | MIRA-2026-04-28-s6a-project-overview-shell-closure-v1 |
| status | active |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| depends_on | `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md` |
| supersedes | none |
| tags | mira, ui, shell, detail-pane, project-overview, session, terminal |
| Owner | Mira |
| Issued by | Lyra |
| Acceptance owner | Lyra |
| Stage | Post-theme interaction baseline closure |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Concurrency rule | May run in parallel with `S6B` and `S6C`; do not exceed 3 active packets total |

## 1. Purpose

Close two accepted shell-level gaps that remain after the theme preset close-out:

1. default Detail content is still effectively blank instead of `Project Overview`;
2. running-session drill-through still takes too many steps because live evidence does not automatically reveal the Terminal panel.

This packet is a shell-behavior closure slice, not a new product feature wave.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/ux-spec-v1.1.md`
5. `docs/acceptance-spec-v1.1.md`
6. `docs/coordination/reviews/2026-04-28-lyra-review-response.md`
7. `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md`
8. this packet

Focus clauses:

- `docs/ux-spec-v1.1.md:78`
- `docs/ux-spec-v1.1.md:110`
- `docs/coordination/reviews/2026-04-28-lyra-review-response.md:31`
- `docs/coordination/reviews/2026-04-28-lyra-review-response.md:39`

## 3. Scope

Primary write targets:

- `ui/src/App.tsx`
- `ui/src/components/ProjectOverview.tsx` (new file allowed)

Touch other UI files only if required to keep the shell coherent.

## 4. Required outcome

### 4.1 Default Detail = Project Overview

When no object is selected and no create/run form is open, the Detail pane must show a real `Project Overview` surface instead of a generic empty-state message.

Minimum visible content:

- current project name,
- current project path,
- active collaboration template,
- running session count,
- suspended or blocked session count,
- open WorkItem count,
- pending Inbox count,
- last reconcile time,
- unresolved review thread count,
- current budget alert state.

Implementation rule:

- Keep the body Chinese-first for operator readability.
- Keep the overview deterministic and seeded from current prototype data.
- Do not invent large new navigation or a dashboard redesign.

### 4.2 Running session selection auto-reveals live evidence

When the user selects a running or input-blocked session from an existing interactive surface, SeatLoom should auto-open the bottom Terminal / Session Panel so the user reaches live evidence with one fewer step.

Rules:

- Apply this primarily to running or prompt-blocked sessions.
- Historical / completed sessions may remain detail-first.
- Do not auto-open the terminal for every object type.
- Do not change the bottom-panel architecture.

Implementation hint:

- `handleSelectObject` is the likely entry point.
- Preserve already-accepted selection behavior for Seats, WorkItems, Handoffs, and Artifacts.

## 5. Guardrails

Do not:

- redesign the shell layout,
- turn Project Overview into a new workspace tab,
- auto-select an Inbox item by default,
- change seeded product meaning,
- reopen theme work,
- add LLM-dependent logic for summary content.

## 6. Acceptance criteria

All must be true:

1. no-selection Detail state is a real `Project Overview` surface;
2. the overview shows the required project-health fields listed above;
3. the body remains Chinese-first and consistent with the current prototype tone;
4. selecting a running or prompt-blocked session auto-opens the Terminal panel;
5. completed or historical sessions do not force terminal auto-open;
6. already-accepted shell behavior does not regress;
7. build passes.

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

- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6a-project-overview-shell-closure-delivery-v1.md`

Required sections:

1. Scope completed
2. Changed files
3. Contract coverage
4. Build result
5. Blockers
6. Evidence paths

## 10. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s6a.txt
[Mira -> Lyra] S6A Project Overview + Shell Closure
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-s6a-project-overview-shell-closure-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s6a /tmp/mira_to_lyra_s6a.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s6a
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
