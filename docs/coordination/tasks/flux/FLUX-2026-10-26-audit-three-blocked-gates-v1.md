# Flux audit — three blocked gates, judged independently

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-26-audit-three-blocked-gates-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-26 |
| version | v1 |
| depends_on | AEGIS-2026-10-26-audit-three-blocked-gates-v1 |
| tags | audit, gates, verification, seat-flux |

- **Auditing dispatch**: `disp-VH6l1HixlUVYuy4m0NEp-` (Aegis → Flux)
- **Packet**: `docs/coordination/tasks/aegis/AEGIS-2026-10-26-audit-three-blocked-gates-v1.md`
  — **not on my branch and not on track**; I worked from the dispatch payload, which is
  fully specified, per AGENTS.md.
- **Why this audit exists**: Aegis already misjudged one of these gates (reading the
  89 NULL `message_count` rows as a hole when 021 defines them as the loaded sentinel
  separating the 183-row corpus from the 271-row inventory). A seat that has judged
  wrong once cannot be the one that certifies its own correction.

## Verdicts, one per gate

| Gate | Implementation dispatch | Verdict | What it actually rests on |
|---|---|---|---|
| **A** — first screen default tab | `disp-first-screen-default-tab` | **PASS on the work**, the **gate event does not exist** | code measured at three refs, no smuggling; announcement content complete but emitted under the wrong id |
| **B** — corpus read | `disp-history-sessions-ui` | **PASS on the capability**, the **gate event is malformed** | 183/89/36680 all reproduced; Aegis's "89 is a defect" is the reverse error; the event that satisfies the block is the wrong type from the wrong seat with no `verified` field |
| **C** — session↔work bridge | `disp-XnCxvGI88oAbdGcyeefhl` | **NOT SATISFIED — and that is correct** | the view genuinely does not exist on live, confirmed across every schema; drift number 3 reproduced read-only |

**The one finding that matters more than any single gate**: the cards have been stuck for
ten days not because the work is missing — **two of the three gates' work is done and
correct** — but because the release mechanism is an **exact-string event-id match with no
other channel**, and the ids the dispatches wait on were never emitted under that exact
string. `referent_exists` (`repositories.rs:795`) is a plain `SELECT EXISTS (SELECT 1
FROM canonical_events WHERE id = $1)`. There is no fuzzy match, no alias, no
"same commit, different id" fallback. Work that is finished and correctly announced does
not release a block; only the exact id does.

---

## Gate A — the first screen's default tab

### The work: measured true at three refs

`useState<Tab>('dispatches')` — the claim holds:

| Ref | Value |
|---|---|
| `track/infra-foundation:ui/src/App.tsx:132` | `useState<Tab>('dispatches')` |
| `packet/mira/knowledge-panel:ui/src/App.tsx:132` | `useState<Tab>('dispatches')` |
| parent `5c5437f~1` | `useState<Tab>('live-seats')` |

One note for the record: the contract and the dispatch both say `App.tsx:92`. That is the
**pre-change** line number — the commit's own diff shows `@@ -88,8 +88,12 @@` moving the
line from 92 to 96 in Mira's tree, and it sits at 132 on track. The number is stale, not
wrong; the line is where it was said to be.

### No smuggling — the whole diff is one line and a comment

```
ui/src/App.tsx  |  6 +--      (6 insertions, 2 deletions)
```

