# Delivery — `disp-Tn-83tjvS_B6YgOV4dCCF` (Flux): re-verification of Mira's five 2026-10-16 slices

- **Dispatch**: `disp-Tn-83tjvS_B6YgOV4dCCF`, issued by seat-lyra-001. A re-verification: five
  slices were green-lit on an invalid basis and two of them are already in `track`.
- **Verdict**: **PASS — all five, each against its OWN dispatch's acceptance criteria, measured.**
- **Verdict event**: `ev-0idwakl17abEU75LNED24` — pre-declared id, emitted exactly as required
  (`ReviewVerdictIssued`, `verdict: pass`).

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-16-mira-five-slices-verdict-v1.md` | new — the verdict: every criterion of every dispatch, with what I measured for it |
| `docs/coordination/tasks/flux/disp-Tn-83tjvS_B6YgOV4dCCF-delivery.md` | this note |
| `.scratch/flux_mira_five_slices/` | new — the measurement scripts, so every number is reproducible |

No source file was changed. I verified in the two checkouts that hold the code — the main checkout
(on `track` @ `8a2d733`, tracked tree clean) for the two **merged** slices, and the mira worktree (on
`packet/mira/knowledge-panel` @ `93226cb`, tracked tree clean) for the three **branch** slices —
**read-only in both**: nothing committed, no tracked file touched. The live database was touched only
by read-only `SELECT`s and read-only MCP calls whose no-write property I measured.

## The background claim, verified first

All five cards carry `flux_verdict_ref = ev-verdict-history-sessions` — a `ReviewVerdictIssued` with
`verdict = fail` about the 2026-10-14 historical-session corpus, unrelated to an app icon, a
reconnect fix, a feed translation or a count. Two of the five (`acc-disp-history-sessions-ui`,
`acc-disp-ol420Y586tNIsbbeh5nlU`) were **accepted** on that basis and merged at `4cfd230`; the other
three were later voided with `feedback` naming exactly this root cause. So the green-light basis was
invalid. **The work is a separate question, and that is what I measured.**

Two facts that make "verify track, not her branch" exact:
- `git diff --name-only 09d7abb 4cfd230` = **70 files**, and the union of `51bccc5`'s and `d04f373`'s
  file lists = the same 70 (`comm` empty both ways). Nothing else rode in on the merge.
- for all 11 source files the two slices touch, `git rev-parse track/infra-foundation:<f>` ==
  `git rev-parse packet/mira/knowledge-panel:<f>` — **byte-identical**. Track's copy is the merged copy.

## The five verdicts, one line each

| # | Dispatch | Commit | In track? | Verdict |
|---|---|---|---|---|
| 1 | `disp-history-sessions-ui` | `51bccc5` | **yes** | **PASS** — 38/183/36680/OfficeBench 49/seatloom 14/cursor 14 all-undated, all re-measured live AND against the source corpus; ordering correct across all 38 projects; line numbers verified against the original file on disk; every §3.2 state present; no mock, no hardcoded flag, no new route; the card's plan is contract §6 verbatim |
| 2 | `disp-ol420Y586tNIsbbeh5nlU` | `d04f373` | **yes** | **PASS** — 5-entry `bundle.icon`, all seven icns tiers, the `.icns` md5-identical inside the release bundle with `CFBundleIconFile` pointing at it, 0 white pixels on the outer ring at every size, exactly two colours, and the actual 16×16 pixels decoded and looked at: a connected three-armed mark, not a blob |
| 3 | `disp-cEyS0pTJRZaC3b64vFUdR` | `18aa027` | no | **PASS** — state hoisted to a store that outlives the view; 7 real tests mount the view and assert zero new attach calls, byte counters surviving (`toEqual` on the whole Activity object), first-open auto-attach, no second attach, dead sessions pruned, list-failure tolerance, IM routing; her reason for rejecting the dispatch's *preferred* fix is verified in the code (`cmd_list_live_sessions` → `Vec<String>`; `cmd_attach_tmux_session` spawns an unconditional second `pipe-pane`) |
| 4 | `disp-QBVu3LeruaTavYLIN5BCI` | `62e535a` | no | **PASS** — all 16 types translated and all 16 confirmed in the vocabulary via `query_ledger`; the 9 dead branches confirmed NOT in the vocabulary; my own 200-row measurement: 153/200 (76 %) would have been bare type names, now 0; nothing invented (misses are stated); real words byte-for-byte; emoji style kept; sorting untouched; contact list untouched |
| 5 | `disp-CEa11raWfi8W_UHiXGKBw` | `f34111d` | no | **PASS** — `isDecided` knows `cancelled`, voided cards collapsed with their reason, accepted/rejected behaviour unchanged, the second instance (`DecisionResult`) fixed; the live number re-measured: **6 accepted / 5 cancelled / 0 pending**, so the section now reads 0 where the bug would read 5 |

## Gates — all run by me, none taken from the delivery

**On track** (items 1 and 2): `tsc` clean · vitest **93/93** · `lint-changed.sh 09d7abb` clean
(6 files) · `vite build` green · `cargo test --workspace` **251/0** · `clippy -D warnings` clean ·
one pre-existing `fmt` red at `src-tauri/src/main.rs:88`, which I verified is byte-identical before
and after her work (her `main.rs` diff is 14 insertions, 0 deletions).

**On the branch** (items 3-5): `tsc` clean · vitest **159/159** · `lint-changed.sh 4cfd230` clean
(10 files) and clean again per-slice against each commit's own parent · `vite build` green ·
`cargo test --workspace` **251/0** · clippy clean · the same inherited `fmt` red.

## Raised, not failed on

- **F-GATE-1 (Aegis / Lyra)** — the acceptance gate checked that the cited verdict *existed* and
  nothing else: not that it was about this dispatch, not what it concluded. Five unrelated cards
  passed on one `fail` verdict, two merged. **Not a defect in any of the five slices.** Nimbus is
  fixing the two defects; I rolled nothing back, per the dispatch's boundary.
- **F-OBS-1 (Lyra)** — contract §4's "root group last" is enforced only in the UI;
  `list_history_projects` returns `(root)` 6th of 38. The operator-facing criterion is met, so item 1
  passes, but the next consumer of that tool gets an order the contract forbids.
- **F-OBS-2 (Mira / Aegis)** — bytes arriving while the live-seats view is unmounted are not counted
  (its output listener is removed on unmount). Persistence — the actual criterion — holds.

## Not proven, stated plainly

I did not click. Everything above is compile-, test-, measured-read- or file-level; I work from a
shell and cannot see the interface. I did not run a fresh `tauri build` (I measured the bundle
already on disk and proved it holds the post-merge commands and the byte-identical `.icns`). The
supervisor-feed measurement used `query_ledger`'s 200-row ceiling, not the 400 the column loads. The
Tauri app was not launched.

## Forward recording (D5 Phase 2 Lane B)

Nothing recorded via `remember`. Every finding is either a file:line fact with a reproduced
measurement already written into the verdict doc, or a process observation aimed at Aegis/Lyra
rather than at my own future behaviour. Nothing here changes a future action of mine in a way that is
not already in the verdict doc, and nothing is a preference of Mr. Zhang's — so per the protocol's
"三条全满足才记;存疑不记", zero entries.
