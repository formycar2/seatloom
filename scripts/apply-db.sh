#!/usr/bin/env bash
# apply-db.sh — apply SeatLoom's schema + seed to a database in the canonical
# order, using $DATABASE_URL (CI-compatible; no docker/podman exec).
#
# =============================================================================
# ORDERING CONTRACT — every migration is classified BY RULE, not by its number.
# =============================================================================
# A migration's placement follows from its relationship to the seed:
#
#   PRE-SEED   the seed writes columns that this migration creates.
#              The seed physically cannot insert into a column that does not
#              exist yet.
#
#   POST-SEED  this migration constrains columns the seed does not provide —
#              typically ADD COLUMN, backfill from seeded rows, SET NOT NULL.
#              It needs the seeded rows to already exist.
#
# A future migration could in principle need BOTH (creating a column the seed
# writes AND constraining a column the seed omits). None does today. If one
# ever does, SPLIT it into a pre-seed part and a post-seed part. Do not try to
# satisfy both halves with a single placement, and do not reorder the seed.
#
# WHY AN EXPLICIT LIST AND NOT A NUMBER RANGE
# The previous rule was "001-005 pre-seed, >=006 post-seed". That is a number
# heuristic standing in for the rule above, and the two agree only while every
# column-adding migration happens to be numbered <=005. It broke as soon as one
# was not: 009_runtime_triple adds seats.default_harness/_agent/_model and
# sessions.harness/agent/model, all of which seed 001 writes, so the >=006
# bucket scheduled it AFTER the seed that depends on it:
#
#   --- seed (001-004) ---
#     infra/postgres/seed/001_real_collaboration_baseline.sql
#   ERROR:  column "default_harness" of relation "seats" does not exist
#   --- apply-db.sh exit code: 3 ---
#
# (Flux commit-pinned gate 2026-09-30 —
#  docs/coordination/tasks/flux/FLUX-2026-09-30-runtime-triple-onyx-verification-delivery-v2.md)
#
# docs/ENGINEERING_STANDARDS.md (Migrations) already recorded the bucket as
# "convention, not enforced structure". This replaces the heuristic with the
# rule it was approximating, and states the reason per entry so the next
# migration is classified by rule rather than by number. A proper
# ordered-migration + separate-idempotent-seed runner stays a tracked Onyx
# follow-up; this only makes the stopgap honest about why each file sits where
# it does.
#
# Fixed by ONYX-2026-09-30-apply-db-ordering-fix-v1.
#
# Usage: DATABASE_URL=postgresql://user:pass@host:port/db scripts/apply-db.sh
# Needs: psql on PATH. Exit nonzero on first SQL error.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

: "${DATABASE_URL:=postgresql://seatloom:seatloom@localhost:5432/seatloom}"
SCHEMA_DIR="infra/postgres/schema"
SEED_DIR="infra/postgres/seed"

psql_f() { psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -f "$1"; }

# --- Ordering lists: "<filename>|<why it sits in this group>" -----------------

# PRE-SEED: the seed writes columns these create.
PRE_SEED_SCHEMA=(
  "001_seatloom_core.sql|creates projects/seats/sessions/workitems/handoffs/artifacts/canonical_events — every table the seed inserts into"
  "002_document_authority.sql|creates documents/document_sections; seed 002 inserts document rows"
  "003_write_ingest_reconcile.sql|creates reconcile_runs/reconcile_items/document_versions written by reconcile and seed 002"
  "004_operational_review_and_continuity.sql|creates checkpoints/review_threads/review_comments; seed 003 inserts into them"
  "005_prompt_and_channel_action_authority.sql|creates prompt_instances/prompt_actions/channel_action_receipts; seed 004 targets them (zero-row baseline)"
  "009_runtime_triple.sql|ADDS seats.default_harness/_agent/_model and sessions.harness/agent/model, which seed 001 WRITES — must exist before the seed runs. Safe here: it only touches columns created in 001 and constrains nothing the seed omits (its sole SET NOT NULL is on sessions.agent, which the seed supplies)."
)

