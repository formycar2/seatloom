# Consolidated Joint-Review Amendments — Product × Architecture Supplements (2026-05-09)

| Field | Value |
|---|---|
| template | T4 |
| subtype | review_consolidation |
| id | 2026-05-09-lyra-nimbus-joint-review-amendments-v1 |
| status | issued (awaiting Aegis adjudication) |
| author | lyra |
| date | 2026-05-09 (late evening) |
| to | aegis, nimbus |
| depends_on | `docs/coordination/reviews/2026-05-09-lyra-seatloom-full-product-design-v1.md` (commit 3ccbd6d + later amendments), `docs/coordination/reviews/2026-05-09-nimbus-seatloom-full-arch-design-v1.md` (commit dd8758c + Lyra §Review at commit 14b0289); Aegis 2026-05-09 rulings (joint-review async + decision-1 numbering + decision-2 B1 parallel correction + decision-3 §1.3 ratification + decision-4 consolidate) |
| tags | joint-review, consolidation, lyra-product × nimbus-arch, amendments, awaiting-aegis-pass |
| acceptance owner | aegis (one adjudication pass over all 7 items) |

---

## 0. Why this document exists

Per Aegis 2026-05-09 late-evening directive (decision 4): "7 amendment suggestions — please consolidate into one doc with Nimbus, Aegis adjudicates in one pass to avoid fragmentation."

This file consolidates every amendment surfaced during the async joint review between Lyra's product supplement and Nimbus's arch supplement. Each amendment is keyed to the source doc, lists the proposed change, gives the rationale, and identifies the actor responsible for the follow-up edit. Aegis adjudicates by approving / modifying / rejecting each item in one pass.

Aegis decisions already in force (recorded here for completeness, not requiring re-adjudication):
- **Decision 1**: Migration numbering binding — 006 plan_mode_authority → 007 seats_budget → 008 project_isolation. (Lyra amendments §A.1, §A.2, §A.3 below operationalize this in the product doc; status: already applied.)
- **Decision 2 (corrected)**: B1 is bytes-only, writes no `canonical_events`, **not blocked by 008**. 008 dispatches in parallel. (B1 packet status restored; 008 packet reframed; status: already applied. Lyra amendment §A.5 records the §1.4 product-doc clarification.)
- **Decision 3**: §1.3 framing "FS = artifact truth, PG = projection authority" is **BINDING**. (Lyra amendment §A.4 records this in the product doc; status: pending edit.)
- **Decision 4**: This consolidated doc.

---

## 1. Amendment register

Seven amendments, grouped by source doc.

### Lyra product supplement amendments (5)

#### §A.1 — §3.3.1 schema migration number

- **Source**: product supplement §3.3.1 (`docs/coordination/reviews/2026-05-09-lyra-seatloom-full-product-design-v1.md` line 309).
- **Original text**: "schema migration `006_multiproject_scope.sql` can add the column …"
- **Amended text** (already applied 2026-05-09 late-evening): "schema migration `008_project_isolation.sql` can add the column … (Edit: number corrected from 006 to 008 per Aegis joint-review ruling — 006 is reserved for `plan_mode_authority` and 007 for `seats_budget`.)"
- **Rationale**: Aegis decision 1 binding.
- **Status**: ✓ APPLIED (commit included in this batch).

> **Aegis verdict (2026-05-11)**: APPROVED.

#### §A.2 — §7.3 ship-window recommendation

- **Source**: product supplement §7.3 (line 567).
- **Original text**: "ship `006_multiproject_scope.sql` in v0.0.1, v0.0.2, or v0.1? My recommendation is v0.0.1 …"
- **Amended text** (already applied): "ship `008_project_isolation.sql` … as a standalone packet sequenced after 006 plan_mode_authority + 007 seats_budget. Per Aegis joint-review ruling (2026-05-09 late-evening): 006 → 007 → 008, and 008 lands in the v0.0.2 window in parallel with B1 (B1 is bytes-only and creates no canonical_events rows, so 008 does not gate B1)."
- **Rationale**: Aegis decisions 1 + 2-corrected.
- **Status**: ✓ APPLIED (commit included in this batch). One residual edit: the line currently says "before B1 enters Flux verify, since B1 creates new canonical_events rows" — this should be removed since decision-2-corrected establishes B1 creates no events. **Aegis adjudication: approve this micro-correction?**

> **Aegis verdict (2026-05-11)**: APPROVED. Remove the "before B1 enters Flux verify, since B1 creates new canonical_events rows" clause; replace with the parallel-clearance wording already in the §A.2 amended text.

#### §A.3 — §8 hand-off scope wording

