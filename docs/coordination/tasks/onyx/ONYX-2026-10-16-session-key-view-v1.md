# ONYX-2026-10-16-session-key-view-v1 — delivery

| Field | Value |
|---|---|
| dispatch | `disp-session-key-view` |
| seat | onyx |
| slice | session↔work bridge, slice 1 (contract `LYRA-2026-10-16-session-work-bridge-contract-v1.md` §11.2 slice 1) |
| event emitted | `ev-session-key-view-landed` (`ArtifactCreated`, announcement — no `verified` field) |
| live database touched | **none** — live was READ only. All writes were on scratch databases, dropped after |
| gate event emitted | **none** (the contract's §5 red line 5 forbids it) |

## 1. What changed

One new file, one read-only view, and two script registrations. No existing column,
constraint, index or FK on `memories` or `archived_sessions` changed — proved by schema
diff in §4.3, not asserted.

| File | Change |
|---|---|
| `infra/postgres/schema/022_session_work_bridge_view.sql` | **new** — creates `v_memory_session_join`, ONE read-only view |
| `scripts/apply-db.sh` | 022 registered in `POST_SEED_SCHEMA` with its reason (the completeness guard refuses to build any database otherwise) |
| `scripts/apply-schema-to-live.sh` | 022 added to the apply loop + three verification lines, so 张先生 can land it on live later without guessing |

Zero rows inserted, updated or deleted. No FK added to `source_ref` — `remember`'s own
contract says of that column *"No FK — recorded, never guessed"*, and the contract's §5
forbids adding an FK to make it enforce something it was never designed to enforce.

## 2. The view — what it answers

`v_memory_session_join` has one row per memory carrying a session reference
(`source_kind = 'archived_session'`), with **both** Tier A keys resolved:

- **A1 (primary)** — `l2_file_path` with the corpus root stripped, against
  `archived_sessions.raw_path`. Both sides *declare* the path; neither derives it. Immune
  to id re-keying, which is why it is primary.
- **A2 (secondary)** — `source_ref` split on the **first** colon, against
  `archived_sessions (harness, id)`. That is the PRIMARY KEY, so this join is at most one
  row by construction.

**The load-bearing number** (contract §11.2 slice 1):

```sql
SELECT count(*) AS memories_with_unresolvable_session_ref
  FROM v_memory_session_join
 WHERE NOT a2_resolves;
```

`join_state` is the exhaustive five-value answer: `both_agree` | `path_only` | `ref_only` |
`disagree` | `none`. The contract's §7.2 needs three states distinguishable and they are:
`both_agree` = records exist and resolve; `path_only`/`ref_only`/`disagree`/`none` = records
exist but do not resolve; **zero rows for a session** = no distilled records at all, which
is a session-side state, not a `join_state` — that is what makes it distinguishable from
"some, but broken".

## 3. What the number is today — and it is not zero

Measured read-only against live on 2026-10-16, then reproduced on scratch:

| | count |
|---|---|
| memories with a session reference | 90 |
| A1 (path pair) resolves | 90 |
| **A2 (session reference) resolves** | **87** |
| **A2 resolves against NO row** | **3** |
| neither key resolves | 0 |
| A1 and A2 resolve to different rows | 0 |

The 3 are `mem-kyloXn9AY3D_YHA6WQk7h`, `mem-Gih4q6JxjyyJmQJCHJmO8`,
`mem-xqtBZmhQd1Iw5t732-tnk` — all `source_ref = 'codex:019dda18-07f6-79e0-8008-e4eb16a5448d'`.
Their `l2_file_path` **does** resolve (A1), to the `archived_sessions` row whose id is the
raw **file name** `rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d`.

That row is one of the 60 codex rows 021 did **not** re-key, because it is one of the 89
inventory-only rows outside the 183-session corpus (`message_count IS NULL` — measured: all
150 corpus codex rows carry native-uuid ids, all 60 non-corpus codex rows still carry
`rollout-<ts>-<uuid>` file-name ids). So the memory's recorded reference names an id that
row has never carried, while its path pointer names the row correctly.

**This is the silent break the contract's number exists to catch, and it is live right now.**
It is the same shape as the one 021's header describes ("3 of which resolve against NO row
today because the live ids are file names") — 021 fixed the 6 references it knew about, and
3 more were hiding behind a path that still resolved. The view reports it as
`join_state = 'path_only'`. It does **not** repair it: the contract's §4.2 rule 4 is
forget + remember, never an in-place UPDATE of `source_ref`, and no row is rewritten here.
Reporting is this file's whole job.

## 4. Verification — everything measured on scratch databases

Three scratch databases were built in `apply-db.sh`'s exact order, then dropped. The live
database was READ only throughout.

### 4.1 Live-shaped scratch (001→021, then the live `topics`/`archived_sessions`/`memories`
rows copied in — 29 / 272 / 157)

