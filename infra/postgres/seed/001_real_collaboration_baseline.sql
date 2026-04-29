-- SeatLoom Real Collaboration Baseline Seed
-- Evidence source: docs/coordination/MEMORY.md, docs/coordination/reviews/2026-04-28-process-mapping-review.md
-- Date range: 2026-04-27 through 2026-04-29
-- Approximation notes: session created_at derived from first-evidence timestamps in MEMORY.md

BEGIN;

-- =============================================================================
-- Project
-- =============================================================================
INSERT INTO projects (id, name, created_at, worker_budget_tokens, supervisor_budget_tokens)
VALUES ('seatloom', 'SeatLoom', '2026-04-27T00:00:00Z', 8192, 32768)
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Seats (real AI agent seats active during 2026-04-27 to 2026-04-29)
-- Evidence: docs/coordination/reviews/2026-04-28-process-mapping-review.md §1
-- =============================================================================
INSERT INTO seats (id, name, default_runtime, capability_tags, status, created_at) VALUES
  ('seat-aegis-001', 'aegis',  'ClaudeCode',  ARRAY['supervision','product-design','architecture-review','stage-gate'], 'active', '2026-04-24T00:00:00Z'),
  ('seat-lyra-001',  'lyra',   'GeminiCli',   ARRAY['product-ownership','coordination','acceptance','planning'],          'active', '2026-04-24T00:00:00Z'),
  ('seat-mira-001',  'mira',   'GeminiCli',   ARRAY['ux-design','react','typescript','component-prototyping'],            'active', '2026-04-24T00:00:00Z'),
  ('seat-nimbus-001','nimbus', 'ClaudeCode',  ARRAY['architecture','rust','tauri','infrastructure'],                      'active', '2026-04-24T00:00:00Z'),
  ('seat-flux-001',  'flux',   'Custom',      ARRAY['qa','verification','ui-audit','ops'],                                'active', '2026-04-24T00:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Project Role Bindings (per-project, seatloom project)
-- Evidence: process-mapping-review §1, MEMORY.md roles section
-- =============================================================================
INSERT INTO project_role_bindings (seat_id, project_id, role, authority_doc_refs, constraints, collaboration_template_ref, active_delegation_id)
VALUES
  ('seat-aegis-001',  'seatloom', 'supervisor',     ARRAY['docs/PRODUCT_TRUTH.md','docs/architecture-decisions.md'],  ARRAY[]::text[],                                          NULL, NULL),
  ('seat-lyra-001',   'seatloom', 'product_owner',  ARRAY['docs/PRODUCT_TRUTH.md','docs/prd-v0.5.md'],                ARRAY['file-first-coordination'],                 NULL, NULL),
  ('seat-mira-001',   'seatloom', 'designer',       ARRAY['docs/ux-spec-v1.1.md','docs/interaction-spec-v1.1.md'],    ARRAY['no-IA-redesign','ui-contract-only'],        NULL, NULL),
  ('seat-nimbus-001', 'seatloom', 'architect',      ARRAY['docs/architecture-design.md','docs/architecture-decisions.md'], ARRAY['no-runtime-widening','infrastructure-only'], NULL, NULL),
  ('seat-flux-001',   'seatloom', 'verifier',       ARRAY['docs/acceptance-spec-v1.1.md'],                            ARRAY['read-only-by-default','no-ui-redesign'],    NULL, NULL)
ON CONFLICT (seat_id, project_id) DO NOTHING;

-- =============================================================================
-- Seat Delegation: Flux acting for Mira (WI-009 contract repair)
-- Evidence: MEMORY.md 2026-04-28 "Flux temporarily reassigned as acting Mira seat"
-- =============================================================================
INSERT INTO seat_delegations (id, issuer_seat_id, from_seat_id, to_seat_id, workitem_id, scope_description, issued_at, expires_at, status)
VALUES (
  'del-flux-acting-mira-001',
  'seat-lyra-001',
  'seat-mira-001',
  'seat-flux-001',
  'wi-009',
  'Flux temporarily takes the acting-Mira role for the bounded UI/UED contract-repair packet (WI-009). No authority to redesign product IA. Scope: constrained UI contract repair under Lyra supervision only.',
  '2026-04-28T09:00:00Z',
  '2026-04-28T23:59:00Z',
  'closed'
) ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Sessions (one representative session per seat, Apr 27-29)
-- Evidence: process-mapping-review §3, MEMORY.md
-- Approximated timestamps from first/last evidence entries
-- =============================================================================
INSERT INTO sessions (id, seat_id, runtime, workspace_path, branch, status, created_at, ended_at) VALUES
  ('ses-aegis-supervisor-001', 'seat-aegis-001', 'ClaudeCode', '/Users/jyxc-dz-0100609/Documents/GitHub/seatloom', 'main', 'completed', '2026-04-27T08:00:00Z', '2026-04-29T23:59:00Z'),
  ('ses-lyra-coord-001',       'seat-lyra-001',  'GeminiCli',  '/Users/jyxc-dz-0100609/Documents/GitHub/seatloom', 'main', 'completed', '2026-04-27T09:00:00Z', '2026-04-29T23:59:00Z'),
  ('ses-mira-ui-001',          'seat-mira-001',  'GeminiCli',  '/Users/jyxc-dz-0100609/Documents/GitHub/seatloom', 'main', 'completed', '2026-04-27T10:00:00Z', '2026-04-29T20:00:00Z'),
  ('ses-nimbus-infra-001',     'seat-nimbus-001','ClaudeCode', '/Users/jyxc-dz-0100609/Documents/GitHub/seatloom', 'main', 'completed', '2026-04-28T08:00:00Z', '2026-04-29T23:59:00Z'),
  ('ses-flux-verify-001',      'seat-flux-001',  'Custom',     '/Users/jyxc-dz-0100609/Documents/GitHub/seatloom', 'main', 'completed', '2026-04-27T11:00:00Z', '2026-04-29T22:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- WorkItems (real collaboration lanes from process-mapping-review §2)
-- =============================================================================
INSERT INTO workitems (id, title, goal, acceptance_criteria, owner_seat_id, status, priority, created_at, updated_at) VALUES
  ('wi-001', 'Define product form and tech stack',
   'Establish SeatLoom architecture decisions (Tauri 2 + React/TS + Rust), product form (desktop + CLI), and all foundational ADRs.',
   ARRAY['architecture-decisions.md written and approved','Tech stack rationale documented'],
   'seat-aegis-001', 'done', 'high',
   '2026-04-27T08:00:00Z', '2026-04-27T12:00:00Z'),

  ('wi-004', 'Deliver Phase 1 UX prototypes',
   'Build the initial SeatLoom React/TypeScript UI baseline aligned to the product contract.',
   ARRAY['App shell renders','Inbox and Timeline views functional','SG-01 acceptance criteria met'],
   'seat-mira-001', 'done', 'high',
   '2026-04-27T10:00:00Z', '2026-04-29T18:00:00Z'),

  ('wi-009', 'Flux acting-Mira UI contract repair',
   'Repair the SG-01 UI contract gaps under Lyra supervision, with Flux acting in the Mira seat.',
   ARRAY['Shell truth restored','Inbox and Timeline surfaces aligned','UI build green','No IA redesign'],
   'seat-flux-001', 'done', 'high',
   '2026-04-28T09:00:00Z', '2026-04-28T23:00:00Z'),

  ('wi-scaffold', 'Nimbus Rust foundation scaffold',
   'Bootstrap the full Cargo workspace with seatloom-core types, Tauri stubs, CLI binary, and compile-safe scaffold.',
   ARRAY['Workspace compiles','All core object types defined','Tauri IPC stubs exist','CLI binary present'],
   'seat-nimbus-001', 'done', 'high',
   '2026-04-28T08:00:00Z', '2026-04-28T20:00:00Z'),

  ('wi-storage', 'Nimbus storage and ledger foundation',
   'Implement deterministic YAML/JSONL project IO, seat registry, delegation storage, and ledger append/read primitives.',
   ARRAY['YAML round-trips','JSONL append+read','SeatRegistry with typed validation','9 tests pass'],
   'seat-nimbus-001', 'done', 'high',
   '2026-04-29T08:00:00Z', '2026-04-29T16:00:00Z'),

  ('wi-hardening', 'Nimbus Rust foundation hardening',
   'Freeze the Rust toolchain to 1.95.0, close all clippy/fmt failures, eliminate warning noise, and set up CI.',
   ARRAY['Exact 1.95.0 toolchain frozen','fmt passes','clippy -D warnings passes','0 warnings cargo check','CI workflow live'],
   'seat-nimbus-001', 'done', 'high',
   '2026-04-29T16:00:00Z', '2026-04-29T22:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Handoffs (real seat-to-seat transfers evidenced in MEMORY.md)
-- =============================================================================
INSERT INTO handoffs (id, from_ref, to_ref, workitem_id, purpose, expected_outcome, required_receipt, status, created_at, sent_at) VALUES
  ('ho-aegis-nimbus-arch',
   'seat:seat-aegis-001', 'seat:seat-nimbus-001', 'wi-scaffold',
   'Hand off the architecture baseline (arch-design v1.0 + arch-decisions) to Nimbus for implementation start.',
   'Nimbus begins foundation scaffold aligned to accepted architecture contract.',
   true, 'completed',
   '2026-04-28T07:00:00Z', '2026-04-28T07:30:00Z'),

  ('ho-lyra-flux-sg01-verify',
   'seat:seat-lyra-001', 'seat:seat-flux-001', 'wi-004',
   'Activate Flux in read-only verification mode to produce a clean SG-01 baseline evidence pack after all UI gaps are closed.',
   '10/10 SG-01 surfaces verified PASS, evidence pack delivered, build confirmed green.',
   true, 'completed',
   '2026-04-29T14:00:00Z', '2026-04-29T14:30:00Z'),

  ('ho-nimbus-lyra-storage',
   'seat:seat-nimbus-001', 'seat:seat-lyra-001', 'wi-storage',
   'Deliver the storage+ledger foundation packet for Lyra acceptance review.',
   'YAML/JSONL IO, seat registry, delegation storage, and 9 passing tests accepted at PASS.',
   true, 'completed',
   '2026-04-29T12:00:00Z', '2026-04-29T12:30:00Z')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Artifacts (typed metadata — payloads remain on disk)
-- Template+subtype from DOCUMENT_TEMPLATES.md §11.1
-- =============================================================================

-- T1 Authority Docs (active contract set)
INSERT INTO artifacts (id, template, subtype, subtype_valid, title, storage_path, created_at) VALUES
  ('ar-product-truth',       'T1AuthorityDoc', 'prd',                    true, 'SeatLoom Product Truth Index',          'docs/PRODUCT_TRUTH.md',              '2026-04-28T12:00:00Z'),
  ('ar-prd-v05',             'T1AuthorityDoc', 'prd',                    true, 'SeatLoom PRD v0.5',                     'docs/prd-v0.5.md',                   '2026-04-28T14:00:00Z'),
  ('ar-interaction-v11',     'T1AuthorityDoc', 'interaction_spec',        true, 'SeatLoom Interaction Spec v1.1',        'docs/interaction-spec-v1.1.md',      '2026-04-28T15:00:00Z'),
  ('ar-ux-v11',              'T1AuthorityDoc', 'ux_spec',                 true, 'SeatLoom UX Spec v1.1',                 'docs/ux-spec-v1.1.md',               '2026-04-28T15:30:00Z'),
  ('ar-acceptance-v11',      'T1AuthorityDoc', 'acceptance_spec',         true, 'SeatLoom Acceptance Spec v1.1',         'docs/acceptance-spec-v1.1.md',       '2026-04-28T16:00:00Z'),
  ('ar-arch-decisions',      'T1AuthorityDoc', 'architecture_decisions',  true, 'SeatLoom Architecture Decisions',       'docs/architecture-decisions.md',     '2026-04-28T10:00:00Z'),
  ('ar-arch-design',         'T1AuthorityDoc', 'architecture_design',     true, 'SeatLoom Architecture Design v1.0',     'docs/architecture-design.md',        '2026-04-28T10:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- T2 Role Profile
INSERT INTO artifacts (id, template, subtype, subtype_valid, title, storage_path, created_at) VALUES
  ('ar-role-mira', 'T2RoleProfile', 'seat_role', true, 'Mira – UX/UED Designer Role Profile', 'docs/coordination/roles/MIRA.md', '2026-04-27T12:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- T3 Task Packets (key Nimbus deliveries)
INSERT INTO artifacts (id, template, subtype, subtype_valid, title, source_workitem_id, storage_path, created_at) VALUES
  ('ar-task-hardening', 'T3TaskPacket', 'task', true,
   'NIMBUS-2026-04-29 Rust Foundation Hardening',
   'wi-hardening',
   'docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-v1.md',
   '2026-04-29T16:00:00Z'),
  ('ar-task-storage', 'T3TaskPacket', 'task', true,
   'NIMBUS-2026-04-29 Storage and Ledger Foundation',
   'wi-storage',
   'docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-storage-ledger-foundation-v1.md',
   '2026-04-29T08:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- T4 Review
INSERT INTO artifacts (id, template, subtype, subtype_valid, title, storage_path, created_at) VALUES
  ('ar-review-process-mapping', 'T4Review', 'process_mapping', true,
   'SeatLoom Process Mapping: Real Workflow → Product Model',
   'docs/coordination/reviews/2026-04-28-process-mapping-review.md',
   '2026-04-28T11:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- T5 Acceptance
INSERT INTO artifacts (id, template, subtype, subtype_valid, title, source_workitem_id, storage_path, created_at) VALUES
  ('ar-acceptance-hardening', 'T5Acceptance', 'acceptance_review', true,
   'Lyra Acceptance: Nimbus Rust Foundation Hardening – PASS',
   'wi-hardening',
   'docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md',
   '2026-04-29T23:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- T6 Daily Memory
INSERT INTO artifacts (id, template, subtype, subtype_valid, title, storage_path, created_at) VALUES
  ('ar-memory-2026-04-29', 'T6DailyMemory', 'daily_log', true,
   'Coordination Daily Memory 2026-04-29',
   'docs/coordination/memory/2026-04-29.md',
   '2026-04-29T00:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- T7 Governance
INSERT INTO artifacts (id, template, subtype, subtype_valid, title, storage_path, created_at) VALUES
  ('ar-governance-coordination-rules', 'T7GovernanceDoc', 'coordination_rules', true,
   'SeatLoom Coordination Rules v1.0',
   'docs/coordination/COORDINATION_RULES.md',
   '2026-04-27T15:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Canonical Events (selected real events from the collaboration timeline)
-- Evidence: MEMORY.md durable decisions and milestone status
-- =============================================================================

-- Session started events
INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload) VALUES
  ('ev-001', 'SessionStarted', '2026-04-27T08:00:00Z', 'seat:seat-aegis-001',  '{"runtime":"ClaudeCode","note":"Aegis supervisor session begins"}'),
  ('ev-002', 'SessionStarted', '2026-04-27T09:00:00Z', 'seat:seat-lyra-001',   '{"runtime":"GeminiCli","note":"Lyra PO coordination session begins"}'),
  ('ev-003', 'SessionStarted', '2026-04-27T10:00:00Z', 'seat:seat-mira-001',   '{"runtime":"GeminiCli","note":"Mira UI design session begins"}'),
  ('ev-004', 'SessionStarted', '2026-04-28T08:00:00Z', 'seat:seat-nimbus-001', '{"runtime":"ClaudeCode","note":"Nimbus infra engineering session begins"}'),
  ('ev-005', 'SessionStarted', '2026-04-27T11:00:00Z', 'seat:seat-flux-001',   '{"runtime":"Custom","note":"Flux verification session begins"}')
ON CONFLICT (id) DO NOTHING;

-- WorkItem created events
INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload) VALUES
  ('ev-010', 'WorkItemCreated', '2026-04-27T08:30:00Z', 'seat:seat-aegis-001',  '{"workitem_id":"wi-001","title":"Define product form and tech stack"}'),
  ('ev-011', 'WorkItemCreated', '2026-04-27T10:30:00Z', 'seat:seat-lyra-001',   '{"workitem_id":"wi-004","title":"Deliver Phase 1 UX prototypes"}'),
  ('ev-012', 'WorkItemCreated', '2026-04-28T09:00:00Z', 'seat:seat-lyra-001',   '{"workitem_id":"wi-009","title":"Flux acting-Mira UI contract repair"}'),
  ('ev-013', 'WorkItemCreated', '2026-04-28T08:00:00Z', 'seat:seat-lyra-001',   '{"workitem_id":"wi-scaffold","title":"Nimbus Rust foundation scaffold"}'),
  ('ev-014', 'WorkItemCreated', '2026-04-29T08:00:00Z', 'seat:seat-lyra-001',   '{"workitem_id":"wi-storage","title":"Nimbus storage and ledger foundation"}'),
  ('ev-015', 'WorkItemCreated', '2026-04-29T16:00:00Z', 'seat:seat-lyra-001',   '{"workitem_id":"wi-hardening","title":"Nimbus Rust foundation hardening"}')
ON CONFLICT (id) DO NOTHING;

-- Delegation issued event
INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload) VALUES
  ('ev-020', 'SeatDelegationIssued', '2026-04-28T09:00:00Z', 'seat:seat-lyra-001',
   '{"delegation_id":"del-flux-acting-mira-001","from_seat":"mira","to_seat":"flux","scope":"bounded UI contract repair WI-009","note":"Mira offline; Flux takes acting-Mira role under Lyra supervision"}')
ON CONFLICT (id) DO NOTHING;

-- Delegation closed event
INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload) VALUES
  ('ev-021', 'SeatDelegationClosed', '2026-04-28T23:00:00Z', 'seat:seat-lyra-001',
   '{"delegation_id":"del-flux-acting-mira-001","reason":"WI-009 delivered and accepted; Mira returns as primary UI seat"}')
ON CONFLICT (id) DO NOTHING;

-- WI-004 review verdict (first SG-01 fail, then pass)
INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload) VALUES
  ('ev-030', 'ReviewVerdictIssued', '2026-04-27T16:00:00Z', 'seat:seat-lyra-001',
   '{"workitem_id":"wi-004","verdict":"reject","reason":"SG-01 FAIL – shell, Inbox, Timeline, Handoff, keyboard contract gaps","linked_evidence":"docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md"}'),
  ('ev-031', 'ReviewVerdictIssued', '2026-04-29T19:00:00Z', 'seat:seat-lyra-001',
   '{"workitem_id":"wi-004","verdict":"pass","reason":"SG-01 GO – all P0 UI surfaces accepted after S7A-S7C closure","linked_evidence":"docs/coordination/acceptance/"}')
ON CONFLICT (id) DO NOTHING;

-- Handoff events
INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload) VALUES
  ('ev-040', 'HandoffSent',     '2026-04-28T07:30:00Z', 'seat:seat-aegis-001',  '{"handoff_id":"ho-aegis-nimbus-arch","to":"nimbus","workitem_id":"wi-scaffold"}'),
  ('ev-041', 'HandoffAccepted', '2026-04-28T08:00:00Z', 'seat:seat-nimbus-001', '{"handoff_id":"ho-aegis-nimbus-arch","from":"aegis","workitem_id":"wi-scaffold"}'),
  ('ev-042', 'HandoffSent',     '2026-04-29T12:30:00Z', 'seat:seat-nimbus-001', '{"handoff_id":"ho-nimbus-lyra-storage","to":"lyra","workitem_id":"wi-storage"}'),
  ('ev-043', 'HandoffAccepted', '2026-04-29T13:00:00Z', 'seat:seat-lyra-001',   '{"handoff_id":"ho-nimbus-lyra-storage","from":"nimbus","workitem_id":"wi-storage"}'),
  ('ev-044', 'HandoffSent',     '2026-04-29T14:30:00Z', 'seat:seat-lyra-001',   '{"handoff_id":"ho-lyra-flux-sg01-verify","to":"flux","workitem_id":"wi-004"}'),
  ('ev-045', 'HandoffAccepted', '2026-04-29T15:00:00Z', 'seat:seat-flux-001',   '{"handoff_id":"ho-lyra-flux-sg01-verify","from":"lyra","workitem_id":"wi-004"}'),
  ('ev-046', 'HandoffCompleted','2026-04-29T16:00:00Z', 'seat:seat-flux-001',   '{"handoff_id":"ho-lyra-flux-sg01-verify","outcome":"10/10 PASS evidence pack delivered"}')
ON CONFLICT (id) DO NOTHING;

-- Artifact created events (active contract set)
INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload) VALUES
  ('ev-050', 'ArtifactCreated', '2026-04-27T12:00:00Z', 'seat:seat-aegis-001', '{"artifact_id":"ar-arch-decisions","title":"Architecture Decisions","template":"T1AuthorityDoc","subtype":"architecture_decisions"}'),
  ('ev-051', 'ArtifactCreated', '2026-04-28T14:00:00Z', 'seat:seat-lyra-001',  '{"artifact_id":"ar-prd-v05","title":"PRD v0.5","template":"T1AuthorityDoc","subtype":"prd"}'),
  ('ev-052', 'ArtifactCreated', '2026-04-28T15:00:00Z', 'seat:seat-lyra-001',  '{"artifact_id":"ar-interaction-v11","title":"Interaction Spec v1.1","template":"T1AuthorityDoc","subtype":"interaction_spec"}'),
  ('ev-053', 'ArtifactCreated', '2026-04-28T12:00:00Z', 'seat:seat-lyra-001',  '{"artifact_id":"ar-product-truth","title":"Product Truth Index","template":"T1AuthorityDoc","subtype":"prd"}'),
  ('ev-054', 'ArtifactCreated', '2026-04-28T11:00:00Z', 'seat:seat-aegis-001', '{"artifact_id":"ar-review-process-mapping","title":"Process Mapping Review","template":"T4Review","subtype":"process_mapping"}'),
  ('ev-055', 'ArtifactCreated', '2026-04-29T23:00:00Z', 'seat:seat-lyra-001',  '{"artifact_id":"ar-acceptance-hardening","title":"Foundation Hardening Acceptance","template":"T5Acceptance","subtype":"acceptance_review"}')
ON CONFLICT (id) DO NOTHING;

-- WorkItem status transitions (key ones)
INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload) VALUES
  ('ev-060', 'WorkItemStatusChanged', '2026-04-27T12:00:00Z', 'seat:seat-aegis-001', '{"workitem_id":"wi-001","old_status":"active","new_status":"done","note":"Architecture decisions finalized"}'),
  ('ev-061', 'WorkItemStatusChanged', '2026-04-28T23:00:00Z', 'seat:seat-lyra-001',  '{"workitem_id":"wi-009","old_status":"active","new_status":"done","note":"Acting-Mira contract repair accepted"}'),
  ('ev-062', 'WorkItemStatusChanged', '2026-04-29T16:00:00Z', 'seat:seat-lyra-001',  '{"workitem_id":"wi-scaffold","old_status":"active","new_status":"done","note":"Foundation scaffold accepted PASS"}'),
  ('ev-063', 'WorkItemStatusChanged', '2026-04-29T15:00:00Z', 'seat:seat-lyra-001',  '{"workitem_id":"wi-storage","old_status":"active","new_status":"done","note":"Storage+ledger foundation accepted PASS"}'),
  ('ev-064', 'WorkItemStatusChanged', '2026-04-29T23:00:00Z', 'seat:seat-lyra-001',  '{"workitem_id":"wi-hardening","old_status":"active","new_status":"done","note":"Foundation hardening accepted PASS"}'),
  ('ev-065', 'WorkItemStatusChanged', '2026-04-29T19:00:00Z', 'seat:seat-lyra-001',  '{"workitem_id":"wi-004","old_status":"in_review","new_status":"done","note":"SG-01 GO — all P0 UI surfaces accepted"}')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Event Object References (link events to their objects)
-- =============================================================================
INSERT INTO event_object_refs (event_id, ref_type, ref_id) VALUES
  ('ev-001', 'session',  'ses-aegis-supervisor-001'),
  ('ev-002', 'session',  'ses-lyra-coord-001'),
  ('ev-003', 'session',  'ses-mira-ui-001'),
  ('ev-004', 'session',  'ses-nimbus-infra-001'),
  ('ev-005', 'session',  'ses-flux-verify-001'),
  ('ev-010', 'workitem', 'wi-001'),
  ('ev-011', 'workitem', 'wi-004'),
  ('ev-012', 'workitem', 'wi-009'),
  ('ev-013', 'workitem', 'wi-scaffold'),
  ('ev-014', 'workitem', 'wi-storage'),
  ('ev-015', 'workitem', 'wi-hardening'),
  ('ev-020', 'delegation', 'del-flux-acting-mira-001'),
  ('ev-021', 'delegation', 'del-flux-acting-mira-001'),
  ('ev-030', 'workitem', 'wi-004'),
  ('ev-031', 'workitem', 'wi-004'),
  ('ev-040', 'handoff',  'ho-aegis-nimbus-arch'),   ('ev-040', 'workitem', 'wi-scaffold'),
  ('ev-041', 'handoff',  'ho-aegis-nimbus-arch'),   ('ev-041', 'workitem', 'wi-scaffold'),
  ('ev-042', 'handoff',  'ho-nimbus-lyra-storage'), ('ev-042', 'workitem', 'wi-storage'),
  ('ev-043', 'handoff',  'ho-nimbus-lyra-storage'), ('ev-043', 'workitem', 'wi-storage'),
  ('ev-044', 'handoff',  'ho-lyra-flux-sg01-verify'), ('ev-044', 'workitem', 'wi-004'),
  ('ev-045', 'handoff',  'ho-lyra-flux-sg01-verify'), ('ev-045', 'workitem', 'wi-004'),
  ('ev-046', 'handoff',  'ho-lyra-flux-sg01-verify'), ('ev-046', 'workitem', 'wi-004'),
  ('ev-050', 'artifact', 'ar-arch-decisions'),
  ('ev-051', 'artifact', 'ar-prd-v05'),
  ('ev-052', 'artifact', 'ar-interaction-v11'),
  ('ev-053', 'artifact', 'ar-product-truth'),
  ('ev-054', 'artifact', 'ar-review-process-mapping'),
  ('ev-055', 'artifact', 'ar-acceptance-hardening'),
  ('ev-060', 'workitem', 'wi-001'),
  ('ev-061', 'workitem', 'wi-009'),
  ('ev-062', 'workitem', 'wi-scaffold'),
  ('ev-063', 'workitem', 'wi-storage'),
  ('ev-064', 'workitem', 'wi-hardening'),
  ('ev-065', 'workitem', 'wi-004')
ON CONFLICT DO NOTHING;

COMMIT;