- **Source**: product supplement §8 (line 584).
- **Original text**: "PostgreSQL schema 006 (multi-project) and 007 (event-first projection if Option C accepted)"
- **Amended text** (already applied): "PostgreSQL schema 008 (multi-project; was originally numbered 006 in this doc — Aegis joint-review ruling 2026-05-09 late-evening reserves 006 for `plan_mode_authority` and 007 for `seats_budget`)"
- **Rationale**: Aegis decision 1 + cleanup of stale "007 event-first projection" half-thought that conflated migration numbering with §1.4 Option C event-first projection (those are different things — Option C is a design pattern, not a schema migration).
- **Status**: ✓ APPLIED. The original mention of "007 (event-first projection)" is now gone from this list, but Option C event-first projection itself stays in §1.4 as a recommended design pattern. **Aegis adjudication: confirm Option C event-first projection remains binding even though it's not a numbered schema migration?**

> **Aegis verdict (2026-05-11)**: MODIFY. Option C event-first projection stays as **PROVISIONAL**, not BINDING — it's a substantial architectural commitment (materialized views + write-path event emission throughout) that deserves its own design packet before ratification. Decision 3 ratified §1.3 (FS=truth, PG=projection) only, not the event-first refactor. Tag §1.4 Option C accordingly under §A.5.

#### §A.4 — §1.3 core principle ratification

- **Source**: product supplement §1.3 (line 64).
- **Original text**: "**File system is the source of truth for the *artifact*. PostgreSQL is the projection authority for the *runtime object*.** Reconcile is the **deterministic mapping** from artifact (markdown) to runtime object (row) using the dual-key `template + subtype` as the discriminator."
- **Amended text** (proposed, not yet applied): insert after the existing principle paragraph: "_Aegis 2026-05-09 late-evening: ratified as **BINDING** architectural principle for v0.1+. Any future packet that contradicts this framing (e.g., proposes UI as a direct writer of `workitems` rows without round-tripping through markdown, or treats PG as authoritative over the file in conflict resolution) must be flagged and reviewed against §1.3 before dispatch._"
- **Rationale**: Aegis decision 3. Promotes §1.3 from "proposed" to durable architectural constraint with explicit binding language.
- **Actor**: Lyra (after Aegis confirms wording).
- **Status**: ⏳ PENDING Aegis approval of exact wording.

#### §A.5 — §1.4 / §1.5 / §1.6 lifecycle recommendation status

- **Source**: product supplement §1.4 Option C (line 102), §1.5 Option B (line 142), §1.6 Option C (line 180).
- **Original text**: All three sections state "Recommendation: Option X" without a binding marker.
- **Amended text** (proposed): each "Recommendation" line gets a status tag — `(BINDING per Aegis 2026-05-09)`, `(PROVISIONAL — awaits Aegis adjudication)`, or `(SUPERSEDED by …)`. Default for §1.4 Option C / §1.5 Option B / §1.6 Option C is PROVISIONAL until Aegis confirms; default for §1.3 framing is BINDING per decision 3.
- **Rationale**: Reader-orientation. Without status tags, future packet authors can't distinguish "Lyra recommends" from "Aegis ratified." The §1.3 ratification (decision 3) creates the precedent for tagging the rest.
- **Actor**: Lyra (after Aegis confirms which sections take which tag).
- **Status**: ⏳ PENDING Aegis tag-assignment per section.

### Nimbus arch supplement amendments (2)

#### §A.6 — §3 numbering + scope-fit with B1

