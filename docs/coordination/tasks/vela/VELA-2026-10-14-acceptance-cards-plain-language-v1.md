# Delivery — both wash acceptance cards re-opened with operator-readable plain-language plans

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-14-acceptance-cards-plain-language-v1 |
| status | delivered |
| author | vela |
| date | 2026-10-14 |
| version | v1 |
| depends_on | AEGIS-2026-10-14-nudge-vela-reissue, AEGIS-2026-10-14-acceptance-reissue-id-param, NIMBUS-2026-10-14-acceptance-reissue-id-param-delivery-v1 |
| tags | acceptance-gate, reissue, plain-language, d2, washing |

## 1. What was asked

The Aegis nudge `AEGIS-2026-10-14-nudge-vela-reissue`: the backend seat landed the
optional `acceptance_id` parameter on `request_acceptance` (integrated in
`fac50cb`, previously stuck on its own branch — that missing integration was the
root cause of my 2026-10-14 failure), the binary was rebuilt, and my tool
subprocess was restarted with it at 13:41. The only missing step was my call:
re-open the two cancelled acceptance cards `acc-disp-wash-vela-p1` and
`acc-disp-wash-vela-p2` under their original ids, using my own §6 plain-language
plans verbatim — nobody else's wording.

## 2. Preconditions verified before calling (each checked, none assumed)

| # | Precondition | Evidence |
|---|---|---|
| 1 | Both dispatches are `done` and owned by `seat-vela-001` | `get_dispatch` for both: status/liveStatus `done` |
| 2 | Both target card ids are in `cancelled` terminal state (the guard's only re-issuable shape) | `get_acceptance_lineage`: status `cancelled`, lineage ends `AcceptanceCancelled` |
| 3 | Both Flux verdict refs exist as `ReviewVerdictIssued` in the lineages | `ev-SWfX7pNZH8PObLcGxg3pY` (p1), `ev-flux-verdict-wash-p2` (p2) |
| 4 | This process's tool surface now carries `acceptance_id` | `describeTool("mcp__seatloom__request_acceptance")` returns the parameter — the restart took; the prior process's schema did not have it |
| 5 | `packet/vela/washing-pipeline` contains the tip of `track/infra-foundation` | The base had advanced 2 commits past my earlier sync (`13b33e3`, `fac50cb`); re-merged as `bd1deb6`. The branch-contains-base guard reads the base **tip at request time**, so a sync made yesterday is not good enough today — re-synced before calling rather than assuming |
| 6 | Gates green on the merged tree | §5 below |

## 3. The two re-issues

Parameters, exactly as the nudge specified:

| field | p1 | p2 |
|---|---|---|
| dispatch_id | disp-wash-vela-p1 | disp-wash-vela-p2 |
| acceptance_id | acc-disp-wash-vela-p1 | acc-disp-wash-vela-p2 |
| branch | packet/vela/washing-pipeline | packet/vela/washing-pipeline |
| base | track/infra-foundation | track/infra-foundation |
| flux_verdict_ref | ev-SWfX7pNZH8PObLcGxg3pY | ev-flux-verdict-wash-p2 |
| how_to_verify | my §6 one-liner, verbatim | my §6 one-liner, verbatim |
| acceptance_plan | my §6 plan, verbatim | my §6 plan, verbatim |

**The submitted text is in Chinese, deliberately.** The operator (Mr. Zhang)
rejected the previous cards as unverifiable and unreadable — the entire rewrite
exists so that someone who has never read the code can follow the card. An
English "plain-language" card would defeat its own purpose. This document
therefore describes the plans in English and does not quote them; the verbatim
operator-facing strings are preserved twice over: immutably in the acceptance
rows themselves, and in git history at commits `49921ff` / `de3656a` (the
superseded Chinese delivery doc — see §6).

**Verbatim proof, transcription-free.** After the calls, both cards were fetched
with `get_acceptance_lineage` and deep-compared **inside the tool sandbox**
against the §6 source blocks, which were extracted programmatically (python,
`json.loads`) from the superseded doc rather than retyped. Result:
`plan_verbatim=true` and `howToVerify_verbatim=true` for both cards, zero field
diffs. Method note: an earlier comparison attempt that re-pasted the returned
JSON by hand produced a false mismatch and was discarded — any verification of
"verbatim" that routes long strings through a human copy step verifies the copy,
not the submission.

## 4. Results

Both re-issues succeeded on the first attempt after the preconditions were met:

| card | status now | last event |
|---|---|---|
| acc-disp-wash-vela-p1 | pending | `AcceptanceRequested` `ev-Gb4wj2MZ2qdIxkYzIpZ2j` (payload `reopened: true`) |
| acc-disp-wash-vela-p2 | pending | `AcceptanceRequested` `ev-Hag5b_KsnoAx5rMrqiaez` (payload `reopened: true`) |

The old technical plans (S0/S1/S2/S2.5 vocabulary, gate numbers) are replaced by
the operator-readable ones. The success condition that did not exist when I
closed `disp-acceptance-cards-plain-language-vela` as failed on 2026-10-14 now
exists: two cards are open, and Mr. Zhang can work through them without reading
code. Acceptance itself is the operator's action (merge into the named base via
the Tauri command) — never mine.

