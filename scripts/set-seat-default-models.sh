#!/usr/bin/env bash
# User ruling: Mira/Nimbus/Lyra -> GPT PTU; every other registered seat -> Step.
# Default is read-only. Only the operator runs --apply against live PostgreSQL.
# Does not restart agents or modify historical session model records.
set -euo pipefail
case "${1:-}" in
  '') APPLY=0 ;;
  --apply) APPLY=1 ;;
  *) echo 'Usage: bash scripts/set-seat-default-models.sh [--apply]' >&2; exit 2 ;;
esac
PG="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
psql_cmd=(podman exec -i "$PG" psql -X -U seatloom -d seatloom -v ON_ERROR_STOP=1)
if [ "$APPLY" = 0 ]; then
  "${psql_cmd[@]}" <<'SQL'
SELECT id, default_model AS current_model,
 CASE WHEN id IN ('seat-mira-001','seat-nimbus-001','seat-lyra-001')
 THEN 'modelproxy/gpt-6.1-sol-sys-ptu'
 ELSE 'modelproxy-anthropic/step-5-preview-sys' END AS requested_model
FROM seats ORDER BY id;
SQL
  echo 'Preview only: no rows changed. Apply: bash scripts/set-seat-default-models.sh --apply'
  exit 0
fi
"${psql_cmd[@]}" <<'SQL'
BEGIN;
DO $$ BEGIN
 IF (SELECT count(*) FROM seats WHERE id IN
 ('seat-mira-001','seat-nimbus-001','seat-lyra-001')) <> 3 THEN
 RAISE EXCEPTION 'One or more GPT-target seats are missing; no changes applied';
 END IF;
END $$;
UPDATE seats SET default_model = CASE
 WHEN id IN ('seat-mira-001','seat-nimbus-001','seat-lyra-001')
 THEN 'modelproxy/gpt-6.1-sol-sys-ptu'
 ELSE 'modelproxy-anthropic/step-5-preview-sys' END;
DO $$ BEGIN
 IF EXISTS (SELECT 1 FROM seats WHERE default_model IS DISTINCT FROM CASE
 WHEN id IN ('seat-mira-001','seat-nimbus-001','seat-lyra-001')
 THEN 'modelproxy/gpt-6.1-sol-sys-ptu'
 ELSE 'modelproxy-anthropic/step-5-preview-sys' END) THEN
 RAISE EXCEPTION 'Model assignment verification failed';
 END IF;
END $$;
COMMIT;
SELECT id, default_model FROM seats ORDER BY id;
SQL
printf '\nDefaults updated. Running sessions have NOT been switched or restarted.\n'
