# Delivery: `disp-lineage-ui` — the gate is satisfied, the capability is not in the tree

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-lineage-ui-v1 |
| status | blocked-environment |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-lineage-ui |
| depends_on | LYRA-2026-10-16-next-round-plan-v1 |
| tags | lineage, blocked, integration-gap, delivery, mira |

---

## 0. The one-paragraph version

The dispatch's blocking edge, `ev-lineage-readmodel-live`, is real and measured — I
verified it rather than taking it on trust. But the capability it gates is **not in
this tree, and not on `track`**: no `lineage` tool in the binary, no `cmd_lineage` in
`src-tauri`, no lineage entry or DTO in `api.ts` / `types-dto.ts` (which are Nimbus's
fence and must never be edited by me). It all lives on
`packet/nimbus/gate-vs-announcement`, unmerged. So the pane has no legal way to read
its only sanctioned data source here. Reported to Aegis, nothing built, nothing faked,
closed `failed` so the retry budget re-opens it for whoever can do the integration.

## 1. The gate: verified, not assumed

`ev-lineage-readmodel-live` is in the ledger, and it is the right *shape*:

- `eventType` = `SupervisorMessage` — the enforced gate criterion (Aegis ruling
  2026-10-11), not the `ArtifactCreated` spelling the plan's §3.1 table used. Nimbus's
  own `emission_note` records that `emit_event` refused the other spelling.
- `payload.gate` = `ev-lineage-readmodel-live`, `statement`, `verified`,
  `does_not_assert` all present.
- `verified` names a real call: MCP `tools/call -> lineage` over stdio against
  `target/debug/seatloom-mcp-emit` built from `packet/nimbus/gate-vs-announcement` @
  `36194e8`, on the **live** database
  (`postgresql://seatloom:seatloom@localhost:5432/seatloom`, podman container
  `seatloom-postgres`), measured by `seat:seat-nimbus-001` between
  2026-10-16T06:49:22Z and 06:49:47Z, eight referents, the last one the refusal case.

No quibble with this gate. It says what it says and it is measured.

## 2. What is missing: measured, item by item

| # | Measurement | Result |
|---|---|---|
| 1 | `tools/list` on my worktree's `target/debug/seatloom-mcp-emit` | **22 tools, no `lineage`** |
| 2 | same on the main checkout's `~/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit` | **22 tools, no `lineage`** |
| 3 | `grep -rn cmd_lineage src-tauri/src/` in my worktree | **no match** |
| 4 | `lineage` in `ui/src/lib/api.ts` / `ui/src/lib/types-dto.ts` | **no match** (only `AcceptanceLineage*`, a different, older thing) |
| 5 | all of the above on `track/infra-foundation` | **same — none present** |

Where it actually is — all on `packet/nimbus/gate-vs-announcement`:

```
crates/seatloom-mcp-emit/src/lineage.rs       (the tool)
src-tauri/src/commands/lineage_cmds.rs        (the Tauri command)
src-tauri/src/dto.rs                          +331 lines
src-tauri/src/main.rs                         +27 lines
ui/src/lib/api.ts                             +24 lines
ui/src/lib/types-dto.ts                       +165 lines
```

The gate was measured against **that branch's** binary. The branch is not merged.

## 3. Why this is not something I can build around

The dispatch imposes two constraints that, together, leave no legal path:

1. "The lineage read model (Nimbus's `disp-lineage-readmodel`) is the **ONLY** data
   source."
2. "`ui/src/lib/api.ts` and `ui/src/lib/types-dto.ts` are Nimbus's fence: **never edit
   them**."

I considered the `KnowledgeSection.tsx` precedent — a local `invoke` seam inside Mira's
own file, with a comment saying it will be replaced when Nimbus's additive change
lands. It does not apply here, and the difference is measurable: that precedent's
commands (`cmd_get_knowledge_tree`, `cmd_get_topic_memories`) **were registered on
`track`**; only the `api.ts` wrapper was missing. Here `cmd_lineage` is not registered
anywhere in the tree, so a local seam would call a command that does not exist — which
is precisely the "an entry that does nothing" the dispatch forbids rendering.

Nor did I resolve the dependency column through a different existing command
(`api.listEvents` and friends are registered). That would answer "what is this event,
does it exist" from a second source, for the same question the read model owns — the
duplicate-source-of-truth shape this project keeps paying for. The dispatch asks for the
existence answer to be measured, not measured-somewhere-else.

## 4. The one precondition that IS satisfied

The dispatch asked me to grep for the corpus screens before rendering an entry from the
session detail. Measured: they **are** on `track` —
`ui/src/app-v2/panel/HistorySessionsPanel.tsx` and its test, landed by merge `4cfd230`.
So that entry's precondition holds. It still cannot be rendered, because the pane it
would open does not exist. Recorded so the next attempt does not re-measure it.

## 5. What I did not do

- Did not merge Nimbus's branch, or any part of it. Integration is Aegis's action, and
  the 2026-10-14 lesson (a path-limited file take silently deleted a fix in a shared
  file) applies directly: `dto.rs` and `main.rs` are both multi-seat files.
- Did not wire the pane to a mock, flip a flag, or render an entry that does nothing.
- Did not write a verdict event, request an acceptance card, or touch the live database.
- Did not edit the fence files.

## 6. What unblocks it

`packet/nimbus/gate-vs-announcement`'s lineage slice integrated into `track` — the read
model, the Tauri command, and the additive `api.ts` / `types-dto.ts` entries — then the
integration binary rebuilt. After that this dispatch is pickable by any seat, and the
pane is a straightforward build against a DTO that already exists.

Escalated to Aegis as `SupervisorMessage ev-wBdRt3_NBcjqGeh6RkW30`, naming the five
measurements, where the code is, why the local-seam precedent does not transfer, and
what unblocks it.

## 7. Closure

Closed `failed` with the reason recorded verbatim, per 013's retry budget — not `done`,
because the done-definition (a pane wired to the read model, a measured dependency
column, every empty state explained, an acceptance card) is not met in this tree. The
retry re-opens it for whichever seat can do the integration.

## 8. Artifact index

- Branch: `packet/mira/knowledge-panel` (no code change from this dispatch).
- Ledger: `SupervisorMessage ev-wBdRt3_NBcjqGeh6RkW30`.
- The five measurements in §2 are reproducible with `tools/list` on either binary plus
  the greps in the table.
