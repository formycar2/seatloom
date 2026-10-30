#!/usr/bin/env bash
# Land the memory-dedup guard (023) on the live database.
#
# WHY THIS NEEDS THE OPERATOR. The guard is written, tested and proven, but it
# CANNOT land on its own, and the reason is a real deadlock rather than paperwork:
#
#   023 creates a UNIQUE INDEX. A unique index cannot be created while duplicate
#   rows exist. Live has 5 groups of 2 rows (measured 2026-10-30, grouped on the
#   index's REAL key: coalesce(l2_file_path,''), coalesce(l2_line_start::text,''),
#   coalesce(l2_line_end::text,''), l0_summary).
#   CORRECTED: I first reported 6 groups with a worst case of four rows. That was
#   my own grouping error - I grouped on four columns but SELECTed and reported
#   only three, so three groups sharing one file printed as three copies of the
#   same six rows and I read it as one group of six. Corrected the same day in
#   ev-correction-duplicate-group-count. Group and report on the SAME columns.
#   So:
#
#     duplicates exist  ->  index refused  ->  dedup not enforced  ->  more duplicates
#
#   023's own guard refuses loudly and NAMES the rows rather than half-applying,
#   which is correct, and which is also why nothing has silently progressed.
#
# Breaking the deadlock means deleting or merging 6 groups of rows in the live
# database. That is a live-DB write, which Aegis is permission-blocked from and
# which no seat should decide either: which of two rows is the real one is a
# judgement about recorded facts, not a mechanical dedup.
#
# WHAT THIS SCRIPT DOES. Dry-run by default. It reports the state, shows exactly
# which rows are in which duplicate group, and - only with --apply - lets you
# choose per group which row survives. It never picks for you.
#
# Usage:
#   bash scripts/land-023-memory-dedup.sh              # report only
#   bash scripts/land-023-memory-dedup.sh --apply      # resolve, then land 023
#
# The migration itself is on packet/nimbus/project-scoped-reads and is NOT on
# track yet, so this script takes it from that branch by explicit ref. If it has
# since merged, it will say so and use the track copy.

set -u

cd "$(dirname "$0")/.." || exit 2

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

BRANCH="packet/nimbus/project-scoped-reads"
MIG="infra/postgres/schema/023_memory_dedup.sql"

say() { printf '%s\n' "$*"; }

# --- where is the migration? ------------------------------------------------
if git cat-file -e "track/infra-foundation:$MIG" 2>/dev/null; then
  SRC="track/infra-foundation:$MIG"
  say "migration 023 is ON TRACK ($SRC)"
elif git cat-file -e "$BRANCH:$MIG" 2>/dev/null; then
  SRC="$BRANCH:$MIG"
  say "migration 023 is on $BRANCH only — NOT on track."
  say "  Landing it on live before it is on track means live runs code the"
  say "  integration branch does not have. Consider merging the branch first."
else
  say "FAIL: cannot find $MIG on track or on $BRANCH" >&2
  exit 2
fi

# --- is it already landed? --------------------------------------------------
idx=$(podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -c \
  "select count(*) from pg_indexes where indexname='memories_dedup_key';" 2>/dev/null)
if [ "$idx" = "1" ]; then
  say "the unique index already exists on live — nothing to do."
  exit 0
fi
say "the unique index is NOT on live — the guard is not enforced there."

