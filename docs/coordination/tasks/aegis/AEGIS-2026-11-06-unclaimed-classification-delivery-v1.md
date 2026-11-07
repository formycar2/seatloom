# Restore visibility of unclassified done-without-card work

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-06-unclaimed-classification-delivery-v1 |
| status | delivered |
| author | aegis |
| date | 2026-11-06 |
| version | v1 |
| tags | heartbeat, classification, correction |

## Scope

Delivery for disp-UnBvI7cGr3MPSKGSChrsY and correction disp-g1m-vpHALg43AuID1JSlk.
Changed scripts/check-unclaimed-work.sh; added scripts/test-unclaimed-work.sh.

The previous acceptance-key exclusion was unsupported. That key carries verification
instructions, not an authoritative category. Missing kinds now remain visible as
unknown candidates. A candidate is NOT automatically an operator acceptance obligation:
card-opening coordination may reference a different delivery. Explicit verification
requests are excluded, with a count printed in both empty and nonempty normal reports.
The inherited 48-hour diagnostic window remains explicit; this is not the complete
unified work list and does not close historical obligations.

## Verification

bash scripts/test-unclaimed-work.sh passes: injected candidate rendering, exclusion
count, quiet behavior, failed/empty sources. SQL policy assertions are STATIC, not
an independent database-fixture proof. Live read exits 1 with seven review candidates
and nine explicit verification exclusions at this snapshot. No live writes or false
cards were created. Queries execute in one read-only repeatable-read transaction;
failed reads exit 2 rather than becoming empty. Five standard salvage/card/verdict
checks exit 0; self-wake check succeeds. Ledger has 35 tables.

Lyra's correction is acknowledged without rewriting her earlier event: inspect the
actual acceptance.dispatch_id, not the existence of some accepted card elsewhere.
Raw counts are window- and time-dependent; they are not a task-classification oracle.

## Forward work

Flux's clean integrated guard verdict ev-hW75U60DweOY5-qM5pc5r names 6f1c566 and has
empty findings. Nimbus has been assigned preparation of the original delivery's
matching acceptance card, not permission to merge/deploy. Deployment preview 703b40c
and work list dc97a16 now exist on packet/nimbus/deploy-preview-dry-run. Independent
verification was re-dispatched because Flux's earlier no-implementation report was
correct at its observation time. Each implementation needs its own verdict.

No new durable memory entry proposed: this correction and its evidence are already
recorded in the dispatches and this delivery. No duplicate memory claim is justified.
No installation, runtime restart, branch integration or live database change performed.
