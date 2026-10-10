# Delivery: Lane A — migration 015 knowledge-layer schema

[Onyx -> Lyra] D5 knowledge-layer, Lane A: migration 015 lands the knowledge-layer
schema — project tree (`projects.parent_project_id`), the `memories` base table
(L0/L1/L2 + cold→hot lifecycle), the executable slug-normalization view over
`archived_sessions`, and the vocabulary widening (+4 memory event types / +1
`memory` ref type). Verified on scratch DBs only; the live `seatloom` database was
never written.

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | implementation-delivery |
| id | ONYX-2026-10-10-d5-knowledge-layer-schema-delivery-v1 |
| status | delivered |
| author | onyx |
| date | 2026-10-10 |
| version | v1 |
| responds to | governed dispatch `disp-d5-lane-a-onyx` (D5 knowledge-layer Lane A) |
| packet | `docs/coordination/tasks/onyx/ONYX-2026-10-10-d5-knowledge-layer-schema-v1.md` (found on `packet/lyra/milestone-summary`; not on my base branch — worked from the payload + that packet verbatim) |
| governing contract | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-knowledge-layer-v1.md` §1-§6 (Lyra sign-off), carried verbatim |
| base branch | `track/infra-foundation` @ `b6751a1` |
| delivery branch | `packet/onyx/d5-knowledge-schema` |
| files changed | `infra/postgres/schema/015_knowledge_layer.sql` (new), `scripts/apply-db.sh` (+1 POST_SEED entry), this doc |
| rust / tsx touched | none (Lane B is Nimbus's `crates/**`; verified: `git status --short` shows only the two files above) |
| live db touched | none — live was READ only (slug census + `pg_dump --data-only` of `archived_sessions` into scratch); all writes were on scratch DBs, dropped after |

---

## 0. What shipped

### 0.1 `infra/postgres/schema/015_knowledge_layer.sql` (new)

1. **Project tree (contract §1)**: `projects.parent_project_id TEXT NULL REFERENCES projects(id)` (`ADD COLUMN IF NOT EXISTS`, additive-safe — existing rows backfill to NULL = root), `projects_parent_not_self CHECK (parent_project_id IS DISTINCT FROM id)` (guarded DO block), partial index `idx_projects_parent ... WHERE parent_project_id IS NOT NULL`.
2. **`memories` table (contract §4 verbatim)**: 22 columns in contract order with exact types/nullability/defaults; the l2 pairing CHECK sits where the contract puts it; named guarded CHECKs `memories_tier_vocab` (cold|warm|hot) and `memories_status_vocab` (active|forgotten); indexes `(project_id, topic_id, status)`, `(project_id, topic_id, tier)`, and the 002-style FTS gin `idx_memories_fts` on `title || l0_summary`; COMMENTs on the table and every column at 010/012 density.
3. **Vocabulary widening (contract §5/§6)**: guarded DO blocks write the **COMPLETE** lists — `canonical_events_event_type_vocab` 25 values (live 21 + `MemoryCreated/MemoryRecalled/MemoryPromoted/MemoryForgotten`), `event_object_refs_ref_type_vocab` 12 values (live 11 + `memory`). The full list is a superset of both the track state (15/10) and the live state (21/11), so 015 applies cleanly from either — verified both ways (§2.2).
4. **Normalization view (contract §2 R1-R5)**: `v_archived_sessions_normalized` — catalog-guarded creation (created only when absent; a future definition change is a follow-up migration's DROP+CREATE, never a silent rewrite). Outputs `harness, id, raw_slug, normalized_parent, normalized_leaf, map_status`. R1 exact (`slug == projects.id`), R2 path (`'a/b'` → leaf `b` under parent `a`; `'a/b/c'` nests likewise; mapped only when that (leaf, parent) pair really exists), R4 unknown → `unmapped` (reported, never guessed), R5 `'(root)'` → `unmapped`. **ZERO row inserts** (R3).
5. **`scripts/apply-db.sh`**: `015_knowledge_layer.sql` appended to `POST_SEED_SCHEMA` after 014 with a rule-based reason (new table + ADD COLUMN + view + vocab widening; seed writes none of these; `memories` REFERENCES `topics(id)` (010, POST-SEED) and `documents(id)` (002), so 015 cannot precede its prerequisites; canonical order 010 → 015). `bash -n` clean; completeness guard passes.

### 0.2 Contract compliance (packet §3.7 red lines)

- **Zero row inserts by the migration** — no project/topic/memory rows; the tree stays materialization-free until Aegis's Phase-2 washing pipeline.
- **`topics` untouched** (columns + vocab), **`documents` model untouched**, **no historical renumbering** (006/007 stay reserved).
- **No second source of truth, no external memory-system code** (OpenViking/Mem0/Letta patterns may be studied, code never imported).
- **`crates/**` untouched** (Lane B is Nimbus's); no branch merges; no live writes.

## 1. Design notes / fidelity notes

- **Named CHECKs.** The contract's l2 pairing CHECK is written inline and unnamed; I named it `memories_l2_pairing` (and the tier/status CHECKs `memories_tier_vocab` / `memories_status_vocab`) in 012/013 house style so the §5.4 verification query can reference them by name. Semantics unchanged — a fidelity note, not a deviation.
- **`source_kind` and `forgotten_reason` are enforced by the writing tools, not the DB.** The contract lists `source_kind`'s four values in prose and requires a reason on forget; I did not add CHECKs the contract does not specify (a DB CHECK would freeze the value set before the MCP tools exist, and a cross-column status/reason CHECK is not in the contract). Both are recorded in the column COMMENTs as tool-enforced.
- **"10 known slugs" = the ten listed in the milestone doc** (`LYRA-2026-10-10-d5-knowledge-layer-milestone.md` §1): `sf-code/storage, sf-code/brainpp, sf-code/stepatlas, ai-assistant/android-agent, ai-assistant/sf-package, fde/field_evaluation, OfficeBench, SteptronOss, cn-gpu-infer-fabric, seatloom`. Since live `projects` holds only `seatloom` (tree materialization is Aegis Phase 2), the scratch verification used the tree these ten slugs imply — 7 roots (`seatloom, OfficeBench, SteptronOss, cn-gpu-infer-fabric, sf-code, ai-assistant, fde`) + 6 leaves (`storage/brainpp/stepatlas` under `sf-code`, `android-agent/sf-package` under `ai-assistant`, `field_evaluation` under `fde`) — inserted as **scratch fixtures only**, zero rows shipped by the migration.
- **Unmapped count over the full 271 rows is 52, not 13** — and that is R4 working as contracted. The 13 `(root)` rows are unmapped per R5; the other 39 rows (23 further unknown slugs: `private_knowledge_base`, `project_delivery/QL`, `sf_model_delivery`, `sf-code/openapi`, `(multi)`, …) are unmapped per R4 (unknown slug → reported, never guessed). Scoped to the known-slug subset, unmapped is exactly the 13 `(root)` rows, matching the packet's expectation. Both numbers are reported here; the full picture goes to Aegis with the `(root)` escalation.

## 2. Verification — scratch only (live untouched)

Environment: `postgres:16-alpine` runner container sharing the `seatloom-postgres` network namespace; fresh scratch DB `seatloom_scratch_d5` (full build) + the pre-built `seatloom_scratch_016` (014 state, for the live-state test); both dropped after. Live `seatloom` was read-only (slug census + data copy).

### 2.1 Full build from clean track state

`scripts/apply-db.sh` on the empty DB: whole `001 → 015` chain + seed, completeness guard passes, exit 0, no SQL error; 015 applied in canonical position after 014.

### 2.2 Applies from the live state (21/11) too

015 applied directly onto the 014-state DB (vocab 21/11 — same state as live): exit 0; `canonical_events_event_type_vocab` gained `MemoryCreated`, `event_object_refs_ref_type_vocab` gained `memory`. Also cross-checked the other serialized lane: 016's file applies cleanly **on top of** 015 with the vocab staying 25/12 — the two same-seat migrations do not interact.

### 2.3 Vocabulary final state (§5.2)

```
event_type count: 25
ref_type count: 12
```

### 2.4 Project tree (§3.2)

Column: `parent_project_id text`, nullable (root = NULL), self-FK to `projects(id)`.
`projects_parent_not_self` → `CHECK ((parent_project_id IS DISTINCT FROM id))`.
Partial index `idx_projects_parent ON projects (parent_project_id) WHERE parent_project_id IS NOT NULL`.
Fixture tree verified: 7 roots / 6 leaves, leaves' `parent_project_id` accepted by the self-FK.

### 2.5 `memories` shape vs contract §4 (verbatim)

22 columns, contract order: `id text PK, project_id text NOT NULL→projects, topic_id text NOT NULL→topics, title text NOT NULL, tags text[] NOT NULL DEFAULT '{}', l0_summary text NOT NULL, l1_overview text, l2_ref_kind text NOT NULL, l2_document_id text NULL→documents, l2_file_path text, l2_body_digest text, [memories_l2_pairing CHECK], source_kind text NOT NULL, source_ref text NOT NULL, heat integer NOT NULL DEFAULT 0, recall_count integer NOT NULL DEFAULT 0, last_recalled_at timestamptz NULL, tier text NOT NULL DEFAULT 'cold', tier_changed_at timestamptz NOT NULL DEFAULT now(), status text NOT NULL DEFAULT 'active', forgotten_reason text, created_at/updated_at timestamptz NOT NULL DEFAULT now()`.

### 2.6 Domain CHECKs — negative + positive (§5.4)

| case | result |
|---|---|
| `l2_ref_kind='document'` without `l2_document_id` | **rejected** — `memories_l2_pairing` |
| `l2_ref_kind='file'` without `l2_file_path` | **rejected** — `memories_l2_pairing` |
| `tier='warmish'` | **rejected** — `memories_tier_vocab` |
| `status='archived'` | **rejected** — `memories_status_vocab` |
| valid row (`l2_ref_kind='file'` + path + digest) | **accepted**; defaults verified: `heat=0, recall_count=0, tier=cold, status=active, last_recalled_at NULL, tags={}` |

FTS index verified: `to_tsvector(title || l0_summary) @@ to_tsquery('acceptance')` returns the row.

### 2.7 Normalization over all 271 rows (§5.3)

```
 map_status | count
------------+-------
 mapped     |  219
 unmapped   |   52
```

- **All 10 known slugs mapped** — incl. parent/leaf decomposition for the path forms:
  `sf-code/storage → (sf-code, storage)`, `sf-code/brainpp → (sf-code, brainpp)`, `sf-code/stepatlas → (sf-code, stepatlas)`, `ai-assistant/android-agent → (ai-assistant, android-agent)`, `ai-assistant/sf-package → (ai-assistant, sf-package)`, `fde/field_evaluation → (fde, field_evaluation)`; exact roots `seatloom (16), OfficeBench (66), SteptronOss (13), cn-gpu-infer-fabric (4), sf-code (5)`.
- **`(root)` → unmapped, 13 rows** (R5) — escalated to Aegis (§3).
- Remaining unmapped (R4, never guessed): `private_knowledge_base 4, project_delivery/QL 4, sf_model_delivery 3, Assistant_Aegis 2, ThoughtOnly 2, ai-assistant/ai-insight 2, ai-insight/tinker-docs 2, sf-code/jobutils 2, sf-code/stepcloud 2, sf-code/stepmind 2, (multi) 1, ColossalAI 1, ai-assistant/skills 1, ai-insight 1, ai-insight/post-training 1, benchmark 1, gnoloaixgnahz.github.io 1, jobutils 1, kindle-youth-audit 1, project_delivery 1, project_delivery/WX 1, sf-code/openapi 1, tools 1, yx_work 1` — reported for the Phase-2 washing decision, not force-fit.
- Note: `sf-code/openapi` is unmapped although its prefix `sf-code` exists — the leaf `openapi` is not a project, and R4 forbids guessing. Same for `project_delivery/QL` (neither side in the tree).

### 2.8 Idempotency (§5.1)

Second apply of `015_knowledge_layer.sql`, catalog counts before/after:

```
BEFORE: pg_class=620 pg_constraint=236 pg_attribute=4139
AFTER:  pg_class=620 pg_constraint=236 pg_attribute=4139   (zero delta; only NOTICE ... already exists, skipping)
```

`memories` column signature (name:type:nullable:default for all 22 columns) byte-identical across applies; the view stays deterministic (`mapped=219 unmapped=52`).

## 3. "(root)" escalation (R5 — decision requested, not taken)

13 `archived_sessions` rows carry the free-text slug `(root)`: no project归属. Per contract R5 no ruling is assumed; the strategy decision belongs to Aegis. Emitted a `SupervisorMessage` event (actor `seat-onyx-001`, subject ref `project:seatloom`, context ref `dispatch:disp-d5-lane-a-onyx`) with payload `{unmapped_count: 13, raw_slug: "(root)", decision_needed_from: "Aegis"}`.

## 4. Finding to flag (pre-existing, NOT fixed — out of this dispatch's lane)

Same as the 016 delivery: `scripts/apply-db.sh` cannot run twice against an already-built database — seed 001's `sessions` upsert supplies no `project_id` and 008's NOT NULL fires before ON CONFLICT resolution (exit 3 at seed 001). Independent of 015; idempotency is measured per-migration (§2.8), exactly as the D2 (013/014) delivery did. **Flagged for Aegis/Lyra**; the seed was not touched under this dispatch.

## 5. Completion flow

1. Committed to `packet/onyx/d5-knowledge-schema` (commit carries the dispatch id).
2. This delivery doc.
3. Gate event `ev-d5-015-knowledge-schema-landed` (ArtifactCreated) emitted — unblocks Nimbus Lane B via the 013 trigger.
4. `SupervisorMessage` "(root)" escalation emitted (Aegis).
5. `complete_dispatch(disp-d5-lane-a-onyx, outcome="done")`.
6. Flux verification self-requested (commit-pinned branch/SHA, base `track/infra-foundation`, COORDINATION_RULES §14).
7. After Flux green: `request_acceptance` with the packet §7 how_to_verify verbatim. The seat never merges itself.