```
memories_with_unresolvable_session_ref ......... 3      <- the load-bearing number
join_state .................................. both_agree 87 | path_only 3
view rows ................................... 90
A1 resolves / A2 resolves .................... 90 / 87
A1 ambiguous (path declared by >1 row) ....... 0
```

### 4.2 Every `join_state` proved reachable, with synthetic rows (scratch only, deleted after)

| fixture | `join_state` | what it proves |
|---|---|---|
| neither key resolves | `none` | the orphaned-memory state exists |
| A2 resolves, A1 does not | `ref_only` | the other half of the break |
| both resolve, **different rows** | `disagree` | the two recorded keys contradicting each other is visible, not collapsed (0 on live today; the view must not be able to hide it) |
| path points at the ambiguous cursor directory | `none` + `a1_path_rows = 2`, `a1_id` NULL | the ambiguity guard |
| `source_ref` with no colon | `path_only`, `ref_id` NULL | a malformed reference is measured unresolved, not half-resolved |
| a `canonical_event` memory | **absent from the view** | scope is right — only session references appear |

Also measured on that scratch: `disagree` resolves `session_id` to **A1's** row (A1 is
primary) while reporting the disagreement; every memory appears **exactly once** even the
ambiguous-path one (no fan-out); and `INSERT` / `UPDATE` / `DELETE` / `TRUNCATE` on the view
are all refused (it is genuinely read-only).

### 4.3 No existing behaviour changed

A second scratch was built with 001–021 and **no** 022. The full
`information_schema.columns` + `pg_constraint` + `pg_indexes` + FK list for `memories` and
`archived_sessions` was dumped from both and diffed:

```
IDENTICAL — no column, constraint, index or FK on either table changed
```

`source_ref` carries **0** foreign keys after 022, as before.

### 4.4 Idempotency, with fingerprints

A full-table `md5` fingerprint over `memories` + `archived_sessions` rows, and a `md5`
fingerprint over the whole `pg_class` catalog:

```
live-shaped scratch   rows d4e56da9…  catalog 5ba84e95…   identical across applies #1/#2/#3
fresh scratch (001→022 from zero)
                      rows d5a7093a…  catalog 13f25ed7…   identical across applies #1/#2/#3
```

So a second apply writes **zero rows and performs zero catalog writes**. The `COMMENT ON
VIEW` sits *inside* the same catalog guard as the `CREATE VIEW`, so a re-apply does not even
rewrite `pg_description` — that is why the catalog fingerprint is byte-identical.

### 4.5 The fresh-database path

001→022 applies cleanly from zero in `apply-db.sh`'s exact order. On a fresh database the
drift number is `0` of `0` view rows — correct and honest: no memory has been written yet,
so there is nothing to drift.

### 4.6 `apply-db.sh`'s completeness guard

Run for real against the schema directory: **20 migrations classified, 0 unclassified**. The
guard would have refused to build any database had 022 been left off the list.

### 4.7 One defect caught by the scratch apply, and fixed before landing

The first draft of the A1 strip was `substring(m.l2_file_path from '/Documents/GitHub/AgentHistory/')`
— with **no capture group**. In PostgreSQL that returns the *matched text*, not the remainder, so
every row normalized to the literal string `/Documents/GitHub/AgentHistory/` and A1 matched
**zero** rows. The view still returned 90 rows and still answered "3" — because A2 was doing all
the work, and the broken half of the join was invisible in the number.