## 5. Gates run on this branch (after the base re-sync)

| gate | result |
|---|---|
| `cargo fmt --all --check` | pass |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | pass |
| `cargo test -p seatloom-core -- --include-ignored --test-threads=1` | 146 passed, 2 failed — see below |
| `scripts/verify-docs.sh track/infra-foundation` | was failing on this branch (my own doc, §6); fixed in this commit; now passes |

The two failing tests (`db::write_api::tests::append_event_in_tx_commits/rolls_back_*`)
are DB-integration tests that require `DATABASE_URL` to point at a reachable
**scratch** database. This environment provides none, and the only reachable
postgres is the live one, which my seat is forbidden to write to. The failure is
environmental and pre-existing at the base commit (verified: the same tests fail
on `track/infra-foundation` without my merge); CI runs them against its postgres
service. No live-DB action was taken.

## 6. Doc-gate remediation — my own debt, fixed rather than grandfathered

The 2026-10-14 plain-language delivery doc
(`disp-acceptance-cards-plain-language-vela-delivery.md`) violated the doc
ratchet three ways: no universal header (§2), a filename outside the
`ROLE-YYYY-MM-DD-<topic>-vN.md` convention (§4), and a Chinese body without the
`.zhs.md` suffix (§10). It was committed without the gate being run, so
`verify-docs.sh` has been red on this branch since `49921ff` — my process
failure, recorded here rather than papered over.

It was **born on this branch and never merged**, so it could not be added to
`.gitlab/docs-grandfather.txt`: the grandfather list is a bootstrap-time
snapshot of pre-existing debt, and the ratchet only ever tightens. It is
replaced by this conforming English document; the Chinese original remains
readable in git history, and its operative content now lives immutably in the
acceptance rows. One dangling reference results: Nimbus's delivery doc lists the
old path in its `depends_on`; that reference is now historical.

## 7. Washing-class statistics — not applicable

This dispatch washed no corpus: no LLM calls, no records extracted, nothing
pruned, no records dropped for missing L2 line numbers, and no redaction scan
was required (the submitted plan text was authored by me today and contains no
secrets, keys, or intranet details).

## 8. What the operator does next

Open SeatLoom's knowledge panel and work through the two cards
(`acc-disp-wash-vela-p1`, `acc-disp-wash-vela-p2`) in order: the topic tree and
the no-secrets check first, then the three sampled memory cards. Each card
states what to look at, what counts as a pass, and what counts as a rejection —
in the operator's own language, which was the point of the whole rewrite.

## 9. Forward recording (completion-protocol step 4)

The memory write path is live in this process (the restarted binary carries the
018-typed `remember`), so the step that failed on 2026-10-11/12 ran for real.
Four records, each anchored to a line range of this document with a recomputed
sha256 body digest (topic `topic-aegis-fleet-operations`):

| kind | confidence | memory id | anchor |
|---|---|---|---|
| fact | quoted | `mem-2_59_9ZWX5n_iPRdnCksD` | §4, lines 70-84 |
| decision | inferred | `mem-7VblF7PPdsBFe008sFaSj` (1 `refines` edge to `mem-kLUAxUaT5aZSGJFeoIWtY`) | §2, line 30 |
| failure | quoted | `mem-95t4x7668ADPSC0jcjAGQ` | §6, lines 103-111 |
| unresolved | quoted | `mem-slns0oN1TwafNjhkRfhAb` | §5, lines 95-101 |

No records were dropped for missing L2 line numbers; no corpus was washed; no
redaction scan was needed (nothing but my own English prose and tool output was
written). No live-DB action was taken beyond the sanctioned memory write path.
