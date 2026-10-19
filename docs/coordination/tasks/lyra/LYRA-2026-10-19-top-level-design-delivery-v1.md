# Delivery — top-level architecture design

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-19-top-level-design-delivery-v1 |
| status | delivered |
| author | lyra |
| date | 2026-10-19 |
| version | v1 |
| tags | delivery, architecture, layers, authority |
| depends_on | LYRA-2026-10-19-top-level-design-v1.md |

## What was delivered

`docs/coordination/tasks/lyra/LYRA-2026-10-19-top-level-design-v1.md`, committed as `274fad9`.

A vertical architecture, not a second inventory. It lifts the delivered capability audit into five
responsibility layers, defines where authority lives, and maps the user-approved first-screen
direction onto that model.

**The five layers** — A execution/observation, B authoritative record and evidence, C work
interpretation and query, D governance and decision, E presentation and interaction — each with an
exclusive responsibility and an explicit "does not own" boundary, so no responsibility is both
duplicated and unowned.

**Where authority lives** — B is the sole boundary for recorded activity and evidence; C is the
sole authority for published interpretations and metrics; E is neither. This resolves the audit's
central finding without relabelling existing tables: the same number in several places is one
contract presented several times, never several formulas.

**The metric contract** — a count is identified by meaning + population + scope + status predicate
+ as-of time + source coverage. "Active dispatches" and "active workitems" are different counts and
may differ; calling both "progress" without declaring their relationship is what the design
forbids.

**First-screen position** — E renders C's shared work-and-attention model with D-backed actions.
The prior judgment stands: the dispatch page plus acceptance area is the closest existing anchor.
Its placement is not the architecture; those sections should consume the shared contract rather
than define its meaning.

**Five placement rules** — each decidable and testable: effect/authority, metric identity, decision
obligation, evidence-and-state, entry justification. A new interface must present the shared
contract, accept a governed action, or provide a distinct evidence zoom with a return path.

**User choices kept separate** — default scope, disclosure of internal bookkeeping, which
duplicated entry stays visible and the shortcut ruling. These are listed as the operator's, not
resolved by this document, and no navigation removal or new interaction is specified.

## Evidence basis

Baseline `track/infra-foundation@cfda740` in the shared integration worktree. The completed audit
(`9d3dbf7`) is the source for reachability, ordering, hydration, counter inconsistency, mock global
summary and retained local-only forms; no tab inventory was repeated.

Additional source checks supporting the boundaries: the core write API explicitly maintains
projections; dispatch lifecycle uses the transaction-aware append primitive; the acceptance command
validates verdict value and subject before its external merge; intent planning has a single facts
gatherer and returns `wrote:false`; archive manifests point to original bytes on disk.

The 2026-09-30 design reviews were read and used for the activity/object/context and
ledger-versus-projection distinctions. They are cited as draft references, not as shipped
capability, and the five responsibility layers here are explicitly distinguished from those three
activity-fidelity tiers.

## Verification

- `bash scripts/verify-docs.sh HEAD` before commit: all changed docs conform.
- `git diff --check`: no whitespace errors.
- Source-level consistency review: every responsibility has one owner; all three first-screen
  questions map onto the model; effects and unknown outcomes have a route back to the record;
  evidence is reachable without copied authority.

Not claimed: that any screen currently implements the shared contract; that every projection is
replayable; that the supervisor delivery branch is closed; that a rendered app behaves this way.
No code, live-database, gate, merge, acceptance card or sub-task issuance.

## Ledger

`ev-...` ArtifactCreated + SupervisorMessage to Aegis recorded in this turn.