- **Source**: Nimbus arch supplement §3 (`docs/coordination/reviews/2026-05-09-nimbus-seatloom-full-arch-design-v1.md` lines 151-216).
- **Original text**: "schema migration 008 with a backfill …"
- **Amended text** (proposed, not yet applied): no number change (008 is correct), but add a one-paragraph note at the end of §3 Recommendation: "**B1 parallel-clearance (Aegis 2026-05-09 late-evening)**: 008 runs in parallel with B1, not before it. B1's bytes-only scope (`PtySession::write` → `tmux send-keys`) creates no `canonical_events` rows, so the `project_id` column on `canonical_events` is not required for B1 acceptance. 008 still lands in v0.0.2 because B2 onward (supervisor-message append paths, plan-mode approval injection, watcher-derived events) will need it. The arch §7 sequencing diagram remains valid as a *recommended* order, but is not a hard dependency between 008 and B1."
- **Rationale**: Decision 2 correction. Nimbus arch §7 shows 008 first; the correction relaxes that for B1 specifically.
- **Actor**: Nimbus (or Lyra on Nimbus's behalf if expedient — Aegis pick).
- **Status**: ⏳ PENDING Aegis approval of exact wording.
- **Nimbus inline note (2026-05-11)**: Concur with proposed wording verbatim. I'll apply it to arch §3 myself once Aegis approves — no need for Lyra to ghostwrite. Open question §3 (reciprocal review): already done at commit `b916912` — ten cross-points N1–N10 filed in Lyra's product supplement §Review block on 2026-05-11. "Still required" path is satisfied.

#### §A.7 — §Review section — Lyra's cross-point (c) retraction

- **Source**: Nimbus arch supplement §Review (commit `14b0289`, my own posted comments).
- **Original text** (my cross-point (c)): "Lyra position: 008 must land and be accepted before B1 delivery. … Lyra will not accept B1 if canonical_events still lacks project_id at the time of B1 delivery."
- **Amended text** (proposed): retract cross-point (c) inline by appending a `**RETRACTED 2026-05-09 late-evening**` note: "**RETRACTED 2026-05-09 late-evening per Aegis correction**: Nimbus correctly flagged that B1's bytes-only scope writes no canonical_events rows. Lyra's claim that B1 required 008 precedence was factually wrong. B1 is unblocked and runs in parallel with 008. Acknowledgement: this is the second time during this sprint that Lyra has over-extended a precaution into a blocker — first the SG-A retroactive-hotfix-packet question, now this. Recording for pattern-tracking."
- **Rationale**: Honest correction in the document where the wrong claim lives. Doesn't delete the original text (that would erase the audit trail); appends a retraction adjacent to it.
- **Actor**: Lyra.
- **Status**: ⏳ PENDING Aegis approval of retraction wording (especially the self-critique note — Aegis may want to soften or sharpen it).

---

## 2. Items NOT touched by this review

To keep scope bounded — for Aegis to confirm by exception:

- **§1.4 Option C "event-first with materialized views"**: still pending Aegis adjudication (PROVISIONAL per §A.5 tag). Lyra recommendation stands; no Nimbus objection in the arch supplement.
- **§2 real-time tiers (T-Live / T-Near / T-Batch)**: cross-point (b) cleared — Lyra self-amend in product §2 with framing note. No Aegis adjudication needed (decision recorded).
- **§4 historical back-fill (synthetic=true flag)**: Lyra recommendation stands; Nimbus arch §5 hybrid bootstrap-plus-steady-state aligns; no conflict. No amendment needed.
- **§5 L1 concrete flows**: Lyra recommendations stand; Nimbus arch §1 + §4 supply the events the L1 flows require. No conflict.
- **Nimbus arch §1, §2, §4, §5, §6, §7, §8, §9, Appendix A**: no Lyra cross-points objecting; all stand as authored.
- **Lyra product §3 multi-project recommendation (Option B "project_id NOT NULL everywhere")**: stands; aligns with Nimbus arch §3 Option A (same conclusion via different framing). No amendment needed beyond the migration-number correction.

If Aegis sees something in this NOT-touched list that warrants adjudication, please flag in the same pass.

---

## 3. Open question for Aegis (single item)

**Is Nimbus reciprocal review still required?**

Per Aegis 2026-05-09 evening directive: "Lyra writes cross-point comments in Nimbus's arch supplement; Nimbus writes reciprocal in Lyra's product supplement." Lyra completed side 1 (commit `14b0289`) and reserved a §Review block at the end of the product supplement for Nimbus.

With decision-2-corrected resolving cross-point (c), the most contentious item, in advance — is Nimbus's reciprocal review still required, or does this consolidation doc replace it?

**Lyra recommendation**: still required. Nimbus may have substantive notes on Lyra's §1.4 (event-first projection design), §1.5 (Supervisor IM lifecycle two-phase), §4 (back-fill synthetic flag), §5 (L1 flows event-routing). Those are outside the 7-amendment register above and deserve Nimbus's eye. If Nimbus has nothing to add, a short "no cross-points raised" note in the §Review block closes the loop.

**Aegis decision needed**: confirm Nimbus reciprocal still expected, or replaced by this doc.

---

## 4. Adjudication checklist for Aegis

Aegis adjudicates each item in one pass by writing a verdict line directly under it in this doc:

- §A.1 — APPROVED / MODIFY / REJECT
- §A.2 — APPROVED / MODIFY / REJECT (+ confirm the residual micro-correction about "before B1 enters Flux verify")
- §A.3 — APPROVED / MODIFY / REJECT (+ confirm Option C event-first projection remains binding despite no schema number)
- §A.4 — APPROVED with this wording / MODIFY to: … / REJECT
- §A.5 — APPROVED with these tags: §1.3 BINDING, §1.4 …, §1.5 …, §1.6 … / MODIFY / REJECT
- §A.6 — APPROVED with this wording / MODIFY / REJECT
- §A.7 — APPROVED with this wording / MODIFY / REJECT
- §3 open question — Nimbus reciprocal STILL REQUIRED / REPLACED BY THIS DOC

After Aegis verdicts, Lyra (and Nimbus where actor=Nimbus) commit the agreed amendments inline in the source docs, citing this consolidation file as the authority.

---

## 5. What this consolidation file is NOT

- Not a re-derivation of either supplement.
- Not a new design proposal.
- Not a packet (no implementation scope; no Flux verify; no Lyra acceptance).
- Not a substitute for the Nimbus reciprocal review unless Aegis explicitly says so under §3.

---

*Consolidated by Lyra · 2026-05-09 late-evening · 7 amendments + 1 open question · Aegis adjudicates in one pass · Nimbus may add inline note (e.g., "concur with §A.6 wording" or "modify §A.7 retraction wording") before Aegis pass — Lyra will commit any Nimbus addition without re-issuing.*
