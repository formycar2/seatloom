#!/usr/bin/env bash
# Are the acceptance cards' verdict refs actually verdicts ABOUT THEM?
#
# RULE: LYRA-2026-10-29-verdict-ref-validity-ruling-v1. Three conditions:
#   1. the event exists and is a ReviewVerdictIssued
#   2. the verdict is a PASS  (neither earlier implementation checked this)
#   3. it is about the dispatch, by shape A (payload.dispatch_id) or shape B
#      (a batch verdict whose subject is the verdict document, the dispatch a
#      context ref)
#
# This script previously implemented only shape A and reported BAD for valid
# batch verdicts, which caused Mira to cancel and re-open correct cards on
# 2026-10-28. It then implemented "payload or ANY dispatch ref", which was too
# loose — a context ref means the verdict MENTIONS the dispatch, not that it
# covers it. The ruling names both shapes and states the half no join can check:
# for a shape-B card the verdict DOCUMENT must be opened and read. This script
# therefore prints the document path for every shape-B card rather than passing
# it silently.

set -u

cd "$(dirname "$0")/.." || exit 2

db() {
  podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F'|' -c "$1" 2>/dev/null
}

if [ -z "$(db "select 1 from pg_tables where tablename='acceptances'")" ]; then
  echo "FAIL: no acceptances table - cannot check anything" >&2
  exit 2
fi

echo "== pending acceptance cards: is each verdict ref a verdict about THIS dispatch? =="
echo "   rule: LYRA-2026-10-29-verdict-ref-validity-ruling-v1 (supersedes the 10-27 ruling on this point)"
bad=0
good=0
batch=0
while IFS='|' read -r card dref vref ev_ok is_pass about shape doc; do
  [ -n "$card" ] || continue
  if [ "$ev_ok" = "t" ] && [ "$is_pass" = "t" ] && [ "$about" = "t" ]; then
    good=$((good + 1))
    if [ "$shape" = "B" ]; then
      batch=$((batch + 1))
      printf '  ok    %-46s shape B (batch verdict — OPEN THE DOCUMENT and confirm it covers the dispatch)\n' "$card"
      printf '          verdict document: %s\n' "$doc"
    fi
  else
    bad=$((bad + 1))
    printf '  BAD  %s\n' "$card"
    printf '       dispatch=%s verdict_ref=%s\n' "$dref" "$vref"
    [ "$ev_ok" = "t" ] || printf '       the verdict event does not exist (or is not ReviewVerdictIssued)\n'
    if [ "$ev_ok" = "t" ] && [ "$is_pass" != "t" ]; then
      printf '       the verdict is not a PASS — it is %s. A fail is never evidence for acceptance.\n' \
        "$(db "select coalesce(payload->>'verdict','(nothing)') from canonical_events where id='$vref'")"
    fi
    [ "$about" = "t" ] || printf '       not about THIS dispatch by either shape: payload.dispatch_id=%s, and no event_object_refs row with ref_type=dispatch names it\n' \
      "$(db "select coalesce(payload->>'dispatch_id','(nothing)') from canonical_events where id='$vref'")"
  fi
done <<< "$(db "
  select a.id, a.dispatch_id, a.flux_verdict_ref,
    exists(select 1 from canonical_events e
            where e.id=a.flux_verdict_ref and e.event_type='ReviewVerdictIssued') as event_ok,
    exists(select 1 from canonical_events e
            where e.id=a.flux_verdict_ref and e.payload->>'verdict'='pass') as is_pass,
    exists(
      select 1 from canonical_events e join dispatches d on d.id = e.payload->>'dispatch_id'
            where e.id=a.flux_verdict_ref and d.id = a.dispatch_id
      union all
      select 1 from event_object_refs r
            where r.event_id=a.flux_verdict_ref and r.ref_type='dispatch' and r.ref_id=a.dispatch_id
    ) as about_this_dispatch,
    case when exists(select 1 from canonical_events e join dispatches d on d.id = e.payload->>'dispatch_id'
                     where e.id=a.flux_verdict_ref and d.id = a.dispatch_id)
         then 'A' else 'B' end as shape,
    coalesce((select r.ref_id from event_object_refs r
               where r.event_id=a.flux_verdict_ref and r.role='subject' and r.ref_type='document'
               limit 1), '') as verdict_doc
  from acceptances a where a.status='pending' order by a.id;")"

echo "  good=$good bad=$bad (of which shape B, batch verdicts: $batch)"

echo
echo "== ReviewVerdictIssued events that name a dispatch which does not exist =="
wild=0
while IFS='|' read -r vid named actor; do
  [ -n "$vid" ] || continue
  wild=$((wild + 1))
  if [ -z "$named" ]; then
    printf '  %-28s EMPTY  names nothing (%s)\n' "$vid" "$actor"
  else
    printf '  %-28s WILD   names %s which has 0 rows (%s)\n' "$vid" "$named" "$actor"
  fi
done <<< "$(db "
  select e.id, e.payload->>'dispatch_id', coalesce(e.actor_ref,'?')
  from canonical_events e
  where e.event_type='ReviewVerdictIssued'
    and not exists (select 1 from dispatches d where d.id = e.payload->>'dispatch_id')
  order by e.occurred_at;")"

if [ "$wild" -gt 0 ]; then
  echo
  echo "  $wild verdict(s) point at nothing. They are not evidence for anything."
  echo "  They are not currently referenced by any pending card (checked separately),"
  echo "  but a future dispatch with one of those ids would inherit them."
fi

if [ "$bad" -gt 0 ]; then
  echo
  echo "FAIL: $bad pending card(s) have a verdict ref that is not a verdict about them."
  echo "Do not let the operator click those: the merge criterion is not met."
  exit 1
fi

echo
echo "OK: every pending card's verdict ref is a verdict about that very dispatch."
exit 0
