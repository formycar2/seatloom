#!/usr/bin/env bash
# Follow-up attacks: A9 done properly (the FK from dispatch_dependencies blocked
# the delete), and A10 (the row-sourced acceptance fact's actor).
set -uo pipefail
DB=seatloom_lineage_ref
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus"
BIN="$ROOT/target/debug/seatloom-mcp-emit"
PROJECT=lineage-ref-proof
NIMBUS=seat-nimbus-001
CTX="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/resume-contexts"
psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1" 2>&1; }
drive() { local a="$1"; shift; printf '%s\n' "$@" | DATABASE_URL="$DBURL" SEATLOOM_ACTOR_REF="$a" \
    SEATLOOM_PROJECT_ID="$PROJECT" SEATLOOM_INTEGRATION_BRANCH="track/infra-foundation" \
    SEATLOOM_HISTORY_CONTEXTS_DIR="$CTX" "$BIN" 2>/dev/null; }
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
call() { echo "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}"; }
text() { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert not r["isError"], r["content"][0]["text"]; print(r["content"][0]["text"])'; }
err()  { tail -1 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]; assert r["isError"], "expected a refusal, got: "+r["content"][0]["text"]; print(r["content"][0]["text"])'; }

echo "########## A9 (retry) — the acceptance whose gated dispatch is gone ##########"
psql "ALTER TABLE acceptances DROP CONSTRAINT IF EXISTS acceptances_dispatch_id_fkey;" >/dev/null
echo "  FKs that would block the delete: $(psql "SELECT conname||' on '||conrelid::regclass FROM pg_constraint WHERE confrelid='dispatches'::regclass;")"
psql "DELETE FROM dispatch_dependencies WHERE dispatch_id='disp-lref-proof';" >/dev/null
psql "DELETE FROM dispatches WHERE id='disp-lref-proof';" >/dev/null
echo "  dispatches rows named disp-lref-proof now: $(psql "SELECT count(*) FROM dispatches WHERE id='disp-lref-proof';")"
echo "  acceptances rows named acc-disp-lref-proof now: $(psql "SELECT count(*) FROM acceptances WHERE id='acc-disp-lref-proof';")"
echo "  input : {\"ref\":{\"kind\":\"acceptance\",\"id\":\"acc-disp-lref-proof\"}}"
echo "  output: $(drive "$NIMBUS" "$INIT" "$INITED" "$(call lineage '{"ref":{"kind":"acceptance","id":"acc-disp-lref-proof"}}')" | err)"
echo "  FINDING: the card EXISTS. get_acceptance() INNER JOINs dispatches, so the"
echo "           'this card gates a dispatch that is gone' note is unreachable and the"
echo "           answer instead claims the acceptance itself does not exist."

echo
echo "########## A10 — who opened the acceptance card, when the ledger says nothing ##########"
echo "  the acceptances table has no requested_by column:"
echo "    $(psql "SELECT string_agg(column_name,', ' ORDER BY ordinal_position) FROM information_schema.columns WHERE table_name='acceptances';")"
echo "  input : {\"ref\":{\"kind\":\"acceptance\",\"id\":\"acc-flux-dangling-verdict\"}}"
drive "$NIMBUS" "$INIT" "$INITED" "$(call lineage '{"ref":{"kind":"acceptance","id":"acc-flux-dangling-verdict"}}')" \
  | text | python3 -c '
import sys, json
d = json.load(sys.stdin)
for f in d["facts"]:
    print("    fact.eventType =", f["eventType"], "| occurredAt =", f["occurredAt"], "| actorRef =", f["actorRef"])
    print("    fact.statement =", f["statement"])
    print("    fact.evidence  =", f["evidence"]["kind"], f["evidence"]["id"])
'
echo "  the card was inserted by this attack with NO AcceptanceRequested event, so"
echo "  nothing records who opened it. The row-sourced fact names 监督者."
echo "  FINDING: an actor is defaulted where the schema records none."
