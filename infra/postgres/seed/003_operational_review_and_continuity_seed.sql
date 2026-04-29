-- SeatLoom Operational Review + Continuity Seed v1
-- Evidence basis:
--   - handoff receipts derived from explicit HandoffAccepted events in 001 seed
--   - checkpoints reconstructed from accepted 2026-04-29 collaboration outcomes
--   - review thread reconstructed from the explicit SG-01 reject -> pass evidence chain
--   - pipeline run reconstructed from the accepted Rust foundation verification gate
-- Approximation note:
--   exact checkpoint body text and pipeline metadata are curated summaries of the
--   accepted evidence, not verbatim runtime transcript slices.

BEGIN;

-- =============================================================================
-- Checkpoints (representative continuity snapshots)
-- =============================================================================
INSERT INTO checkpoints (
    id, session_id, trigger,
    summary_what_was_done, summary_current_state, summary_open_questions, summary_quality,
    artifact_ids_at_checkpoint, branch, last_commit, transcript_tail_ref,
    continuity_tier0, continuity_tier1, continuity_tier2, continuity_budget_tokens,
    delta_context, created_at
) VALUES
  (
    'cp-nimbus-infra-001', 'ses-nimbus-infra-001', 'session_ended',
    'Nimbus completed storage, PostgreSQL baseline, typed document authority, and foundation hardening work with passing local Rust verification.',
    'Infrastructure baseline is clean and compile-verified; PostgreSQL is the authority and the next step is higher-level acceptance plus remaining operational schema completion.',
    ARRAY['Should pipeline runs also capture remote verification evidence by commit hash?'], 'full',
    ARRAY['ar-task-storage','ar-task-hardening','ar-acceptance-hardening'],
    'main', 'a658086b54323259fda2ad2a958d097701f1fbbd', NULL,
    '{"seat":"nimbus","runtime":"ClaudeCode","workspace":"seatloom","role":"architect"}'::jsonb,
    '{"active_workitems":["wi-storage","wi-hardening"],"last_checkpoint_basis":"accepted delivery packets","branch":"main"}'::jsonb,
    '{"key_decisions":["PostgreSQL replaces SQLite FTS5 as retrieval authority","Rust toolchain frozen to 1.95.0"],"accepted_gates":["foundation hardening PASS"]}'::jsonb,
    16384,
    '{"new_since_previous":["postgres authority","typed documents","hardening automation"],"risk":"low"}'::jsonb,
    '2026-04-29T23:10:00Z'
  ),
  (
    'cp-lyra-coord-001', 'ses-lyra-coord-001', 'session_ended',
    'Lyra closed SG-01, accepted Nimbus foundation packets, and consolidated the v0.5 contract set into the active product truth baseline.',
    'Coordination truth is stable; remaining work shifts from baseline closure to deeper data coverage and infrastructure completeness.',
    ARRAY['How should new markdown-authored collaboration data be ingested continuously after SeatLoom becomes the primary workspace?'], 'full',
    ARRAY['ar-product-truth','ar-prd-v05','ar-interaction-v11','ar-ux-v11','ar-acceptance-v11'],
    'main', 'd007721bd36e9fdd0e145e68a04e32c1bc4d9cf2', NULL,
    '{"seat":"lyra","runtime":"GeminiCli","workspace":"seatloom","role":"product_owner"}'::jsonb,
    '{"active_workitems":["wi-004","wi-hardening"],"pending_handoffs":[],"last_gate":"product baseline freeze GO"}'::jsonb,
    '{"decisions":["mobile is required for monitoring + feedback loops","template+subtype is canonical artifact typing"],"accepted_reviews":["product baseline freeze"]}'::jsonb,
    24576,
    '{"new_since_previous":["SG-01 GO","baseline freeze GO","PostgreSQL baseline PASS"],"risk":"medium"}'::jsonb,
    '2026-04-29T23:40:00Z'
  ),
  (
    'cp-flux-verify-001', 'ses-flux-verify-001', 'artifact_produced',
    'Flux completed the commit-pinned SG-01 UI verification pass and later the Rust foundation hardening verification pass.',
    'Verification seat is idle with a clean pass verdict recorded; future work should stay commit-pinned and evidence-first.',
    ARRAY['Should remote workspace verification be standardized as a pipeline run template?'], 'minimal',
    ARRAY['ar-acceptance-hardening'],
    'main', 'a658086b54323259fda2ad2a958d097701f1fbbd', NULL,
    '{"seat":"flux","runtime":"Custom","workspace":"seatloom","role":"verifier"}'::jsonb,
    '{"latest_verifications":["SG-01 UI baseline","rust foundation hardening"],"commit_pinned":true}'::jsonb,
    '{"policy":["read-only by default","commit-pinned acceptance"],"open_followup":["docker-capable PG verification standardized"]}'::jsonb,
    8192,
    '{"new_since_previous":["10/10 UI baseline PASS","foundation hardening PASS"],"risk":"low"}'::jsonb,
    '2026-04-29T23:20:00Z'
  )
ON CONFLICT (id) DO NOTHING;

UPDATE sessions SET last_checkpoint_id = 'cp-nimbus-infra-001' WHERE id = 'ses-nimbus-infra-001';
UPDATE sessions SET last_checkpoint_id = 'cp-lyra-coord-001' WHERE id = 'ses-lyra-coord-001';
UPDATE sessions SET last_checkpoint_id = 'cp-flux-verify-001' WHERE id = 'ses-flux-verify-001';