# POST-SEED: these constrain columns the seed does not provide.
POST_SEED_SCHEMA=(
  # Order within this group is significant: 008 BEFORE 010.
  # 010's G5 guard RAISES if sessions.project_id is missing or still nullable,
  # and 008:51 / 008:106-107 is what creates and tightens it. Reversing these
  # two makes 010 abort by design rather than silently proceed.
  "008_project_isolation.sql|backfills project_id on workitems/sessions/handoffs/canonical_events from seeded rows, then SET NOT NULL. Seed 001 supplies no project_id, so running this before the seed makes the seed fail: 'null value in column \"project_id\" violates not-null constraint'."
  "010_p0_schema_foundation.sql|adds canonical_events.schema_version NOT NULL and event_object_refs.role NOT NULL, then backfills both from existing rows. Seed 001 inserts into BOTH tables and supplies NEITHER column, so pre-seed would break the seed exactly as 008 would have. Also widens the event_object_refs primary key to include role and adds a partial unique index scoped so legacy rows cannot violate it — both need the seeded rows present."
  "021_history_sessions_corpus.sql|creates archived_sessions + v_archived_sessions_normalized and inserts the corpus inventory rows. The seed writes none of these tables/columns, so by the same rule as 011/012 it is POST-SEED. WAS UNCLASSIFIED until 2026-10-26 — Flux's audit of the three blocked gates (disp-VH6l1HixlUVYuy4m0NEp-) found this was the ONE unclassified file of 20, and apply-db.sh's completeness guard therefore refused to build ANY database. Already applied to live (measured: 183 corpus / 89 inventory-only rows), so this entry records an existing fact rather than applying anything. Registered by AEGIS-2026-10-26-fix-apply-db-unclassified-v1."
  # 022 NOT REGISTERED YET: it lives on packet/nimbus/project-scoped-reads and is
  # not on track, so listing it here would fail the guard the other way ("listed
  # but absent"). Uncomment the line below on the day the branch merges.
  # "022_session_work_bridge_view.sql|CREATE VIEW v_memory_session_join, read-only, ZERO row inserts — same rule as 015's view, so POST-SEED. Registered by disp-XnCxvGI88oAbdGcyeefhl."
  "011_agent_history_archive.sql|creates archived_sessions (the agent-history index) — a brand-new table the seed never writes to, with NO FK to seats/projects and no DDL against any seeded table. The PRE-SEED rule is 'the seed writes columns this migration creates'; the seed writes none of archived_sessions' columns, so by rule it is POST-SEED (the 'otherwise' bucket). Placement relative to the seed is in fact immaterial here — archived_sessions and the seed share nothing — so the default non-pre-seed classification is the honest one, and it matches the migration's own header ('Additive, post-seed, idempotent'). Registered by ONYX-2026-10-08-p0-step6-dispatch-schema-v1 §0 (was on disk but in neither list, so the completeness guard refused to build any database)."
  "012_governed_dispatch.sql|widens two existing vocab CHECKs (canonical_events_event_type_vocab, event_object_refs_ref_type_vocab) and creates dispatches/dispatch_dependencies — brand-new tables the seed never writes. The seed writes none of these new columns, so by the same rule as 011 it is POST-SEED. Widening the two vocab CHECKs must also run POST-SEED: those CHECKs exist only after 010 (POST-SEED) has created them, so 012 cannot precede its own prerequisite. No seeded row references any value 012 adds, so post-seed placement cannot break the seed. Registered by ONYX-2026-10-08-p0-step6-dispatch-schema-v1."
  "013_dispatch_states.sql|widens dispatches_status_vocab 4->6 (+failed,+cancelled) and canonical_events_event_type_vocab 15->18 (+SeatDispatchFailed/Retried/Cancelled), adds dispatches.retry_count/max_retries, and creates the fn_dispatch_recompute_ready function plus three AFTER-INSERT triggers that persist and notify the blocked->ready flip. Brand-new columns on a table the seed never writes, plus widenings of CHECKs 010/012 created (which exist only post-seed), so by the same rule as 011/012 it is POST-SEED and cannot precede its prerequisites. Registered by disp-Rlth465QCOr9L5HK_CIu5 (D2, 013 landing)."
  "014_acceptance_gate.sql|creates acceptances (the D2 acceptance-gate object) — a brand-new table the seed never writes, FK to dispatches(id) which 012 creates, so 014 must follow 012 — and widens canonical_events_event_type_vocab 18->21 (+AcceptanceRequested/Granted/Rejected) and event_object_refs_ref_type_vocab 10->11 (+acceptance). Same POST-SEED rule as 011/012/013: the seed writes none of these columns and the widened CHECKs exist only post-seed. Canonical order 012 -> 013 -> 014 (014's event_type DROP/ADD installs the full 21-value list, so it assumes 013 already ran). Registered by disp-Rlth465QCOr9L5HK_CIu5 (D2, 014 landing)."
  "015_knowledge_layer.sql|adds projects.parent_project_id (self-FK + not-self CHECK + partial index), creates the memories table (L0/L1/L2 progressive disclosure + cold->hot lifecycle, three domain CHECKs, three indexes incl. the 002-style FTS gin), creates the v_archived_sessions_normalized view (executable R1-R5 slug rule, ZERO row inserts), and widens canonical_events_event_type_vocab and event_object_refs_ref_type_vocab by writing the COMPLETE lists (+4 memory lifecycle event types = 25; +memory ref_type = 12) so it applies cleanly from either the track state (15/10) or the live state (21/11). Same POST-SEED rule as 011/012/013/014: the seed writes none of these columns/tables, the widened CHECKs exist only post-seed, and memories REFERENCES topics(id) (010, POST-SEED) and documents(id) (002), so 015 cannot precede its prerequisites. Canonical order 010 -> 015. Registered by disp-d5-lane-a-onyx (D5 knowledge-layer Lane A, 015 landing)."
  "016_acceptance_plan.sql|adds acceptances.acceptance_plan JSONB NOT NULL DEFAULT '{}' (the structured acceptance plan, shape v1) plus the acceptances_plan_required CHECK (acceptance_plan <> '{}'::jsonb) added NOT VALID. Same POST-SEED rule as 011/012/013/014: the seed writes none of acceptances' rows, and the table itself exists only after 014 (POST-SEED) creates it, so 016 cannot precede its prerequisite. Canonical order 014 -> 016 (015 is the parallel D5 knowledge-layer migration on the same seat — serialized, never co-edited). NOT VALID is deliberate: the one legacy pending row is exempt until Aegis backfills Lyra's authored plan and VALIDATEs (live-DB act, Aegis lane). Registered by disp-accplan-lane-c-onyx (acceptance-lineage Lane C, 016 landing)."
  "017_acceptance_cancel.sql|widens acceptances_status_vocab 3->4 (+cancelled — the requesting seat's withdrawal of its own pending request, terminal like accepted/rejected) and canonical_events_event_type_vocab to the COMPLETE 26-value list (+AcceptanceCancelled; the 22 acceptance/dispatch values + the 4 memory values from 015), writing full lists in the 013/015 guarded pattern so it applies cleanly from either the track state (3/21) or the live state (3/25 — 015/016 applied out-of-band). Same POST-SEED rule as 011/012/013/014: both widened CHECKs exist only post-seed and the seed writes no acceptances rows. Closes the surfaced gap behind Nimbus's shipped cancel_acceptance tool (Aegis dispatch disp-E8BP2vKlOwMHLomm0eX8J drafted the DDL in a .scratch copy, NOT FOR COMMIT); the Rust mirror EventType::ALL: [EventType; 26] on packet/nimbus/d5-mcp-recall is the pinned enum this list matches verbatim. Canonical order 014 -> ... -> 017 (015/016 are the parallel same-seat lanes on their own branches). Registered by disp-acclose-onyx (acceptance-cards closeout, 017 landing)."
  "018_memory_hardening.sql|hardens 015's memories table against the five measured gaps (D5 Phase 2 Lane A1): +kind (6-value record-type vocab, additive DEFAULT 'fact' is backfill-only — the remember tool requires kind explicitly), +supersedes (self-FK, documents.supersedes style) and the new memory_relations table (from_id/to_id FKs + rel 4-value vocab + PK — two spellings of one edge, write-path-consistent in one transaction), +resolved_by/resolved_at with the status vocab widened 2->3 (+resolved) and the boolean-equality pairing CHECK (resolved IFF resolver set, enforced both directions), +confidence (quoted|inferred, BINARY never a score, additive DEFAULT 'inferred' backfill), +l2_line_start/l2_line_end with the load-bearing CHECK (file + archived_session L2 pointers MUST carry a well-formed line range — pruning may shrink the LLM input, never the pointer), and canonical_events_event_type_vocab widened to the COMPLETE 27-value list (+MemoryResolved; order-independent vs 017 by construction — the full list preserves AcceptanceCancelled whichever order the two land in). Same POST-SEED rule as 015: the seed writes none of these columns, memories exists only after 015 (POST-SEED) and memory_relations FKs it, and the widened CHECKs exist only post-seed, so 018 cannot precede its prerequisite. Canonical order 015 -> 018. Zero data rows inserted or rewritten (additive DEFAULTs backfill only; the live apply is Aegis's act). Registered by disp-mem018-onyx (D5 Phase 2 Lane A1, 018 landing — the hard prerequisite for the washing pipeline's S3 extraction)."
  "019_topic_opened.sql|widens canonical_events_event_type_vocab 27->28 (+TopicOpened) by DROP+ADD of the COMPLETE 28-value list in 013's guarded pattern (guard checks for 'TopicOpened'), so it applies cleanly from the track state (27, post-018) or any intermediate and no pre-019 value is silently reverted (a stale first draft in Nimbus's scratch would have reverted 015/018; the shipped list is cross-checked value-by-value against the pinned Rust EventType::ALL on packet/nimbus/open-topic). Same POST-SEED rule as 010/012/013/014/017/015/018: it widens ONE vocab CHECK those migrations created, which exists only post-seed, and the seed writes no canonical_events rows. No ref_type widening ('topic' exists since 010), no new columns/tables, zero data rows. Closes the live block on Nimbus's open_topic tool (Aegis ruling 1, AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md): until 019 is landed AND live-applied, open_topic refuses at the CHECK — the overnight chain (Vela's write-in, the headline card) sits behind this. Canonical order 018 -> 019. Registered by disp-019-onyx (TopicOpened, 019 landing)."
  "020_operator_decisions.sql|creates operator_decisions — the structured 'waiting on the human' object, closing the measured gap where a product-direction ruling or an operator-only precondition existed ONLY as a SupervisorMessage and nothing could answer 'how many things are waiting on him' (fleet-status.sh read acceptances alone) — and widens canonical_events_event_type_vocab 28->30 (+OperatorDecisionRequested/Resolved) and event_object_refs_ref_type_vocab 12->13 (+decision), both by DROP+ADD of the COMPLETE lists in 013's guarded pattern (guards check for the new value; no pre-020 value contains it as a substring), so it applies cleanly from the track state (28/12, post-019) or any intermediate. Same POST-SEED rule as 010/012/013/014/017/015/018/019: both widened CHECKs exist only post-seed, and the seed writes none of operator_decisions' columns. NO SEED DATA by design — a pending '等你决定' nobody created is contamination, not a feature. resolved is terminal and immutable (operator_decisions_resolved_pairing, 018's IFF pattern); only human:* and seat:seat-aegis-001 may create (operator_decisions_creator_shape). Canonical order 019 -> 020. Registered by disp-dwSmYkULRU4HdSDJCDWH9 (AEGIS-2026-10-22-operator-decision-object-v1, 020 landing)."
  "022_session_work_bridge_view.sql|creates v_memory_session_join — the ONE read-only view that makes the session<->work bridge measurable (contract LYRA-2026-10-16-session-work-bridge-contract-v1 §11.2 slice 1). A memory's source_ref is recorded provenance with NO FK by design, so the join can break in silence and already has: before 021's re-key, 3 codex references resolved against nothing and nothing reported it. The view joins on the contract's PRIMARY key A1 (l2_file_path with the corpus-root prefix stripped == archived_sessions.raw_path, both sides DECLARED and immune to id re-keying) AND its SECONDARY key A2 (source_ref split on the first colon == (harness, id)), reporting both verdicts plus a resolved_by column, so a disagreement between them stays countable instead of collapsing into one number. It writes NO data (a view has no rows), so classification follows what it READS: memories and archived_sessions, which no seed file writes (measured: zero INSERT INTO memories in any seed; the seed's only sessions writes are 5 rows this view does not touch). Same POST-SEED rule as 015's v_archived_sessions_normalized. Catalog-guarded creation (015's pattern) so a re-apply writes zero catalog rows and a later definition change must be a follow-up migration's DROP+CREATE rather than a silent rewrite. Canonical order 021 -> 022. Registered by disp-XnCxvGI88oAbdGcyeefhl (AEGIS-2026-10-26-two-gates-blocked-v1 task A) — the gate ev-session-key-readmodel-live measured this view ABSENT and three dispatches have been blocked on it since 10-16."
  "023_gate_criterion.sql|RENUMBERED from 020 on the 2026-10-28 merge of track/infra-foundation: both branches had independently landed a migration numbered 020 (track's 020_operator_decisions.sql and this one), and only one file can carry a number, so this one takes the next free slot rather than colliding. The renumber changed ONLY the filename and this migration's self-references — no SQL object was renamed and no dependency changed. WHAT IT DOES: replaces the BODY of fn_dispatch_recompute_ready (013) so a GATE-SHAPED event dep must be a MEASURED gate rather than merely an existing row (the release-time half of Aegis ruling 2026-10-11; criterion shape is Nimbus's gate.rs on packet/nimbus/gate-vs-announcement), adds the two IMMUTABLE criterion helpers (fn_gate_is_gate_id / fn_gate_row_is_gate) that the new body, the new dispatch_dependencies CHECK and the new v_dispatch_gate_held view all share so the criterion exists in ONE place per language, and adds a CHECK refusing a gate-shaped dep_value on a non-event edge. POST-SEED by the same rule as 011-013: the seed writes no dispatch_dependencies rows and none of these objects touch a seeded table's columns. Uses CREATE OR REPLACE rather than DROP+CREATE because the three 013 triggers are bound to the function — a DROP would take them with it. Zero data rows inserted or rewritten. NOTE FOR THE RUST SIDE: track's crates/seatloom-mcp-emit/src/gate.rs cites this file by its OLD name (020_gate_criterion.sql) in a comment — that citation is now stale and is listed as a known gap in this dispatch's delivery note rather than silently edited. Canonical order 013 -> 023. Registered by disp-onyx-gate-release-check (gate release-time check, 020 landing)."
)

