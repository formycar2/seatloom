-- 023 — `remember` refuses a repeat write instead of duplicating it.
--
-- THE DEFECT, MEASURED ON LIVE 2026-10-29 (read-only), NOT INFERRED:
--
--   select l2_file_path, l2_line_start, l2_line_end, count(*)
--     from memories where l2_file_path is not null and l2_line_start is not null
--    group by 1,2,3 having count(*) > 1;
--
-- returns 6 groups. Five of them are NOT duplicates — four distinct memories
-- cite ONYX-2026-10-14-history-sessions-data-delivery-v1.md lines 1-326 and
-- share the same source_ref, differing only in what they say, and that is
-- correct behaviour. Aegis's dispatch reported "6 groups with duplicates"; the
-- honest number of duplicate ROWS is smaller, and the distinction decides the
-- key. The Aegis query groups on the pointer alone, so it cannot tell a
-- duplicate from two records that happen to cite the same evidence.
--
-- The real duplicates, grouped on the pointer plus the substance:
--
--   select l2_file_path, l2_line_start, l2_line_end, l0_summary, count(*)
--     from memories group by 1,2,3,4 having count(*) > 1;
--   → 4 groups, 8 rows, of which:
--     · 3 groups carry NO line range at all (l2_line_start IS NULL)
--     · 1 group is Vela's 2026-10-11 idempotency probe, whose only difference
--       was a title suffix
--
-- WHY THE RETRY SHAPE MATTERS. A write that times out and is retried creates a
-- second row with no error and no warning. Every count downstream — recall
-- results, promotion candidates, the drift numbers — then overstates what is
-- known. The probe that found this was run by a seat doing post-insert
-- verification, not by the owner of the write path, which is worth its own note:
-- the guard on this table was whoever happened to check afterwards.
--
-- THE KEY, AND WHY NOT THE OBVIOUS ONE. (l2_file_path, l2_line_start,
-- l2_line_end) is the obvious candidate and it is WRONG: it would refuse the
-- four legitimate ONYX rows, which share a pointer and a source_ref and differ
-- only in substance. The key is the pointer PLUS l0_summary — what the record
-- actually says.
--
-- `title` is deliberately NOT in the key. It changes, and a key that moves with
-- a rename lets the same memory back in — which is exactly how Vela's probe got
-- a second row out of a write whose only difference was a title suffix. The
-- dispatch's own do_not names this: "do not dedup on title text".
--
-- NULL-SAFE, BECAUSE THREE OF THE FOUR GROUPS HAVE NO LINES. `= NULL` matches
-- nothing, so a predicate written with plain equality would miss precisely the
-- groups that need catching. The index uses COALESCE sentinels instead; `0` is
-- safe because line numbers start at 1 (018 §G5 requires the range when the
-- kind needs it, and 0 is not a legal line number).

-- Guard first: this index cannot be created while duplicates exist, and a
-- failure here must NAME the rows rather than print "could not create unique
-- index" and leave the reader guessing. Refusing loudly is the whole point —
-- a migration that half-applies is a migration nobody can trust.
DO $$
DECLARE
  dup RECORD;
  n int := 0;
BEGIN
  FOR dup IN
    SELECT coalesce(l2_file_path, '(null)') AS p,
           l2_line_start AS s,
           l2_line_end   AS e,
           count(*)      AS rows,
           min(id)       AS sample
      FROM memories
     GROUP BY 1,2,3, l0_summary
    HAVING count(*) > 1
     ORDER BY rows DESC
  LOOP
    n := n + 1;
    -- One NOTICE, and the path is NOT interpolated: a `%` inside a RAISE format
    -- string is read as a placeholder and fails with "too few parameters
    -- specified for RAISE". The sample row id is enough to find the group.
    RAISE NOTICE 'duplicate group %: % rows at lines %-%, sample row %',
      n, dup.rows, dup.s, dup.e, dup.sample;
  END LOOP;
  IF n > 0 THEN
    -- No `%` anywhere in this string: a RAISE format string reads `%` as a
    -- placeholder, and "group(s)" made psql count four placeholders while three
    -- arguments were supplied.
    RAISE EXCEPTION
      '023 cannot apply: % memory duplicate groups remain. The application-level guard in insert_memory stops NEW duplicates; repairing the EXISTING rows is a live-database act that needs the operator''s decision, not this migration.',
      n;
  END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS memories_dedup_key
    ON memories (
      coalesce(l2_file_path, ''),
      coalesce(l2_line_start::text, ''),
      coalesce(l2_line_end::text, ''),
      l0_summary
    );

COMMENT ON INDEX memories_dedup_key IS
  'The application-level guard in insert_memory refuses a repeat write; this index is the durable backstop for any writer that does not go through it. Deliberately excludes title: a key that moves with a rename lets the same memory back in.';
