# Delivery — Flux self-verification gap: the acceptance gate class rule

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | lyra-disp-TkbJSDe-G2l1Hh4g5QSut-delivery |
| status | completed |
| author | lyra |
| date | 2026-10-27 |
| version | v1 |
| tags | acceptance, gate-class, verification, flux, decision, delivery |

## Decision

Closed the undefined square with a **content-based** rule, not a seat-based one.
The governing guard (`flux_verdict_ref` must name a `ReviewVerdictIssued` about
the dispatch being accepted) is correct and unchanged. What was missing is a rule
for the seat that cannot be verified by the mechanism it serves.

Three classes, declared in the dispatch payload at issue time
(`acceptance_class`), each defined by the work's content rather than by who ran
it:

| Class | Content | Card? | Evidence the card carries |
|---|---|---|---|
| `implementation` | any diff to code, config, schema, migration, seed, script, CI input | Yes | independent GREEN verdict from a seat other than the actor, plus the operator running the PO plan |
| `verification` | reads state, issues a verdict, commits no diff | Yes | a pass verdict about this dispatch, plus its own evidence |
| `review` | investigates or reports, commits no diff, opens no card | No | its durable artifact is the work product |

Plus two negative rules stated so they cannot be misread: **no `verdict_exempt`
exists**, and **a seat never issues the verdict for its own dispatch**.

Aegis's inclination (C primarily, A as backstop) was adopted in its narrow,
content-defined form: read-only-with-no-diff opens no card, because there is no
merge candidate. The blanket exempt list was rejected — a whitelist's boundary
moves, and "exempt" hides a gap rather than stating it. Aegis's option B (Aegis
re-checks every verdict) was rejected as a serialization point on the lane that
must stay independent.

## Files

- `docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md` (new) — the decision memo: verdict GO, option-by-option reasoning, the per-dispatch classification of all eight blocked Flux dispatches, enforcement status, known limits.
- `docs/coordination/ENGINEERING_STANDARDS.md` (edit) — new section `## Acceptance gate class`, version row `v1` → `v2`, date to 2026-10-27, one `tags` addition, changelog line under the header, and two new follow-ups (6: enforce the class in code; 7: Lyra releases the eight once classified). Id stays `engineering-standards-v1` so existing `depends_on` / document references keep resolving.

## Verification

- `git diff --check` on the staged files — clean.
- `bash scripts/verify-docs.sh HEAD` — `verify-docs: changed docs vs HEAD — all conform.` (both new/changed docs pass the header, naming, English and template-directory ratchet).
- `bash scripts/verify-docs.sh --audit | grep engineering-standards` — still absent, i.e. the standard remains conforming rather than grandfather-covered.
- Both artifacts use `write`/`edit`, no code-modifying heredoc, no `cd <absolute path> &&`.

## Escalation / notes for Aegis

- The rule is **prose until Follow-up 6 lands**. An issuer can still mis-declare
  a code-changing dispatch as `review` and dodge the card; detection is manual
  today (the commit list is right there). Code enforcement (reject classless
  requests; forbid a verdict naming its issuer's own dispatch; add
  `engineering_standards` to the T7 allow-list in `DOCUMENT_TEMPLATES.md` §11.1
  and its two Rust mirrors) is Follow-up 6, not done here.
- **No verdicts were issued and no dispatch was declared passed.** The decision
  removes the structural impossibility; the eight verdicts are still owed per
  dispatch by an independent seat. §3 of the memo maps each one.
- **"Independent" means a different seat, not a different human.** Two dispatches
  from one operator session can still be self-consistent. Stated as a known
  limit in memo §6 rather than solved.
- The two Rust allow-list mirrors named in the standard's debt note
  (`artifact_store.rs`, `document_parser.rs`) do not exist at the paths given in
  this worktree. Not investigated — out of scope for a governance dispatch.
- Shared coordination logs (`MEMORY.md`, `docs/coordination/memory/`) untouched
  per the seat's completion protocol.
- Unrelated pre-existing dirty files (the two 2026-10-19 documents,
  `.saih/`, the untracked `.zhs.md` acceptance doc) were left exactly as found
  and are not in this commit.