It was caught by the scratch apply (§4.1), where `a1_resolves` came back `0` against a live
measurement of `90`. The fix is `(.*)$` inside the pattern (022:248). The view was **dropped and
recreated** — which is the documented path for a definition change, never a silent rewrite.

This is recorded because it is the exact failure class this whole slice exists to prevent: a join
that reports a plausible number while one of its two keys silently resolves nothing. A future
reader of the drift number should know it was nearly wrong in precisely that way.

## 5. What I did not do

- **Did not touch the live database.** No migration applied, no row written, no gate event.
- **Did not repair the 3 broken references.** forget + remember is the contract's rule
  (§4.2 rule 4), and that is a write-path act, not a schema act.
- **Did not add an FK or a CHECK on `source_ref`.** The contract forbids it explicitly.
- **Did not widen the washing pipeline's scope.** Operator's decision, not this dispatch's.
- **Did not build the read model.** That is slice 2, Nimbus's, blocked on this announcement.

## 6. For Nimbus (slice 2, the read model)

1. **The drift number is `WHERE NOT a2_resolves`**, not `WHERE join_state = 'none'`. A memory
   can be attached to a session by its path pointer while its recorded reference resolves
   against nothing — that is exactly the silent break, and it is 3 today.
2. **`a1_path_rows > 1` means the path is ambiguous.** `raw_path` is not unique: one cursor
   directory is declared by two sessions. The view refuses to pick, so a read must report
   the ambiguity rather than resolve it.
3. **`session_message_count IS NULL` means the joined session is inventory-only**, outside
   the 183-row corpus. 021's hand-off note #1 — filter on it or you double-count.
4. **"No distilled records" is zero rows for that session**, not `join_state = 'none'`. The
   contract's §7.2 requires the two look different; keep them different.
5. The view carries no index beyond what 011/021 already have. At 272 sessions × 90 memories
   a seq scan is sub-millisecond; add an index only when a measurement says to.
6. Never parse a topic id as a key (contract §4 anti-pattern 1). The view does not.

## 7. Hand-off for the live apply

`scripts/apply-schema-to-live.sh` now includes 022 and prints three lines for it: the view
exists, its row count, and the drift number — the last one **reports an error if it is not
3**, because a number that silently changes means something moved and 张先生 should ask
before anyone acts on it. That script is the auditable path for the live apply; it is not run
here, because touching live is not this dispatch's act.

---

## 8. Addendum — the event id, and why there are two announcements

**`emit_event` generates a nanoid `event_id` when you omit the parameter.** I omitted it, so
the slice-1 announcement first landed as `ev-xv1u55DLCLznYELeQJ2S_`. The contract's §11.1
mandates the id `ev-session-key-view-landed` verbatim, and §11 states that *"Dependency
edges are event ids, never prose"* — so with only the generated id, `disp-session-key-readmodel`'s
edge `ev-session-key-view-landed` could never be satisfied and slice 2 would have stayed
`blocked` forever. Nothing in the product would have said so; Nimbus would simply never have
been woken.

The fix was to emit the announcement again **under the mandated id**, which is what the
dependency resolver matches on (`dd.dep_value = NEW.id`, 013). Verified afterwards:
`disp-session-key-readmodel` flipped `blocked` → `ready` for `seat-nimbus-001`, and the other
slices are unchanged.

The ledger is append-only, so the first row was left standing rather than hidden or rewritten.
Both rows are `ArtifactCreated` from `seat-onyx-001`, both describe **one** artifact at **one**
commit (`0e32054`), and the second carries a `why_this_event_id_is_a_restatement` field saying
exactly this. Neither carries a `verified` field — both are announcements, and
`ev-session-key-view-landed` is not gate-shaped under 020's criterion (`%-live-%` / `%-live`),
so when 020 lands live the resolver will still treat it as plain existence.

The general lesson, recorded as a memory: **when a contract names an event id, that id is
infrastructure, not a label.** Pass it explicitly.