# --- Completeness guard ------------------------------------------------------
# A migration present on disk but absent from both lists would be silently
# skipped — a worse failure than the one this script just fixed, because the
# database would come up quietly incomplete. Fail loudly instead.
classified=""
for entry in ${PRE_SEED_SCHEMA[@]+"${PRE_SEED_SCHEMA[@]}"} ${POST_SEED_SCHEMA[@]+"${POST_SEED_SCHEMA[@]}"}; do
  classified="$classified ${entry%%|*}"
done

unclassified=0
for path in "$SCHEMA_DIR"/*.sql; do
  [ -f "$path" ] || continue
  base="$(basename "$path")"
  case " $classified " in
    *" $base "*) ;;
    *)
      if [ "$unclassified" -eq 0 ]; then
        echo "ERROR: unclassified migration(s) in $SCHEMA_DIR — refusing to apply a partial schema." >&2
        echo "Add each to PRE_SEED_SCHEMA or POST_SEED_SCHEMA in this script, with a reason:" >&2
        echo "  PRE-SEED  if the seed writes columns the migration creates." >&2
        echo "  POST-SEED if the migration constrains columns the seed does not provide." >&2
      fi
      echo "  unclassified: $base" >&2
      unclassified=1
      ;;
  esac
done
[ "$unclassified" -eq 0 ] || exit 2

# --- Apply -------------------------------------------------------------------
apply_group() {
  label="$1"; shift
  echo "--- $label ---"
  for entry in "$@"; do
    file="${entry%%|*}"
    reason="${entry#*|}"
    path="$SCHEMA_DIR/$file"
    if [ -f "$path" ]; then
      echo "  $path"
      echo "      why: $reason"
      psql_f "$path"
    else
      # Listed but absent: expected when this script runs on a branch that does
      # not carry the migration yet. Announced, never silent.
      echo "  SKIP $file — not present on this branch"
    fi
  done
}

echo "apply-db: target=$DATABASE_URL"

apply_group "pre-seed schema (seed writes columns these create)" \
  ${PRE_SEED_SCHEMA[@]+"${PRE_SEED_SCHEMA[@]}"}

echo "--- seed (001-004) ---"
for f in "$SEED_DIR"/*.sql; do
  [ -f "$f" ] && { echo "  $f"; psql_f "$f"; }
done

apply_group "post-seed schema (constrains columns the seed does not provide)" \
  ${POST_SEED_SCHEMA[@]+"${POST_SEED_SCHEMA[@]}"}

echo "apply-db: done."
