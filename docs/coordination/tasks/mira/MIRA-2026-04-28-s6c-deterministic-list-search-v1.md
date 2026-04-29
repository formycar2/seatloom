# MIRA-2026-04-28-S6C-Deterministic-List-Search-v1

| Field | Value |
|---|---|
| template | T2 |
| subtype | task_packet |
| id | MIRA-2026-04-28-s6c-deterministic-list-search-v1 |
| status | active |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| depends_on | `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md` |
| supersedes | none |
| tags | mira, ui, inbox, workitems, search, deterministic |
| Owner | Mira |
| Issued by | Lyra |
| Acceptance owner | Lyra |
| Stage | Post-theme interaction baseline closure |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Concurrency rule | May run in parallel with `S6A` and `S6B`; do not exceed 3 active packets total |

## 1. Purpose

Close the accepted baseline search gap for the two highest-traffic list surfaces outside Timeline.

Timeline already has search and filters. Inbox and WorkItems still need a deterministic text filter so the operator can find known items without hunting visually.

## 2. Contract basis

Read only these refs before editing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/ux-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/coordination/reviews/2026-04-28-lyra-review-response.md`
6. `docs/coordination/acceptance/2026-04-28-lyra-mira-theme-preset-closeout-acceptance.md`
7. this packet

Focus clauses:

- `docs/coordination/reviews/2026-04-28-lyra-review-response.md:50`
- `docs/ux-spec-v1.1.md:650`
- `docs/acceptance-spec-v1.1.md:125`

## 3. Scope

Primary write targets:

- `ui/src/views/InboxView.tsx`
- `ui/src/views/WorkItemsView.tsx`

Touch shared helpers only if truly needed for normalization.

## 4. Required outcome

Add lightweight deterministic search on both list surfaces.

### 4.1 Inbox

Minimum match fields:

- summary text,
- object reference,
- linked seat or owner name if already present in the row data,
- priority / type text if easy to expose from the current data model.

### 4.2 WorkItems

Minimum match fields:

- WorkItem id,
- title,
- goal.

### 4.3 Required list behavior

- filtering is immediate and local;
- no LLM dependency;
- result count remains visible;
- empty search result state is explicit and calm;
- clearing search restores the full list.

Implementation rule:

- keep the UI compact;
- do not turn either view into a full evidence-search panel;
- this is list filtering, not global retrieval.

## 5. Guardrails

Do not:

- add semantic search,
- call any backend,
- redesign Inbox or WorkItems hierarchy,
- make the search controls visually dominant,
- change accepted typed Artifact behavior.

## 6. Acceptance criteria

All must be true:

1. Inbox supports deterministic text filtering;
2. WorkItems supports deterministic text filtering;
3. result count stays visible while filtering;
4. empty search results have a clear no-match state;
5. clearing the query restores the full list;
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

- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6c-deterministic-list-search-delivery-v1.md`

Required sections:

1. Scope completed
2. Changed files
3. Search behavior coverage
4. Build result
5. Blockers
6. Evidence paths

## 10. Direct tmux reply contract

Send this exact format:

```bash
cat <<'MSG' >/tmp/mira_to_lyra_s6c.txt
[Mira -> Lyra] S6C Deterministic List Search
completed:
- ...
build:
- `cd ui && pnpm build` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-04-28-s6c-deterministic-list-search-delivery-v1.md
- ...
MSG

tmux load-buffer -b mira_to_lyra_s6c /tmp/mira_to_lyra_s6c.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b mira_to_lyra_s6c
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