-- =============================================================================
-- Handoff receipts (derived from explicit acceptance events)
-- =============================================================================
INSERT INTO handoff_receipts (id, handoff_id, acknowledged_by, acknowledged_at, note, source_channel) VALUES
  (
    'hr-ho-aegis-nimbus-arch-001',
    'ho-aegis-nimbus-arch',
    'seat-nimbus-001',
    '2026-04-28T08:00:00Z',
    'Nimbus accepted the architecture baseline handoff and started the scaffold packet.',
    'desktop'
  ),
  (
    'hr-ho-nimbus-lyra-storage-001',
    'ho-nimbus-lyra-storage',
    'seat-lyra-001',
    '2026-04-29T13:00:00Z',
    'Lyra acknowledged the storage+ledger delivery for acceptance review.',
    'desktop'
  ),
  (
    'hr-ho-lyra-flux-sg01-verify-001',
    'ho-lyra-flux-sg01-verify',
    'seat-flux-001',
    '2026-04-29T15:00:00Z',
    'Flux acknowledged the bounded SG-01 verification packet and entered read-only audit mode.',
    'desktop'
  )
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Pipeline runs (representative deterministic verification run)
-- =============================================================================
INSERT INTO pipeline_runs (
    id, pipeline_id, workitem_id, status, current_stage, stage_index, trigger,
    initiated_by, result_summary, artifact_ids, evidence_refs, run_metadata,
    started_at, finished_at
) VALUES
  (
    'plrun-rust-foundation-gate-001',
    'pl-rust-foundation-gate',
    'wi-hardening',
    'completed',
    'accepted',
    4,
    'gate',
    'seat:seat-flux-001',
    'Rust foundation hardening gate completed PASS after cargo check/test/fmt/clippy and Lyra acceptance.',
    ARRAY['ar-task-hardening','ar-acceptance-hardening'],
    ARRAY[
      'docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-delivery-v1.md',
      'docs/coordination/tasks/flux/FLUX-2026-04-29-foundation-hardening-verification-delivery-v1.md',
      'docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md'
    ],
    '{"steps":["cargo check","cargo test -p seatloom-core","cargo fmt --all --check","cargo clippy -p seatloom-core --all-targets -- -D warnings"],"verifier":"flux","acceptor":"lyra","commit_pinned":true,"verdict":"pass"}'::jsonb,
    '2026-04-29T21:30:00Z',
    '2026-04-29T23:05:00Z'
  )
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- Review threads + comments (shared canonical review object family)
-- Grounded in the SG-01 reject -> pass evidence chain from the real collaboration.
-- =============================================================================
INSERT INTO review_threads (
    id, project_id, source_channel, mode,
    target_kind, target_id, workitem_id,
    anchor_kind, title, status, requires_followup, review_tier,
    change_tier_record, evidence_refs,
    created_by, assigned_to, created_at, updated_at, resolved_at, resolved_by
) VALUES (
    'rt-wi004-sg01-gap-001',
    'seatloom',
    'desktop',
    'manual',
    'workitem',
    'wi-004',
    'wi-004',
    'object',
    'SG-01 shell and workflow contract gaps',
    'resolved',
    true,
    'L3',
    '{
      "tier":"L3",
      "reason":"SG-01 fail revealed shell, Inbox, Timeline, Handoff, and keyboard-contract gaps that change delivery readiness semantics.",
      "changed_clauses":["UX-01","UX-03","UX-05","UX-11"],
      "impact_level":"high",
      "executor":"seat:seat-mira-001",
      "reviewer":"seat:seat-lyra-001",
      "ack_mode":"full_gate",
      "evidence_refs":["docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md"]
    }'::jsonb,
    ARRAY[
      'docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md',
      'docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-delivery-v1.md'
    ],
    'seat:seat-lyra-001',
    'seat:seat-mira-001',
    '2026-04-27T16:00:00Z',
    '2026-04-29T19:00:00Z',
    '2026-04-29T19:00:00Z',
    'seat:seat-lyra-001'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO review_comments (
    id, thread_id, parent_comment_id, author_ref, body_text,
    mode, source_channel, state, evidence_refs, comment_metadata,
    created_at, updated_at
) VALUES
  (
    'rc-wi004-sg01-gap-001',
    'rt-wi004-sg01-gap-001',
    NULL,
    'seat:seat-lyra-001',
    'SG-01 fails on shell truth, Inbox next-action guidance, Timeline drill-through, Handoff visibility, and keyboard contract alignment. Route a full-gate fix loop before closure.',
    'manual',
    'desktop',
    'resolved',
    ARRAY['docs/coordination/acceptance/2026-04-27-mira-ui-acceptance.md'],
    '{"source_event_id":"ev-030","verdict":"reject"}'::jsonb,
    '2026-04-27T16:00:00Z',
    '2026-04-27T16:00:00Z'
  ),
  (
    'rc-wi004-sg01-gap-002',
    'rt-wi004-sg01-gap-001',
    'rc-wi004-sg01-gap-001',
    'seat:seat-lyra-001',
    'Closed after S7A-S7C delivery and the 10/10 PASS evidence pack. Keep the thread linked to the acceptance trail for future retrieval.',
    'manual',
    'desktop',
    'active',
    ARRAY['docs/coordination/tasks/flux/FLUX-2026-04-29-v05-ui-baseline-evidence-pack-delivery-v1.md'],
    '{"source_event_id":"ev-031","verdict":"pass"}'::jsonb,
    '2026-04-29T19:00:00Z',
    '2026-04-29T19:00:00Z'
  )
ON CONFLICT (id) DO NOTHING;

COMMIT;
