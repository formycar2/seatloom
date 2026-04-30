-- SeatLoom Document Authority Seed v1
-- Seeds typed coordination documents from the real repository.
-- Universal header fields sourced from actual document headers.
-- Body text:
--   - Short docs are seeded inline below.
--   - Full bodies for large docs are populated by scripts/ingest-documents.sh
--     which reads actual repo files at ingest time.
-- Evidence: docs/PRODUCT_TRUTH.md active contract set, MEMORY.md milestone log.

BEGIN;

-- =============================================================================
-- T1 Authority Docs — active contract set
-- Evidence: docs/PRODUCT_TRUTH.md §1.2
-- =============================================================================

INSERT INTO documents (id, project_id, artifact_id, template, subtype, subtype_valid,
    doc_id, title, status, author, doc_date, version,
    depends_on, supersedes, tags, file_path, body_text, body_length, parse_status)
VALUES
  ('doc-product-truth', 'seatloom', 'ar-product-truth',
   'T1AuthorityDoc', 'prd', true,
   'product-truth-index', 'SeatLoom Product Truth Index', 'active', 'lyra',
   '2026-04-28', 'v1',
   '{}', NULL, ARRAY['product','truth','index','canonical'],
   'docs/PRODUCT_TRUTH.md',
   '# SeatLoom Product Truth Index

Single entrypoint for active product truth during v0.5 consolidation.
Active contract set: docs/prd-v0.5.md, docs/interaction-spec-v1.1.md,
docs/ux-spec-v1.1.md, docs/acceptance-spec-v1.1.md, docs/architecture-decisions.md,
docs/architecture-design.md.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial'),

  ('doc-prd-v05', 'seatloom', 'ar-prd-v05',
   'T1AuthorityDoc', 'prd', true,
   'prd-v0.5', 'SeatLoom PRD v0.5', 'draft', 'lyra',
   '2026-04-28', 'v0.5',
   ARRAY['prd-v0.4'], 'prd-v0.4', ARRAY['product','requirements','contract','v0.5'],
   'docs/prd-v0.5.md',
   '# SeatLoom PRD v0.5

SeatLoom is a local-first continuity system for human-plus-agent project work.
Five core product modules: Data Engine, Seat three-layer architecture,
Playbook system, Supervisor Layer, Artifact review system.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial'),

  ('doc-interaction-v11', 'seatloom', 'ar-interaction-v11',
   'T1AuthorityDoc', 'interaction_spec', true,
   'interaction-spec-v1.1', 'SeatLoom Interaction Spec v1.1', 'draft', 'lyra',
   '2026-04-28', 'v1.1',
   ARRAY['prd-v0.5'], NULL, ARRAY['interaction','flows','contract','v1.1'],
   'docs/interaction-spec-v1.1.md',
   '# SeatLoom Interaction Spec v1.1

Flow definitions for all INT-01 through INT-P3-01 interactions.
Global rules: confirmation-first, deterministic-first, focus, audit, isolation,
budget, artifact-typing, review-change-tier, mobile-channel-consistency.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial'),

  ('doc-ux-v11', 'seatloom', 'ar-ux-v11',
   'T1AuthorityDoc', 'ux_spec', true,
   'ux-spec-v1.1', 'SeatLoom UX Spec v1.1', 'draft', 'lyra',
   '2026-04-28', 'v1.1',
   ARRAY['prd-v0.5'], NULL, ARRAY['ux','screen','layout','contract','v1.1'],
   'docs/ux-spec-v1.1.md',
   '# SeatLoom UX Spec v1.1

Screen structure, visible fields, surface layout, and interaction affordances
for all UX-01 through UX-15 surfaces and mobile companion.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial'),

  ('doc-acceptance-v11', 'seatloom', 'ar-acceptance-v11',
   'T1AuthorityDoc', 'acceptance_spec', true,
   'acceptance-spec-v1.1', 'SeatLoom Acceptance Spec v1.1', 'draft', 'lyra',
   '2026-04-28', 'v1.1',
   ARRAY['prd-v0.5','interaction-spec-v1.1','ux-spec-v1.1'], 'acceptance-spec-v1.0',
   ARRAY['acceptance','gate','verification','v1.1'],
   'docs/acceptance-spec-v1.1.md',
   '# SeatLoom Acceptance Spec v1.1

Pass/fail contract for the active v0.5 product-design set.
Gate model: Gate A (Contract Integrity), Gate B (P0 Delivery Readiness), Gate C (P1/P2).

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial'),

  ('doc-arch-decisions', 'seatloom', 'ar-arch-decisions',
   'T1AuthorityDoc', 'architecture_decisions', true,
   'architecture-decisions', 'SeatLoom Architecture Decisions', 'approved', 'aegis',
   '2026-04-28', 'v1.0',
   '{}', NULL, ARRAY['architecture','adr','decisions'],
   'docs/architecture-decisions.md',
   '# SeatLoom Architecture Decisions

AD-001 through AD-012. Key decisions: Tauri 2 + React/TS + Rust (AD-002),
Pack Engine (AD-004), Dual-key artifact typing (AD-008), Seat three-layer (AD-009),
Event-first review (AD-010), Retrieval order fixed (AD-011), Prompt architecture (AD-012).
Storage authority realigned to PostgreSQL (see AD-011 updated).

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial'),

  ('doc-arch-design', 'seatloom', 'ar-arch-design',
   'T1AuthorityDoc', 'architecture_design', true,
   'architecture-design', 'SeatLoom Architecture Design v1.0', 'draft', 'aegis',
   '2026-04-28', 'v1.0',
   ARRAY['architecture-decisions'], NULL, ARRAY['architecture','design','engineering'],
   'docs/architecture-design.md',
   '# SeatLoom Architecture Design v1.0

Full system architecture. Cargo workspace: seatloom-core + seatloom-tauri + seatloom-cli.
Storage authority: PostgreSQL (structured truth), disk (evidence payloads/export).
DB module: tokio-postgres + deadpool-postgres. File stores: cache/compat layer only.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- T2 Role Profile
-- =============================================================================

INSERT INTO documents (id, project_id, artifact_id, template, subtype, subtype_valid,
    doc_id, title, status, author, doc_date, version, depends_on, tags, file_path,
    body_text, body_length, parse_status)
VALUES
  ('doc-role-mira', 'seatloom', 'ar-role-mira',
   'T2RoleProfile', 'seat_role', true,
   'role-mira', 'Mira – UX/UED Designer Role Profile', 'active', 'aegis',
   '2026-04-27', 'v1',
   '{}', ARRAY['role','designer','ux','mira'],
   'docs/coordination/roles/MIRA.md',
   '# Mira - UX/UED Designer

Mission: Deliver high-quality, contract-aligned React/TypeScript UI components
for SeatLoom that faithfully implement the v0.5 UX Spec.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- T3 Task Packets (key accepted Nimbus infra deliveries)
-- =============================================================================

INSERT INTO documents (id, project_id, artifact_id, template, subtype, subtype_valid,
    doc_id, title, status, author, doc_date, version, depends_on, tags, file_path,
    body_text, body_length, parse_status)
VALUES
  ('doc-task-hardening', 'seatloom', 'ar-task-hardening',
   'T3TaskPacket', 'task', true,
   'NIMBUS-2026-04-29-foundation-hardening-v1',
   'NIMBUS-2026-04-29 Rust Foundation Hardening', 'delivered', 'lyra',
   '2026-04-29', 'v1',
   ARRAY['NIMBUS-2026-04-29-foundation-quality-automation-v1'],
   ARRAY['nimbus','rust','infrastructure','toolchain','ci','lint','formatting'],
   'docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-v1.md',
   '# Task: Nimbus Rust Foundation Hardening

Freeze Rust 1.95.0, close clippy/fmt failures, eliminate warnings, add CI.
Done: toolchain frozen, fmt+clippy passing, cargo check 0 warnings, 22 tests passing.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial'),

  ('doc-task-db-baseline', 'seatloom', 'ar-task-db-baseline',
   'T3TaskPacket', 'task', true,
   'NIMBUS-2026-04-29-real-collaboration-db-baseline-v1',
   'NIMBUS-2026-04-29 Real Collaboration PostgreSQL Baseline', 'delivered', 'lyra',
   '2026-04-29', 'v1',
   ARRAY['NIMBUS-2026-04-29-foundation-hardening-v1'],
   ARRAY['nimbus','postgres','infrastructure','seed','baseline'],
   'docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md',
  '# Task: Nimbus Real Collaboration PostgreSQL Baseline

PostgreSQL schema for core SeatLoom object families seeded with real collaboration data.
5 seats, 6 WIs, 3 handoffs, 15 artifacts T1-T7, 34 canonical events.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- T4 Review
-- =============================================================================

INSERT INTO documents (id, project_id, artifact_id, template, subtype, subtype_valid,
    doc_id, title, status, author, doc_date, version, depends_on, tags, file_path,
    body_text, body_length, parse_status)
VALUES
  ('doc-review-process-mapping', 'seatloom', 'ar-review-process-mapping',
   'T4Review', 'process_mapping', true,
   'process-mapping-2026-04-28',
   'SeatLoom Process Mapping: Real Workflow → Product Model', 'active', 'aegis',
   '2026-04-28', 'v1',
   '{}', ARRAY['review','process','mapping','seats','workflow'],
   'docs/coordination/reviews/2026-04-28-process-mapping-review.md',
   '# SeatLoom Process Mapping: Real Workflow → Product Model

Maps the actual 2-day collaboration (Apr 27-28) onto the SeatLoom object model.
5 seats, 11 WorkItems, 6 Sessions, 7 Handoffs, 27 Artifacts in real workflow.
3 product gaps found: seat delegation, supervisor rehydration, WorkItem rejection flow.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- T5 Acceptance
-- =============================================================================

INSERT INTO documents (id, project_id, artifact_id, template, subtype, subtype_valid,
    doc_id, title, status, author, doc_date, version, depends_on, tags, file_path,
    body_text, body_length, parse_status)
VALUES
  ('doc-acceptance-hardening', 'seatloom', 'ar-acceptance-hardening',
   'T5Acceptance', 'acceptance_review', true,
   'lyra-nimbus-foundation-hardening-acceptance',
   'Lyra Acceptance: Nimbus Rust Foundation Hardening – PASS', 'active', 'lyra',
   '2026-04-29', 'v1',
   ARRAY['NIMBUS-2026-04-29-foundation-hardening-delivery-v1'],
   ARRAY['acceptance','nimbus','hardening','pass'],
   'docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md',
   '# Acceptance: Nimbus Rust Foundation Hardening

Verdict: PASS. Exact Rust 1.95.0, portable verification script, CI parity,
zero-warning cargo check, passing fmt/clippy, no scope widening confirmed.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- T6 Daily Memory
-- =============================================================================

INSERT INTO documents (id, project_id, artifact_id, template, subtype, subtype_valid,
    doc_id, title, status, author, doc_date, version, depends_on, tags, file_path,
    body_text, body_length, parse_status)
VALUES
  ('doc-memory-2026-04-29', 'seatloom', 'ar-memory-2026-04-29',
   'T6DailyMemory', 'daily_log', true,
   'memory-2026-04-29',
   'Coordination Daily Memory 2026-04-29', 'active', 'lyra',
   '2026-04-29', 'v1',
   '{}', ARRAY['daily','memory','log','coordination'],
   'docs/coordination/memory/2026-04-29.md',
   '# Coordination Daily Memory - 2026-04-29

51 key decisions logged. Key milestones: SG-01 GO, Product Baseline Freeze GO,
ENV-001 closed, Nimbus foundation hardening PASS, PostgreSQL baseline delivered.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- T7 Governance
-- =============================================================================

INSERT INTO documents (id, project_id, artifact_id, template, subtype, subtype_valid,
    doc_id, title, status, author, doc_date, version, depends_on, tags, file_path,
    body_text, body_length, parse_status)
VALUES
  ('doc-coordination-rules', 'seatloom', 'ar-governance-coordination-rules',
   'T7GovernanceDoc', 'coordination_rules', true,
   'coordination-rules-v1.0',
   'SeatLoom Coordination Rules v1.0', 'active', 'lyra',
   '2026-04-28', 'v1.0',
   '{}', ARRAY['governance','coordination','rules','protocol'],
   'docs/coordination/COORDINATION_RULES.md',
   '# SeatLoom Coordination Rules v1.0

File-first coordination, mandatory writeback, artifact structure requirements,
role persistence contracts, daily cadence, stage-gate protocol, terminal summary
contract, direct seat dispatch rule. Language policy: English artifacts, Chinese terminal.

[Full body available via scripts/ingest-documents.sh]',
   NULL, 'partial')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Document sections (key headings for navigation/search)
-- Representative section set; full extraction via ingest-documents.sh
-- =============================================================================

INSERT INTO document_sections (id, document_id, ordinal, heading_text, heading_level, anchor_slug, body_excerpt, search_text)
VALUES
  ('sec-prd-v05-1', 'doc-prd-v05', 1, 'Product thesis', 2, 'product-thesis',
   'SeatLoom is a local-first continuity system for human-plus-agent project work.',
   'product thesis seatloom local-first continuity'),
  ('sec-prd-v05-2', 'doc-prd-v05', 2, 'Product architecture', 2, 'product-architecture',
   'SeatLoom operates as five connected product modules: Data Engine, Seat three-layer architecture, Playbook system, Supervisor Layer, Artifact review system.',
   'product architecture modules data engine seat playbook supervisor artifact'),
  ('sec-prd-v05-3', 'doc-prd-v05', 3, 'Priority boundaries', 2, 'priority-boundaries',
   'P0/P1/P2 boundary definitions and user promise statements.',
   'priority boundaries p0 p1 p2'),

  ('sec-arch-decisions-1', 'doc-arch-decisions', 1, 'AD-011: Data Engine Retrieval Order — Fixed Layer Sequence', 2, 'ad-011-data-engine-retrieval-order',
   'PostgreSQL is now the P0 retrieval authority for structured and full-text search.',
   'retrieval order postgresql structured full-text semantic'),
  ('sec-arch-decisions-2', 'doc-arch-decisions', 2, 'AD-008: Dual-Key Artifact Typing', 2, 'ad-008-dual-key-artifact-typing',
   'template+subtype as canonical classification key for artifact rendering and routing.',
   'artifact typing template subtype dual-key'),
  ('sec-arch-decisions-3', 'doc-arch-decisions', 3, 'AD-009: Seat Three-Layer Model and Delegation Overlay', 2, 'ad-009-seat-three-layer-model',
   'SeatIdentity (global), ProjectRoleBind (per-project), SeatDelegation (overlay).',
   'seat three-layer identity role delegation'),

  ('sec-coordination-rules-1', 'doc-coordination-rules', 1, 'Why this exists', 2, 'why-this-exists',
   'Terminal output is ephemeral. This rule set makes collaboration durable.',
   'coordination rules durable file-first writeback'),
  ('sec-coordination-rules-2', 'doc-coordination-rules', 2, 'Mandatory writeback rule', 2, 'mandatory-writeback-rule',
   'No role may claim completion without writeback.',
   'writeback mandatory completion artifacts'),

  ('sec-review-process-1', 'doc-review-process-mapping', 1, 'Seat Registration', 2, 'seat-registration',
   '5 seats: aegis (supervisor), lyra (PO), mira (designer), nimbus (architect), flux (verifier).',
   'seat registration aegis lyra mira nimbus flux'),
  ('sec-review-process-2', 'doc-review-process-mapping', 2, 'WorkItem Lifecycle', 2, 'workitem-lifecycle',
   '11 WorkItems across 2 phases: product design and design refinement.',
   'workitem lifecycle phases product design')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Document associations (link documents to project objects)
-- =============================================================================

INSERT INTO document_associations (id, document_id, assoc_type, assoc_id, is_primary)
VALUES
  -- PRD/specs associate at project scope
  ('da-prd-proj',          'doc-prd-v05',            'project', 'seatloom', true),
  ('da-int-proj',          'doc-interaction-v11',    'project', 'seatloom', true),
  ('da-ux-proj',           'doc-ux-v11',             'project', 'seatloom', true),
  ('da-acc-proj',          'doc-acceptance-v11',     'project', 'seatloom', true),
  ('da-arch-dec-proj',     'doc-arch-decisions',     'project', 'seatloom', true),
  ('da-arch-des-proj',     'doc-arch-design',        'project', 'seatloom', true),
  ('da-truth-proj',        'doc-product-truth',      'project', 'seatloom', true),
  ('da-coord-proj',        'doc-coordination-rules', 'project', 'seatloom', true),
  ('da-memory-proj',       'doc-memory-2026-04-29',  'project', 'seatloom', true),
  -- Task packets associate with their workitems
  ('da-task-hard-wi',      'doc-task-hardening',     'workitem', 'wi-hardening', true),
  ('da-task-db-wi',        'doc-task-db-baseline',   'workitem', 'wi-storage',   true),
  -- Acceptance docs associate with their workitems
  ('da-acc-hard-wi',       'doc-acceptance-hardening', 'workitem', 'wi-hardening', true),
  -- Process mapping review at project scope
  ('da-review-pm-proj',    'doc-review-process-mapping', 'project', 'seatloom', true)
ON CONFLICT (id) DO NOTHING;

COMMIT;
