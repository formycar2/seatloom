# SeatLoom Document Templates v1.0

| Item | Content |
|------|---------|
| Document | Document Templates Specification |
| Status | Conditionally Adopted — Template taxonomy accepted; repo migration remains staged |
| Author | Aegis |
| Date | 2026-04-28 |
| Purpose | Standardize all coordination artifacts into typed templates for structured storage, indexing, and retrieval |

---

## 0. Why This Exists

After 2 days of collaboration, our project has produced 52 markdown files. They naturally cluster into 7 types, but their internal structures are inconsistent. This creates three problems:

1. **Indexing difficulty**: Layer 1 (PostgreSQL structured index) cannot reliably extract fields from inconsistent formats
2. **Retrieval noise**: Layer 2 (FTS) returns low-quality results when documents lack standardized metadata
3. **Automation gap**: Data Engine cannot auto-generate or auto-route documents without knowing their type and schema

This document defines 7 canonical templates. Every coordination artifact must conform to one of them.

---

## 1. Template Classification

| # | Template | Purpose | Storage Path | Lifecycle |
|---|----------|---------|-------------|-----------|
| T1 | Authority Doc | Product/architecture contract — the "what" | `docs/` | Versioned (v0.1, v0.2...) |
| T2 | Role Profile | Seat identity and capability definition | `docs/coordination/roles/` | Long-lived, updated on role change |
| T3 | Task Packet | Scoped work assignment from supervisor to worker | `docs/coordination/tasks/<role>/` | issued → in_progress → delivered → accepted/rejected |
| T4 | Review | Analysis, findings, and recommendations | `docs/coordination/reviews/` | Draft → pending review → accepted/disputed |
| T5 | Acceptance | Pass/fail verdict against acceptance criteria | `docs/coordination/acceptance/` | Issued (immutable once published) |
| T6 | Daily Memory | Daily log of decisions, priorities, blockers | `docs/coordination/memory/` | One per day, append-only within day |
| T7 | Governance Doc | Cross-cutting rules, principles, protocols | `docs/coordination/` | Active, versioned on major change |

---

## 2. Universal Header (Required for All Templates)

Every document MUST begin with a YAML-compatible metadata table:

```markdown
# <Document Title>

| Field | Value |
|-------|-------|
| template | T1/T2/T3/T4/T5/T6/T7 |
| subtype | <template-specific subtype> |
| id | <unique identifier> |
| status | <template-specific status> |
| author | <seat name or "human"> |
| date | YYYY-MM-DD |
| version | <semver or "v1"> |
| depends_on | <list of document ids or paths> |
| supersedes | <document id, if replacing an older version> |
| tags | <comma-separated keywords for retrieval> |
```

The `template+subtype` pair is the canonical classification key for product behavior, while `id` remains the unique document identifier for storage. The `tags` field feeds Layer 2 (FTS) and Layer 3 (semantic) indexing.

---

## 3. T1: Authority Doc

**Purpose**: Product requirements, architecture design, UX spec, interaction spec, acceptance spec — the contracts that govern implementation.

**Status values**: `draft` → `review` → `approved` → `superseded`

**Subtype values (required)**:

| subtype | Meaning | Typical file(s) |
|--------|---------|-----------------|
| `prd` | Product requirements contract | `docs/prd-*.md` |
| `ux_spec` | Screen-level UX specification | `docs/ux-spec-*.md` |
| `interaction_spec` | Flow-level interaction contract | `docs/interaction-spec-*.md` |
| `acceptance_spec` | Acceptance/gate criteria contract | `docs/acceptance-spec-*.md` |
| `architecture_design` | Engineering architecture design | `docs/architecture-design*.md` |
| `architecture_decisions` | Architecture decision records | `docs/architecture-decisions*.md` |

**Required sections**:

```markdown
# <Document Title>

| Field | Value |
|-------|-------|
| template | T1 |
| subtype | prd |
| id | prd-v0.5 |
| status | draft |
| author | lyra |
| date | 2026-04-28 |
| version | v0.5 |
| depends_on | prd-v0.4 |
| supersedes | prd-v0.4 |
| tags | product, requirements, contract |

## 0. Document Purpose

<Why this document exists and what decisions it governs>

## 1-N. Content Sections

<Varies by document type>

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| v0.5 | 2026-04-28 | lyra | Added Data Engine, Supervisor Layer, ... |
| v0.4 | 2026-04-27 | aegis | Design-first contract baseline |
```

**Database schema**:

```sql
INSERT INTO documents (id, template, subtype, status, author, date, version, supersedes, tags)
VALUES ('prd-v0.5', 'T1', 'prd', 'draft', 'lyra', '2026-04-28', 'v0.5', 'prd-v0.4',
        ARRAY['product','requirements','contract']);
```