Six insertions: the changed line plus **five comment lines** replacing the one comment
that would otherwise have contradicted the code ("UM0: open on the live mirror, the one
surface that proves itself" is false the moment the line changes). Two deletions: the old
line and the old comment. **Nothing else moved.** The contract forbids adding a partition,
column, component, route or visual treatment — a comment is none of those, and a comment
that contradicts its own code is the misleading-comment failure this project has already
paid for.

Also verified independently:
- **Nothing persists `activeTab`.** The only `localStorage` keys `App.tsx` touches are
  `seatloom.supervisor.open` (get and set), so the default applies on **every** open, not
  just the first.
- **Zero `metaKey`/`ctrlKey` lines in the diff** — the ⌘K layer is untouched.

### The announcement: complete content, wrong id

`ev-vrg-SFOoMv9BBj2jx_uSG` (`ArtifactCreated`, `seat-mira-001`, 2026-10-16T10:17) carries
**every element the contract's §6.1 requires**: `branch`, `commit`, `files`, and
`the_exact_one_line_change`. It correctly has **no `verified` field** — the contract says
"Announcement; no `verified` field" for this id, and Mira honoured that. Its `not_proven`
field is unusually honest: *"Nobody has opened the built app."*

**But the contracted id was `ev-first-screen-default-tab-landed`, and it was never
emitted.** Mira used a generated id. `disp-first-screen-verify` is blocked on:

```
{"depKind":"event","depValue":"ev-first-screen-default-tab-landed","exists":false}
```

**Gate A verdict: the work PASSES; the gate event does not exist.** The block is real and
correct given the mechanism — but what it is waiting for has already been built,
announced, and correctly measured. It is waiting for a string.

## Gate B — the corpus read

### The capability: independently measured, and it is satisfied

| Check | My measurement | Card / Aegis's number |
|---|---|---|
| `message_count IS NOT NULL` | **183** | 183 ✅ |
| `message_count IS NULL` | **89** (codex 60 / stepcode 19 / gemini 9 / opencode 1) | 89 ✅ — same split |
| `sum(message_count)` | **36680** | 36680 ✅ |
| distinct corpus projects | **38** | 38 ✅ |
| `OfficeBench` sessions | **49** | 49 ✅ |
| `seatloom` sessions | **14** | 14 ✅ |
| `(root)` unclassified | **8** | 8 ✅ |
| cursor rows with `first_ts IS NULL` | **14** | 14 (rendered as "time not recorded") ✅ |

Every number on the operator's acceptance card reproduces exactly against live.

### Aegis's reverse error, confirmed

The dispatch `disp-2o_BDY1tb3b92ZQwaYXEC` ("fill the 89 missing message_counts, drive the
count from 89 toward 0") **is built on a misreading**, and 021's own column comment says
so in as many words:

> `NULL = no extracted conversation exists for this row — that is what distinguishes the
> 183-row corpus from the 271-row archive inventory, so a project-grouped read must
> filter on this column being NOT NULL to avoid double counting the 89 inventory-only
> rows.`

Driving NULL → 0 does not close a hole; it **destroys the sentinel** and makes every
project-grouped count double. Vela refused to write and escalated, which was right, and
independently reproduced the 89 and the per-harness split **from the CSV and the on-disk
archive without reading the database at all** — matching Aegis's live measurement exactly.
That is the correct way to disagree with a live measurement: reproduce it from a different
source.

**Of the 89, 0 can be filled from data that exists today.** 51 codex + 19 stepcode have raw
files on disk but are outside the 183-session corpus scope; 9 codex were archived then
pruned; 9 gemini have no transcript at all (`AGENTS.md`: gemini is register-only this
round); 1 opencode points at a path that does not exist on disk. Filling any of them is
fabrication.

### The code is correct, both halves

- `repositories.rs:970` — `LEFT JOIN archived_sessions a ON a.project = p.id AND
  a.message_count IS NOT NULL`
- `repositories.rs:1018` — `AND a.message_count IS NOT NULL`

Both filters are gated on `corpus_columns_present()`, so a database without 021 refuses
honestly rather than reading zero — I read that guard myself rather than taking it on
trust.

- `HistorySessionsPanel.tsx:141` — `countText(n) { return n == null ? 'not loaded' : String(n); }`
- `HistorySessionsPanel.tsx:373` — a red note, in the operator's language: *"message count
  not loaded: the corpus migration (021) has not been applied to the current database, so no
  number can be given here — this is not 0."*

**The UI never shows 0 for NULL.** It says the migration is absent and that this is not
zero. That is exactly the honest-empty behaviour the contract asks for.

### The gate event: right id, wrong everything else

`ev-history-sessions-read-live` **does exist** (`exists: true`), which is why
`disp-history-sessions-ui` was released and completed. But read against the contract's
§8.1 table:

| Required | Actual |
|---|---|
| Type `ArtifactCreated` | **`SupervisorMessage`** |
| Emitter Nimbus | **`seat-aegis-001`** |
| Payload **`verified`** (mandatory): which call, exactly what it returned, on which database, when | **absent** — a `conditions` object with three prose `DONE` notes |

So the block was released by an event that is the wrong type, from a seat other than the
one the contract names, missing the one field the contract marks mandatory. The
measurement inside it is accurate — I reproduced it — but the event does not meet the
contract it claims to satisfy.

**Gate B verdict: the capability PASSES; the gate event is malformed.** Aegis's "satisfied"
is right about the substance and wrong about the reason, and the reason matters because
the same misreading produced a dispatch (`disp-2o_BDY1tb3b92ZQwaYXEC`) asking a seat to
corrupt the corpus.

## Gate C — the session↔work bridge

### The view genuinely does not exist — I checked every schema, not just public

The packet warned "maybe it's in another schema/name". It is not:

| Query | Result |
|---|---|
| `information_schema.views` (all schemas) | **142 rows**, all `pg_catalog`/`information_schema` except one: `public.v_archived_sessions_normalized` |
| non-system schemas | **`public` only** |
| `pg_class` where `relname ~* 'session\|memory'` | `archived_sessions`, `sessions`, `memory_relations`, `v_archived_sessions_normalized` + indexes. **No view** |

**`v_memory_session_join` does not exist on live, in any schema, under any name.**

### Why it cannot be applied yet

`022_session_work_bridge_view.sql` exists on `packet/onyx/topic-opened-019 @ 0e32054` and
on Nimbus's branch (`f7bfe23`). It is **not on `track/infra-foundation`**, so
`scripts/apply-schema-to-live.sh` cannot reach it.

**The reason is specific and I reproduced it**: `apply-db.sh`'s completeness guard refuses
to apply a partial schema. Of the 20 files in `infra/postgres/schema/`, exactly **one** is
in neither `PRE_SEED_SCHEMA` nor `POST_SEED_SCHEMA`:

```
UNCLASSIFIED: 021_history_sessions_corpus.sql
```

021 was applied to live **out-of-band** (the operator ran a script on 2026-10-16), which
is why live has the corpus columns while `apply-db.sh` still refuses. So the canonical
path to 022 is blocked by 021's missing registration — and 022's own registration note
already says "Canonical order 021 → 022". Nimbus reported this on 10-22; I re-checked and
it is still true. **Registering 021 in `apply-db.sh` is the single change that unblocks the
canonical path to Gate C**, and it is Aegis's/operator's call, not a verifier's edit.

### The drift number, reproduced read-only

I ran the view's exact join logic **inline as a CTE, without creating the view**, so live
was never written:

```
 resolved_by | rows
-------------+------
 both        |   87
 a1_path     |    3
```

Exactly Nimbus's claim. The three rows are one `source_ref`,
`codex:019dda18-07f6-79e0-8008-e4eb16a5448d`, carried by 3 memories: the **path key A1
resolves** (the row exists with that `raw_path`) but the **source_ref key A2 does not**,
because 021 re-keyed `id` to the native session id while those 3 memories keep the file
name. Two further facts I measured that sharpen the finding:

- that row's `message_count` **is NULL** and `first_ts` **is NULL** — it is one of the 89
  inventory-only rows, and its id is **not in `SESSIONS-FLAT.csv`**. So the 3 memories
  point at a real session that has **no extracted conversation**, which is a different and
  more precise statement than "the reference is broken".
- `resolved_by='unresolved'` is **0** and `missing IS NOT NULL` is **0** — nothing points
  at nothing. The drift is a key disagreement, not an orphan.

**Gate C verdict: NOT SATISFIED, and that verdict is correct.** Emitting the gate now would
assert something false — a real call against live fails with `ViewMissing`, which is the
honest answer for that database. This is the one gate of the three where the block is
protecting real work.

---

## What I did not check

- **`sessions` row count after 022.** Nimbus reports 5 → 5 → 5 on a seeded scratch DB. Live
  has 5 rows now; I did not apply 022 to verify post-apply, because applying a migration to
  the live database is not this seat's act.
- **A fresh `pnpm install`.** Not relevant to these three gates, but stated so nobody reads
  it in.
- **The full `App` rendered.** Gate A's four operator behaviours were verified at source
  level (the default value, the absence of any `activeTab` persistence, the untouched ⌘K
  layer), not by opening the built app. Mira's announcement says the same thing about her
  own work, which is the honest position for both of us.
- **`apply-db.sh` end-to-end.** I simulated its completeness guard by listing the schema
  directory against the two arrays; I did not run a full apply, which needs a database and
  is not mine to build.

## Reproduction

```bash
# ── Gate C: the view is absent, in every schema ──────────────────────────────
podman exec seatloom-postgres psql -U seatloom -d seatloom \
  -c "select table_schema, table_name from information_schema.views \
      where table_schema not in ('pg_catalog','information_schema');"
#  public | v_archived_sessions_normalized     <-- the only user view

podman exec seatloom-postgres psql -U seatloom -d seatloom \
  -c "select n.nspname, c.relname, c.relkind from pg_class c \
      join pg_namespace n on n.oid=c.relnamespace where c.relname ~* 'session|memory';"

# ── Gate C: the drift number, read-only, no view created ────────────────────
podman exec -i seatloom-postgres psql -U seatloom -d seatloom -f - <<'SQL'
WITH keyed AS (
  SELECT m.id, m.source_ref,
         regexp_replace(m.l2_file_path,'^~/Documents/GitHub/AgentHistory/','') AS a1_path,
         split_part(m.source_ref,':',1) AS a2_harness,
         substring(m.source_ref from position(':' in m.source_ref)+1) AS a2_id
  FROM memories m WHERE m.source_kind='archived_session' AND coalesce(m.source_ref,'')<>''
)
SELECT CASE WHEN s1.id IS NOT NULL AND s2.id IS NOT NULL THEN 'both'
            WHEN s1.id IS NOT NULL THEN 'a1_path'
            WHEN s2.id IS NOT NULL THEN 'a2_source_ref' ELSE 'unresolved' END AS resolved_by,
       count(*)
FROM keyed k LEFT JOIN archived_sessions s1 ON s1.raw_path = k.a1_path
             LEFT JOIN archived_sessions s2 ON s2.harness=k.a2_harness AND s2.id=k.a2_id
GROUP BY 1 ORDER BY 2 DESC;
SQL
#  both | 87    a1_path | 3

# ── Gate B: the three numbers, and the per-harness NULL split ───────────────
podman exec seatloom-postgres psql -U seatloom -d seatloom -c \
  "select count(*) filter (where message_count is not null) corpus,
          count(*) filter (where message_count is null) inventory,
          sum(message_count) total from archived_sessions;"
podman exec seatloom-postgres psql -U seatloom -d seatloom -c \
  "select harness, count(*) rows, count(message_count) non_null,
          count(*)-count(message_count) null_count from archived_sessions group by 1;"

# ── Gate A: the default tab at three refs ───────────────────────────────────
git -C <mira> show track/infra-foundation:ui/src/App.tsx | grep -n "useState<Tab>"
git -C <mira> show packet/mira/knowledge-panel:ui/src/App.tsx | grep -n "useState<Tab>"
git -C <mira> show 5c5437f~1:ui/src/App.tsx | grep -n "useState<Tab>"
git -C <mira> show 5c5437f --stat --format=""        # one file, 6 insertions, 2 deletions

# ── The mechanism that makes a finished job still block ─────────────────────
#   crates/seatloom-core/src/db/repositories.rs:795  referent_exists()
#   "SELECT EXISTS (SELECT 1 FROM canonical_events WHERE id = $1)"
#   — an exact id match. No alias, no fallback.

# ── The one change that unblocks Gate C's canonical path ────────────────────
cd <repo>/infra/postgres/schema
classified=$(grep -oE '"[0-9]{3}_[a-z_]+\.sql' ../../scripts/apply-db.sh | tr -d '"')
for f in *.sql; do case " $classified " in *" $f "*) ;; *) echo "UNCLASSIFIED: $f";; esac; done
#  UNCLASSIFIED: 021_history_sessions_corpus.sql
```

## Verdict events

One `ReviewVerdictIssued` per gate, each naming its own implementation dispatch in
`payload.dispatch_id` (the contract spelling — the correction Aegis made me apply earlier
today):

| Gate | dispatch_id | verdict |
|---|---|---|
| A | `disp-first-screen-default-tab` | pass, with the gate event absent |
| B | `disp-history-sessions-ui` | pass, with the gate event malformed |
| C | `disp-XnCxvGI88oAbdGcyeefhl` | fail, correctly |

## Files I did not change

No source file, no migration, no live-database write of any kind (every database access in
this audit was a `SELECT` or a read-only CTE). I did not emit `ev-first-screen-default-tab-landed`
— inventing a gate id another seat is contracted to emit would be exactly the
self-certification this dispatch exists to prevent.
