-- 016_acceptance_plan.sql
-- =============================================================================
-- Structured acceptance plan (acceptance-lineage milestone, Lane C / migration
-- slot 016) — the acceptance card must carry the product-operation script
-- (scenario / steps / expected), and a card without a plan must not exist.
--
-- Authority: docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md
-- (Lyra sign-off 2026-10-10) §1 (shape v1) and §2 (enforcement); dispatched to
-- Onyx by disp-accplan-lane-c-onyx. The operator's blocking feedback on the
-- acceptance card (branch + one-line how_to_verify only — no cause-and-effect,
-- no acceptance plan) split into two DISTINCT gaps: lineage is a pure display
-- gap (Nimbus/Mira lanes, NO new storage), the acceptance plan is a REAL data
-- gap (this migration). The plan rides the existing AcceptanceRequested event
-- payload and lands on the acceptances projection as ONE JSONB column — same
-- pattern as dispatches.payload JSONB (012). NO new table, NO new event type,
-- NO new ref_type (contract §6 red lines).
--
-- Slot 016: next after 014 (015 is the parallel D5 knowledge-layer migration
-- dispatched to the same seat — the two are serialized by the seat, never
-- co-edited; 006/007 stay reserved). POST-SEED — it adds a column to and
-- constrains a table (acceptances) the seed never writes, and 014 (POST-SEED)
-- must have created that table first. Canonical order 014 -> 016
-- (apply-db.sh). See scripts/apply-db.sh.
--
-- Additive and idempotent, 010/012/013 house style: guarded DO-block +
-- ADD COLUMN IF NOT EXISTS, so a SECOND apply performs NO catalog write
-- (verified in the delivery doc).
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- Column form: jsonb / NOT NULL / DEFAULT '{}'
-- SELECT column_name, data_type, is_nullable, column_default
--   FROM information_schema.columns
--  WHERE table_name='acceptances' AND column_name='acceptance_plan';
-- -- CHECK form: present, convalidated = f (NOT VALID), definition contains <> '{}'
-- SELECT conname, convalidated, pg_get_constraintdef(oid)
--   FROM pg_constraint WHERE conname = 'acceptances_plan_required';
-- -- Negative case: INSERT ... acceptance_plan='{}' is REJECTED (new rows enforced)
-- -- Positive case: acceptance_plan='{"plan_version":1,...}' is ACCEPTED
-- -- Idempotency: a second `psql -f 016_acceptance_plan.sql` performs NO catalog
-- -- write (DO-block guard-skips; ADD COLUMN IF NOT EXISTS no-ops).
-- =============================================================================

BEGIN;

-- =============================================================================
-- acceptances.acceptance_plan — ONE JSONB column, shape v1 (contract §1).
-- =============================================================================
-- NOT NULL DEFAULT '{}'::jsonb makes the column additive-safe: the pre-existing
-- pending row(s) backfill harmlessly to '{}' and stay readable — '{}' is
-- explicitly NOT a plan, which is exactly why the CHECK below is NOT VALID for
-- now (the legacy pending row is exempt until Aegis backfills Lyra's authored
-- plan for it and VALIDATEs the constraint — a live-DB act, Aegis lane, NOT
-- done here). New rows are held to the full CHECK from the moment it is added.
--
-- The plan is authored by the PO (Lyra) at request time and is immutable after
-- (same immutability rule as how_to_verify). Shape validation (exact keys,
-- non-empty scenario/steps/expected) lives in the TOOL layer
-- (request_acceptance: a missing/malformed plan is a TYPED rejection before any
-- DB contact — contract §2, house discipline: typed errors, never a DB-error
-- crutch). The DB enforces ONLY non-emptiness here; it does not re-implement
-- the shape validator.
ALTER TABLE acceptances ADD COLUMN IF NOT EXISTS acceptance_plan JSONB NOT NULL DEFAULT '{}'::jsonb;

-- =============================================================================
-- acceptances_plan_required — no-plan-no-card (contract §2, DB layer).
-- =============================================================================
-- CHECK (acceptance_plan <> '{}'::jsonb) — empty object is not a plan.
--
-- NOT VALID IS DELIBERATE, NOT AN OVERSIGHT: NOT VALID exempts existing rows
-- from the check while the constraint still applies to every INSERT/UPDATE.
-- The one legacy pending acceptance row (acc-disp-me6Nm8e9D6ZRe4fKyVokY, Mira
-- D2 UI) currently carries no plan. It is exempt until Aegis backfills
-- Lyra's authored plan for it and runs
--   ALTER TABLE acceptances VALIDATE CONSTRAINT acceptances_plan_required;
-- as a live-DB act on the Aegis lane. Whoever reads this migration next: the
-- constraint is intentionally left un-validated — do not "fix" it blindly, the
-- handoff above is the procedure.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'acceptances_plan_required') THEN
    ALTER TABLE acceptances
      ADD CONSTRAINT acceptances_plan_required
      CHECK (acceptance_plan <> '{}'::jsonb) NOT VALID;
  END IF;
END $$;

-- =============================================================================
-- COMMENTs — the reasoning is part of the artifact (010/012 house style).
-- The column comment quotes the contract's shape v1 VERBATIM (same text as
-- Lane A/B/D carry) so the shape has one authoritative wording.
-- =============================================================================
COMMENT ON COLUMN acceptances.acceptance_plan IS
  'ACCEPTANCE PLAN — shape v1 (value-domain contract §1, Lyra sign-off 2026-10-10): acceptance_plan JSONB, an object with EXACTLY these keys (unknown key = typed rejection): plan_version INTEGER = 1; scenario STRING non-empty (what the user is trying to achieve, product language); steps ARRAY non-empty (the product-operation script; each item: action STRING non-empty = where to click / what to fill / what to say, input STRING optional = the input for that step); expected ARRAY non-empty (each item: output STRING non-empty = the observable output, expect STRING non-empty = the pass criterion, seeing X = pass); preconditions ARRAY of STRING, optional (empty allowed) = required prior state. ''{}'' is NOT a plan. One plan per acceptance, set at request time, immutable after (same immutability rule as how_to_verify). Shape validation lives in the TOOL layer (request_acceptance typed rejection before DB contact); the DB only enforces non-emptiness via acceptances_plan_required (CHECK acceptance_plan <> ''{}''::jsonb, added NOT VALID — the one legacy pending row is exempt until Aegis backfills Lyra''s authored plan and VALIDATEs; that backfill is a live-DB act, Aegis lane). Contract: docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md §1-§2.';

COMMIT;