---

## 4. T2: Role Profile

**Purpose**: Define a Seat's identity, capabilities, constraints, and collaboration boundaries.

**Status values**: `active` → `deprecated`

**Subtype values (required)**:

| subtype | Meaning |
|--------|---------|
| `seat_role` | Standard seat role profile (Lyra/Nimbus/Mira/Flux) |

**Required sections**:

```markdown
# <Seat Name> - <Role Title>

| Field | Value |
|-------|-------|
| template | T2 |
| subtype | seat_role |
| id | role-lyra |
| status | active |
| author | aegis |
| date | 2026-04-27 |
| version | v1 |
| tags | role, product_owner, supervisor |

## Mission

<One sentence>

## Capabilities

<Structured list — feeds Seat Card>

## Primary Inputs

<What this seat receives and from whom>

## Primary Outputs

<What this seat produces and for whom>

## Must Do

<Mandatory behaviors>

## Must Not Do

<Prohibited behaviors>

## Collaboration Boundaries

| Can receive from | Can deliver to | Cannot interact with directly |
|-----------------|----------------|------------------------------|
| Aegis, Flux | Mira, Nimbus, Flux | Mr. Zhang (escalate via Aegis) |
```

---

## 5. T3: Task Packet

**Purpose**: Scoped work assignment with clear done definition and acceptance reference.

**Status values**: `issued` → `in_progress` → `delivered` → `accepted` / `rejected`

**Subtype values (required)**:

| subtype | Meaning | Typical target |
|--------|---------|----------------|
| `task` | Standard implementation/design task | Mira/Nimbus |
| `fix` | Fix a failed acceptance or regression | Mira/Nimbus |
| `integration` | Integrate outputs across surfaces (e.g., UI + backend) | Nimbus |
| `verification` | Verification/acceptance execution packet | Flux |

**Required sections**:

```markdown
# Task: <Short Title>

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | TASK-mira-2026-04-28-001 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | prd-v0.5, ux-spec-v1.1 |
| tags | ui, prototype, phase1 |

## Objective

<1-3 sentences: what to deliver>

## Input Files

- `<path>` — <why needed>

## Done Definition

- [ ] <verifiable criterion 1>
- [ ] <verifiable criterion 2>

## Acceptance Spec Reference

- `<path>` §<section>

## Constraints

- <scope limits, budget limits, prohibitions>

## Delivery Instructions

Write delivery report as T3 with status `delivered` to `tasks/<role>/`.
```

---

## 6. T4: Review

**Purpose**: Analysis, findings, and recommendations on a topic. May contain proposals that become inputs to Authority Docs.

**Status values**: `draft` → `pending_review` → `accepted` / `disputed` / `partially_accepted`

**Subtype values (required)**:

| subtype | Meaning | Example |
|--------|---------|---------|
| `gap_review` | Contract gaps / design review with findings | interaction design review |
| `benchmark` | Benchmark/competitive analysis | Multica benchmark |
| `process_mapping` | Map real workflow into model / postmortem | 2-day process mapping |
| `design_proposal` | New module proposal / architecture proposal | core value deep dive |

**Required sections**:

```markdown
# <Review Title>

| Field | Value |
|-------|-------|
| template | T4 |
| subtype | gap_review |
| id | review-interaction-design-2026-04-28 |
| status | pending_review |
| author | aegis |
| date | 2026-04-28 |
| version | v1 |
| reviewer | lyra |
| scope | interaction-spec-v1.0, ux-spec-v1.0, prd-v0.4 |
| tags | interaction, ux, review, gaps |

## 0. Overall Assessment

<1-3 sentences summary + score if applicable>

## 1-N. Findings

Each finding should follow:

### <ID>. <Finding Title>

- **Severity**: Critical / High / Medium / Low
- **Current state**: <what exists>
- **Gap**: <what's missing or wrong>
- **Recommendation**: <what to do>
- **Affected documents**: <list>

## Priority Fix List

| Priority | ID | Fix Item | Owner |
|----------|-----|----------|-------|
| Critical | C1 | ... | ... |

## Recommended Next Steps

<Ordered list>
```

---

## 7. T5: Acceptance

**Purpose**: Pass/fail verdict against specific acceptance criteria. Immutable once published.

**Status values**: `issued` (immutable)

**Subtype values (required)**:

| subtype | Meaning |
|--------|---------|
| `acceptance_review` | Acceptance report against an acceptance spec / task packet |
| `gate_decision` | Gate decision artifact (GO/HOLD/CONDITIONAL GO) |

