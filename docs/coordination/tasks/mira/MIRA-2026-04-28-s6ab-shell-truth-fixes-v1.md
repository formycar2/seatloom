# MIRA-2026-04-28-S6AB-Shell-Truth-Fixes-v1

| Field | Value |
|---|---|
| template | T2 |
| subtype | fix |
| id | MIRA-2026-04-28-s6ab-shell-truth-fixes-v1 |
| status | active |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| depends_on | `docs/coordination/acceptance/2026-04-28-mira-s6-interaction-baseline-acceptance.md` |
| supersedes | none |
| tags | mira, ui, s6, shell, truth, shortcut, follow-up |
| Owner | Mira |
| Issued by | Lyra |
| Acceptance owner | Lyra |
| Stage | S6 return packet |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Concurrency rule | One active packet only for this return; do not run internal parallel edits |

## 1. Purpose

Close the two returned gaps from the S6 acceptance review without reopening the accepted search work.

This is a bounded truth-fix packet, not a redesign:

1. remove fabricated shell-health values from `Project Overview`,
2. close the missing Inbox session -> terminal auto-open path,
3. remove unsupported shortcut help content,
4. clean touched labels back to Chinese-first consistency.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/ux-spec-v1.1.md`
5. `docs/acceptance-spec-v1.1.md`
6. `docs/coordination/acceptance/2026-04-28-mira-s6-interaction-baseline-acceptance.md`
7. this packet

Focus clauses:

- `docs/ux-spec-v1.1.md:77`
- `docs/ux-spec-v1.1.md:110`
- `docs/acceptance-spec-v1.1.md:134`
- `docs/coordination/acceptance/2026-04-28-mira-s6-interaction-baseline-acceptance.md:39`

## 3. Scope

Primary write targets:

- `ui/src/components/ProjectOverview.tsx`
- `ui/src/App.tsx`
- `ui/src/components/ShortcutHelpDialog.tsx`

Do not reopen or touch:

- `ui/src/views/InboxView.tsx`
- `ui/src/views/WorkItemsView.tsx`
- theme preset files
- unrelated shell layout files

## 4. Required outcome

### 4.1 Project Overview shell truth

The default Detail surface must remain `Project Overview`, but the displayed health values must be truthful.

Required behavior:

- `running sessions`, `blocked sessions`, `open WorkItems`, `pending Inbox`, and `last reconcile` may continue to be computed from seeded store data;
- `unresolved review threads` must be derived from current seeded prototype data instead of a fixed literal;
- `budget alert state` must not be fabricated:
  - derive it from real seeded data **only if** the current model already supports a truthful rule, or
  - show a visibly explicit unavailable / not-yet-indexed state instead of pretending the budget is healthy;
- if the current collaboration-template label is not project-derived, present it as a fixed baseline label clearly, not as a fake dynamic metric;
- touched labels should remain Chinese-first.

Implementation guidance:

- Prefer deterministic projections from the existing `useDataStore` seed.
- Do not add new backend state or large model changes in this packet.
- Do not invent thresholds that the current prototype does not actually model.

### 4.2 Inbox session drill-through

When an Inbox row resolves to a `Running` or `InputRequired` session, the bottom terminal panel must auto-open just like the regular session-selection path already does.

Rules:

- apply only to session rows,
- only auto-open for `Running` or `InputRequired`,
- keep WorkItem / Handoff / Artifact Inbox behavior unchanged.

### 4.3 Shortcut help truth

The shortcut dialog must list only real supported shortcuts.

Required action:

- remove `Tab` unless you implement real `Tab` behavior in the same packet.

Preferred action:

- keep the help dialog compact and Chinese-first,
- clean touched labels if they are still mixed-language noise.

## 5. Guardrails

Do not:

- redesign the shell,
- add a new dashboard,
- change the accepted `S6C` search behavior,
- add fake metrics,
- introduce new LLM-dependent logic,
- start broader typography or theme work from this packet.

## 6. Acceptance criteria

All must be true:

1. `Project Overview` no longer contains fabricated shell-health values;
2. any unsupported health field now shows a truthful unavailable state instead of a fake positive state;
3. Inbox-linked running or input-blocked sessions auto-open the terminal panel;
4. shortcut help lists only implemented shortcuts;
5. touched labels remain Chinese-first and readable;
6. accepted `S6C` behavior does not regress;
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

- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6ab-shell-truth-fixes-delivery-v1.md`

Required sections:

1. Scope completed
2. Changed files
3. Truth fixes applied
4. Build result
5. Blockers
6. Evidence paths

## 10. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s6ab.txt
[Mira -> Lyra] S6AB Shell Truth Fixes
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-s6ab-shell-truth-fixes-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s6ab /tmp/mira_to_lyra_s6ab.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s6ab
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
