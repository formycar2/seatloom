#!/usr/bin/env bash
# Are the acceptance cards' verdict refs actually verdicts ABOUT THEM?
#
# The 10-27 ruling made flux_verdict_ref a hard condition: it must name an
# existing ReviewVerdictIssued whose payload.dispatch_id names the very dispatch
# being accepted.
#
# Checking "the event exists" is not enough. On 2026-10-28 I found a verdict
# whose payload.dispatch_id was `disp-B4Y5oAhMQwtXm5Vva1fw` - zero rows in
# dispatches. The real id is `disp-B4Y5oAohMQwtXm5Vva1fw`: letter o vs digit 0,
# one character, in a nanoid alphabet that contains both. It looked exactly like
# a green third-party verdict and pointed at nothing.
#
# An empty pointer errors. A wild pointer does not - and if a dispatch ever
# appears with that id, the verdict silently becomes evidence for unrelated work.
#
# This script joins both halves and prints what it finds. Run it every heartbeat.

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
bad=0
good=0
while IFS='|' read -r card dref vref ev_ok names; do
  [ -n "$card" ] || continue
  if [ "$ev_ok" = "t" ] && [ "$names" = "t" ]; then
    good=$((good + 1))
  else
    bad=$((bad + 1))
    printf '  BAD  %s\n' "$card"
    printf '       dispatch=%s verdict_ref=%s\n' "$dref" "$vref"
    [ "$ev_ok" = "t" ] || printf '       the verdict event does not exist (or is not ReviewVerdictIssued)\n'
    [ "$names" = "t" ] || printf '       the verdict does not name THIS dispatch - it names %s\n' \
      "$(db "select coalesce(payload->>'dispatch_id','(nothing)') from canonical_events where id='$vref'")"
  fi
done <<< "$(db "
  select a.id, a.dispatch_id, a.flux_verdict_ref,
    exists(select 1 from canonical_events e
            where e.id=a.flux_verdict_ref and e.event_type='ReviewVerdictIssued') as event_ok,
    exists(select 1 from canonical_events e join dispatches d on d.id = e.payload->>'dispatch_id'
            where e.id=a.flux_verdict_ref and d.id = a.dispatch_id) as names_this_dispatch
  from acceptances a where a.status='pending' order by a.id;")"

echo "  good=$good bad=$bad"

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