**Required sections**:

```markdown
# Acceptance: <Target Description>

| Field | Value |
|-------|-------|
| template | T5 |
| subtype | acceptance_review |
| id | accept-mira-ui-2026-04-27 |
| status | issued |
| author | lyra |
| date | 2026-04-27 |
| version | v1 |
| target | TASK-mira-2026-04-27-001 |
| verdict | PASS / FAIL / CONDITIONAL |
| tags | acceptance, ui, mira |

## Verdict

**<PASS / FAIL / CONDITIONAL>**

Reason: <1-2 sentences>

## Coverage Matrix

| # | Criterion | Result | Evidence | Gap |
|---|-----------|--------|----------|-----|
| 1 | ... | PASS/FAIL | <file/command> | <description if FAIL> |

## Issues Found

| ID | Severity | Description |
|----|----------|-------------|
| A1 | P0 | ... |

## Gate Decision

<GO / HOLD / CONDITIONAL GO>

## Follow-up Actions

- <action 1>
- <action 2>
```

---

## 8. T6: Daily Memory

**Purpose**: Daily log of decisions, priorities, blockers, and handoffs. One file per day, append-only.

**Status values**: `open` → `closed`

**Subtype values (required)**:

| subtype | Meaning |
|--------|---------|
| `daily_log` | Standard daily memory log |

**Required sections**:

```markdown
# Daily Memory - YYYY-MM-DD

| Field | Value |
|-------|-------|
| template | T6 |
| subtype | daily_log |
| id | daily-2026-04-28 |
| status | open |
| author | lyra |
| date | 2026-04-28 |
| tags | daily, memory |

## Context

- Project: SeatLoom
- Coordination owner: <name>

## Today Key Decisions

1. <decision>

## Active Priorities

1. <priority>

## Open Blockers

- <blocker>

## Expected Handoffs

- <from> → <to>: <what>

## Progress Updates

### <role> (<time>)
- Done: ...
- In progress: ...
- Blocked: ...

## End of Day Close

- Accepted outputs: ...
- Deferred items: ...
- Tomorrow first action: ...
```

---

## 9. T7: Governance Doc

**Purpose**: Cross-cutting rules, principles, and protocols that govern collaboration behavior.

**Status values**: `draft` → `active` → `superseded`

**Subtype values (required)**:

| subtype | Meaning | Typical file |
|--------|---------|--------------|
| `coordination_rules` | File-first coordination operating rules | COORDINATION_RULES.md |
| `workflow_principles` | AI-native workflow principles | AI_NATIVE_WORKFLOW_PRINCIPLES.md |
| `collaboration_protocol` | Role routing and packet protocol | COLLABORATION_PROTOCOL.md |
| `document_templates` | Template taxonomy and schema | DOCUMENT_TEMPLATES.md |

**Required sections**:

```markdown
# <Governance Title>

| Field | Value |
|-------|-------|
| template | T7 |
| subtype | coordination_rules |
| id | gov-coordination-rules |
| status | active |
| author | aegis |
| date | 2026-04-27 |
| version | v1.0 |
| tags | governance, rules, coordination |

## 1. Why This Exists

<Problem this governance doc solves>

## 2-N. Rules / Principles / Protocols

<Content>

## Adoption Checklist

- [ ] <adoption step>

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
```

---

## 10. Database Schema for Template Storage (PostgreSQL)

All templates share a common envelope. PostgreSQL is the single storage engine for all three retrieval layers: structured index (relational), full-text search (tsvector), and semantic index (pgvector).

### 10.1 Core Document Table

```sql
CREATE TYPE template_type AS ENUM ('T1','T2','T3','T4','T5','T6','T7');
CREATE TYPE verdict_type AS ENUM ('PASS','FAIL','CONDITIONAL');

CREATE TABLE documents (
  id TEXT PRIMARY KEY,
  template template_type NOT NULL,
  subtype TEXT NOT NULL,
  status TEXT NOT NULL,
  author TEXT NOT NULL,
  date DATE NOT NULL,
  version TEXT,
  title TEXT,
  supersedes TEXT REFERENCES documents(id),
  depends_on TEXT[],
  tags TEXT[],
  reviewer TEXT,
  to_seat TEXT,
  priority TEXT,
  verdict verdict_type,
  file_path TEXT NOT NULL,
  metadata JSONB DEFAULT '{}',
  created_at TIMESTAMPTZ DEFAULT now(),
  updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_docs_template ON documents(template);
CREATE INDEX idx_docs_subtype ON documents(subtype);
CREATE INDEX idx_docs_status ON documents(status);
CREATE INDEX idx_docs_author ON documents(author);
CREATE INDEX idx_docs_date ON documents(date);
CREATE INDEX idx_docs_tags ON documents USING GIN(tags);
CREATE INDEX idx_docs_depends ON documents USING GIN(depends_on);
CREATE INDEX idx_docs_metadata ON documents USING GIN(metadata);
```