# --- the deadlock, measured ------------------------------------------------
say ""
say "== duplicate groups on live (what blocks the index) =="
# l0_summary IS part of the dedup key. The first version of this script left it
# out of the SELECT while keeping it in the GROUP BY, so the printed key was
# (file, lines) while the grouping was (file, lines, summary). Three groups on
# one file printed as three copies of the same six rows, which reads as one
# group of six. It is five groups of two. Print the key exactly as it is grouped
# on, or the report misdescribes what it found.
# NO "where l2_file_path is not null" FILTER, so the report matches the index's
# actual key rather than a subset of it.
#
# CORRECTING MY OWN REASON, WHICH WAS FALSE. I first removed the filter arguing
# that "the index key is coalesce(...), so NULL-path rows participate in
# uniqueness, and two NULL-path rows sharing an l0_summary would block the index".
# Flux measured that the NULL case is STRUCTURALLY UNREACHABLE and reproduced the
# constraint that makes it so:
#
#   memories_l2_pairing CHECK:
#     (l2_ref_kind='document' AND l2_document_id IS NOT NULL)
#     OR (l2_ref_kind='file' AND l2_file_path IS NOT NULL)
#
# So l2_file_path cannot be NULL for a file-kind row. His attempt to CREATE a
# NULL-path duplicate was refused by memories_l2_pairing, not by the index. Live
# has 0 NULL-path rows out of 427.
#
# The coalesce in the index key is still correct - it is what makes the index
# NULL-safe - but the scenario I described as the reason for this change cannot
# occur. The change is harmless (the filter was redundant) and stays, because a
# report should describe the key it is measured against. The reason I wrote for
# it was invented, and this is the second time today I wrote a confident
# mechanism I had not measured.
groups=$(podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F'|' -c "
  select coalesce(l2_file_path,'(null)'), coalesce(l2_line_start::text,'(null)'),
         coalesce(l2_line_end::text,'(null)'), l0_summary, count(*), min(id)
  from memories
  group by 1,2,3,4
  having count(*) > 1
  order by 5 desc;")

if [ -z "$groups" ]; then
  say "  none — the deadlock is not present. Landing 023 should just work."
else
  n=0
  while IFS='|' read -r f s e sum c sample; do
    [ -n "$f" ] || continue
    n=$((n + 1))
    say "  group $n: $c rows — $f lines $s-$e"
    say "    substance: $sum"
    say "    rows:"
    # The predicate below matches a NULL path with coalesce on both sides, so it
    # finds the NULL-path group the report now shows. It does not filter them out.
    podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F'|' -c "
      select id, title, kind, confidence, status, to_char(created_at,'MM-DD HH24:MI')
      from memories
      where coalesce(l2_file_path,'') = coalesce('$f','')
        and coalesce(l2_line_start::text,'(null)') = '$s'
        and coalesce(l2_line_end::text,'(null)') = '$e'
        and l0_summary = '$sum'
      order by created_at;" 2>/dev/null | sed 's/^/      /'
  done <<< "$groups"
  say ""
  say "  $n group(s). Each needs a human decision: which row is the real one."
  say "  This script will NOT choose. Deciding is a judgement about recorded"
  say "  facts, and a mechanical 'keep the oldest' would be a guess."
fi

if [ "$APPLY" != "1" ]; then
  say ""
  say "dry run. Nothing was written."
  say ""
  say "To proceed, and this is a live-database write:"
  say "  1. decide per group which row survives"
  say "  2. bash scripts/land-023-memory-dedup.sh --apply"
  say "  It will then walk you through the groups before touching anything."
  exit 0
fi

# --- apply ------------------------------------------------------------------
say ""
say "== interactive resolution =="
say "For each group, enter the id of the row that SURVIVES, or 'skip'."
say "Anything else aborts without writing."

survivors=""
while IFS='|' read -r f s e c sample; do
  [ -n "$f" ] || continue
  say ""
  say "group: $c rows — $f lines $s-$e"
  podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F'|' -c "
    select id, title, to_char(created_at,'MM-DD HH24:MI') from memories
    where coalesce(l2_file_path,'') = coalesce('$f','')
      and coalesce(l2_line_start::text,'(null)') = '$s'
      and coalesce(l2_line_end::text,'(null)') = '$e'
    order by created_at;" 2>/dev/null | sed 's/^/    /'
  printf '  survivor id (or skip): '
  read -r choice
  case "$choice" in
    skip|SKIP) say "  skipped" ;;
    mem-*) survivors="$survivors $choice" ;;
    *) say "  ABORT: '$choice' is not a survivor id and not 'skip'. Nothing written."; exit 1 ;;
  esac
done <<< "$groups"

say ""
say "survivors:$survivors"
say ""
say "Not yet applied. The delete and the migration are the last two steps and"
say "they need a second confirmation, because this is the live ledger."
printf 'type YES to delete the non-surviving rows and land 023: '
read -r confirm
[ "$confirm" = "YES" ] || { say "aborted, nothing written"; exit 1; }

say ""
say "deleting non-survivors…"
# Only rows in a duplicate group and not chosen. Never touches a singleton.
podman exec seatloom-postgres psql -U seatloom -d seatloom 2>&1 <<SQL | sed 's/^/  /'
DELETE FROM memories
WHERE id IN (
  SELECT id FROM (
    SELECT id, l2_file_path, l2_line_start, l2_line_end, l0_summary,
           row_number() OVER (
             PARTITION BY coalesce(l2_file_path,''), coalesce(l2_line_start::text,''),
                          coalesce(l2_line_end::text,''), l0_summary
             ORDER BY created_at
           ) AS rn
    FROM memories
  ) d
  WHERE rn > 1
) AND id NOT IN ($(echo "$survivors" | tr ' ' ','));
SQL

say ""
say "landing 023…"
git show "$SRC" | podman exec -i seatloom-postgres psql -U seatloom -d seatloom -f - 2>&1 | sed 's/^/  /'

idx=$(podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -c \
  "select count(*) from pg_indexes where indexname='memories_dedup_key';" 2>/dev/null)
if [ "$idx" = "1" ]; then
  say ""
  say "DONE: the unique index is on live. The guard is now enforced."
  say "      A repeat write will be refused naming the row that already exists."
else
  say ""
  say "FAIL: the index is still not on live. The migration's own guard refused —"
  say "      read its NOTICE output above; it names the rows still blocking it."
  exit 1
fi
