# Flux's attack scripts for `disp-session-key-verify` (the session↔work bridge)

Slices under test: Onyx's view `022_session_work_bridge_view.sql` @ `0e32054` on
`packet/onyx/topic-opened-019`, and Nimbus's read model `703c863` on
`packet/nimbus/gate-vs-announcement`.

Nimbus's own proof (`crates/seatloom-mcp-emit/proof/session_work_readmodel_proof.sh`) passes. These
scripts attack what it does not construct: the live-shaped scratch it builds has **no** `ref_only`,
`none`, `disagree` or ambiguous-path row, so those states are built here.

**Run Nimbus's proof first** — it builds the scratch `seatloom_sw` these scripts read.

| Script | What it attacks |
|---|---|
| `flux_session_work_attack.sh` | the misspelled-`source_ref` family (`none`, `ref_only`, no-colon, empty-id, multi-colon), the topic-id lookup, the empty topic, unknown/empty keys, and the load-bearing wall from three sides |
| `flux_sw_followup.sh` | the two states that do NOT panic (the ambiguous path and `disagree`), and the exact boundary of the one that does |
| `flux_sw_final.sh` | the source file gone from disk, the wall in the states that do answer, read-only and one-implementation |

All three are READ ONLY against the scratch. Every one of them constructs rows on `seatloom_sw`,
which is disposable and was never the live database.

## The finding, in one paragraph

022 computes `(a1.path_rows = 1) AS a1_resolves`. `a1` is a `LEFT JOIN LATERAL … LIMIT 1`, so when
**zero** `archived_sessions` rows declare a memory's corpus path, `a1.path_rows` is NULL and
`NULL = 1` is NULL — `a1_resolves` is NULL, not false. `MemorySessionJoinRow.a1_resolves` is a plain
`bool` (`crates/seatloom-core/src/db/models.rs:718`), so `r.get(15)` at
`crates/seatloom-core/src/db/repositories.rs:3288` cannot deserialize it and **panics**, with no
JSON-RPC response at all. It is not contained (a second, good call in the same process never runs) and
not one memory (the forward read of a session that merely owns one such memory panics too). The
view's own `join_state` for that row is correct (`ref_only`) — the SQL is right, the reader is wrong.
On live today 0 of 90 memories hit it, so it is a landmine rather than an outage, but it sits exactly
where the contract's load-bearing wall is: the A2 half of a broken key is reported beautifully, the A1
half takes the product down.

Reproduce the panic in one line, after the proof has built the scratch:

```bash
podman exec -i seatloom-postgres psql -U seatloom -d seatloom_sw -tAc \
  "INSERT INTO memories (id,project_id,topic_id,title,tags,l0_summary,l2_ref_kind,l2_file_path,
     l2_body_digest,source_kind,source_ref,heat,recall_count,tier,tier_changed_at,status,created_at,
     updated_at,kind,confidence,l2_line_start,l2_line_end)
   VALUES ('mem-flux-panic','seatloom',(SELECT id FROM topics ORDER BY id LIMIT 1),'p','{}','s',
     'file','raw/claude-code/no-such-session.jsonl',repeat('a',64),'archived_session',
     'claude-code:01565715-3e7f-48c8-9ff0-fdd8edbfdf9f',0,0,'warm',NOW(),'active',NOW(),NOW(),
     'decision','quoted',1,10);"
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"memory_session","arguments":{"memory_id":"mem-flux-panic"}}}' \
  | DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_sw \
    SEATLOOM_ACTOR_REF=human:supervisor SEATLOOM_PROJECT_ID=seatloom \
    target/debug/seatloom-mcp-emit 2>&1 >/dev/null | head -3
# -> thread 'main' panicked at crates/seatloom-core/src/db/repositories.rs:3288:24:
#    error retrieving column 15: error deserializing column 15
```