### 10.2 Layer 2: Full-Text Search (Built-in tsvector)

```sql
ALTER TABLE documents ADD COLUMN search_vector tsvector;

CREATE OR REPLACE FUNCTION documents_search_update() RETURNS trigger AS $$
BEGIN
  NEW.search_vector :=
    setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
    setweight(to_tsvector('english', COALESCE(array_to_string(NEW.tags, ' '), '')), 'B');
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_docs_search
  BEFORE INSERT OR UPDATE ON documents
  FOR EACH ROW EXECUTE FUNCTION documents_search_update();

CREATE INDEX idx_docs_fts ON documents USING GIN(search_vector);

-- Artifact content: separate table for full-text on file bodies
CREATE TABLE document_content (
  doc_id TEXT REFERENCES documents(id) ON DELETE CASCADE,
  chunk_index INT,
  chunk_text TEXT NOT NULL,
  section_heading TEXT,
  search_vector tsvector,
  PRIMARY KEY (doc_id, chunk_index)
);

CREATE TRIGGER trg_content_search
  BEFORE INSERT OR UPDATE ON document_content
  FOR EACH ROW EXECUTE FUNCTION
    (SELECT setweight(to_tsvector('english', NEW.chunk_text), 'C'));

CREATE INDEX idx_content_fts ON document_content USING GIN(search_vector);
```

### 10.3 Layer 3: Semantic Index (pgvector)

```sql
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE document_embeddings (
  doc_id TEXT REFERENCES documents(id) ON DELETE CASCADE,
  chunk_index INT,
  embedding vector(768),
  PRIMARY KEY (doc_id, chunk_index)
);

CREATE INDEX idx_embeddings_ivfflat
  ON document_embeddings USING ivfflat (embedding vector_cosine_ops)
  WITH (lists = 100);
```

### 10.4 Unified Three-Layer Query Examples

```sql
-- Layer 1: Structured query — all active task packets for mira
SELECT * FROM documents
WHERE template = 'T1' AND to_seat = 'mira' AND status = 'issued';

-- Layer 1: JSONB metadata query
SELECT * FROM documents
WHERE metadata @> '{"workitem": "WI-012"}';

-- Layer 1: Array contains — all documents depending on prd-v0.4
SELECT * FROM documents
WHERE 'prd-v0.4' = ANY(depends_on);

-- Layer 1: Latest version of each authority doc
SELECT DISTINCT ON (SPLIT_PART(id, '-v', 1))
  * FROM documents
WHERE template = 'T1'
ORDER BY SPLIT_PART(id, '-v', 1), version DESC;

-- Layer 2: Full-text search across document metadata
SELECT id, title, ts_rank(search_vector, q) AS rank
FROM documents, plainto_tsquery('english', 'callback TypeError') q
WHERE search_vector @@ q
ORDER BY rank DESC;

-- Layer 2: Full-text search across artifact content
SELECT dc.doc_id, dc.section_heading, dc.chunk_text,
       ts_rank(dc.search_vector, q) AS rank
FROM document_content dc, plainto_tsquery('english', 'passport.js OAuth') q
WHERE dc.search_vector @@ q
ORDER BY rank DESC
LIMIT 10;

-- Layer 3: Semantic similarity search (find chunks nearest to a query embedding)
SELECT dc.doc_id, dc.chunk_index, dc.chunk_text,
       de.embedding <=> $1::vector AS distance
FROM document_embeddings de
JOIN document_content dc ON de.doc_id = dc.doc_id AND de.chunk_index = dc.chunk_index
ORDER BY de.embedding <=> $1::vector
LIMIT 5;

-- Combined: structured filter + semantic search
SELECT dc.doc_id, dc.chunk_text, de.embedding <=> $1::vector AS distance
FROM document_embeddings de
JOIN document_content dc ON de.doc_id = dc.doc_id AND de.chunk_index = dc.chunk_index
JOIN documents d ON d.id = de.doc_id
WHERE d.template = 'T4' AND d.status = 'accepted'
ORDER BY de.embedding <=> $1::vector
LIMIT 5;
```

### 10.5 Why PostgreSQL Over SQLite

| Dimension | PostgreSQL | SQLite |
|-----------|-----------|--------|
| Structured queries | Full SQL + JSONB + array ops | Basic SQL + JSON1 extension |
| Full-text search | Built-in tsvector/tsquery with ranking and weighting | FTS5 extension, limited ranking |
| Semantic search | pgvector extension, production-grade ANN index | Requires separate library (usearch) |
| All three layers | **Single engine** | Three separate engines stitched together |
| Concurrent access | Multi-process safe | Single-writer lock |
| Scalability | Handles millions of rows | Slows at ~100k rows with complex queries |
| Deployment | Requires PG server (local or Docker) | Zero-dependency single file |

Trade-off: PostgreSQL requires a running server, which adds deployment complexity for a "local-first" desktop app. Mitigation: bundle a lightweight embedded PG (e.g., `embedded-postgres` crate for Rust) or run PG in a Docker sidecar managed by the Tauri app.

---

## 11. Template Validation Rules

The Data Engine should enforce at file-write time:

| Rule | Enforcement |
|------|------------|
| Every `.md` in `docs/coordination/` must have the universal header | Reconcile Engine warns on missing header |
| `template` field must be one of T1-T7 | Reject otherwise |
| `subtype` must be present and valid for the chosen template | Reject otherwise |
| `id` must be unique across all documents | Reconcile Engine detects duplicates |
| T3 must have `to` field | Reject if missing |
| T5 `verdict` must be PASS/FAIL/CONDITIONAL | Reject otherwise |
| T1 with `supersedes` must update the old doc's status to `superseded` | Reconcile Engine auto-updates |
| `tags` must be non-empty | Warn (not reject) |

### 11.1 Subtype allow-lists (v1.0)

These allow-lists should be implemented in the verifier to prevent taxonomy drift:

| template | allowed subtypes |
|----------|------------------|
| T1 | `prd`, `ux_spec`, `interaction_spec`, `acceptance_spec`, `architecture_design`, `architecture_decisions` |
| T2 | `seat_role` |
| T3 | `task`, `fix`, `integration`, `verification` |
| T4 | `gap_review`, `benchmark`, `process_mapping`, `design_proposal` |
| T5 | `acceptance_review`, `gate_decision` |
| T6 | `daily_log` |
| T7 | `coordination_rules`, `workflow_principles`, `collaboration_protocol`, `document_templates` |

---

## 12. Migration Plan for Existing Documents

52 existing files need header addition. This can be done in one batch:

| Template | Files to migrate | Effort |
|----------|-----------------|--------|
| T1 | 8 (prd, ux-spec, interaction-spec, etc.) | Add header + changelog |
| T2 | 4 (role files) | Add header + capabilities section |
| T3 | 14 (task packets) | Add header (most already have partial metadata) |
| T4 | 10 (reviews) | Add header + standardize finding format |
| T5 | 3 (acceptance) | Add header + standardize verdict format |
| T6 | 2 (daily memory) | Add header |
| T7 | 4 (governance) | Add header + changelog |

Estimated effort: ~2 hours of mechanical work (suitable for Flux or automated script).

---

## 13. Relationship to SeatLoom Product

These templates are not just for our internal coordination — they directly inform the product's Artifact type system through a `template+subtype` dual key.

| Template | SeatLoom family label | Representative subtypes | Auto-generated by |
|----------|-----------------------|-------------------------|-------------------|
| T1 | `authority_doc` | `prd`, `ux_spec`, `interaction_spec`, `acceptance_spec`, `architecture_design`, `architecture_decisions` | Human / Supervisor |
| T2 | `role_profile` | `seat_role` | Human / Supervisor |
| T3 | `task_packet` | `task`, `fix`, `integration`, `verification` | Supervisor / Route Engine |
| T4 | `review` | `gap_review`, `benchmark`, `process_mapping`, `design_proposal` | Human / Supervisor |
| T5 | `acceptance_verdict` | `acceptance_review`, `gate_decision` | Gate Engine + Human |
| T6 | `daily_memory` | `daily_log` | Reconcile Engine (auto) + Lyra (manual) |
| T7 | `governance` | `coordination_rules`, `workflow_principles`, `collaboration_protocol`, `document_templates` | Human |

When SeatLoom is built, its Artifact system should natively understand this dual key, enabling:
- family-level Detail Pane rendering plus subtype-specific summary blocks and actions,
- family/subtype search filters (e.g., "show all `gate_decision` artifacts"),
- subtype-aware automation (e.g., `gate_decision` drives gate surfaces while `verification` packets drive execution follow-up).

---

*This document defines the template standard for all SeatLoom coordination artifacts. Adopted into the active v0.5 contract set on 2026-04-28; repo-wide header migration remains staged.*
