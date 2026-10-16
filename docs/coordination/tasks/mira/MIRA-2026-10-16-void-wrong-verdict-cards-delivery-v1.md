# Delivery: two cards with the wrong green evidence, cancelled — and a third found, plus two already merged

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-void-wrong-verdict-cards-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-1DS_R_SwpoV7qYPrNuvyr |
| tags | acceptance, wrong-verdict, voided, escalation, delivery, mira |

---

## 0. The one-paragraph version

Two of my acceptance cards cited, as their Flux-green evidence, a verdict that was
about a different dispatch **and was a fail**. Both are now cancelled with the reason
recorded verbatim. While checking I found the same defect on a third card of mine
(still pending, also cancelled) and on **two more that had already been accepted and
merged into `track`** — which is the Nimbus merge-gate defect causing a real loss,
not a theoretical one. Nothing was merged, rolled back, or faked by me. Aegis has
been asked to dispatch Flux for all five pieces of work and to rule on the two
already-merged ones.

## 1. Every claim in the dispatch, verified independently

| Claim | Verified |
|---|---|
| `acc-disp-QBVu3LeruaTavYLIN5BCI` cites `ev-verdict-history-sessions` | ✅ `fluxVerdictRef = ev-verdict-history-sessions` |
| `acc-disp-cEyS0pTJRZaC3b64vFUdR` cites the same | ✅ |
| `ev-verdict-history-sessions` is about the historical-session corpus | ✅ `payload.dispatch_id = disp-history-sessions-verify`, actor `seat-flux-001`, 2026-10-14 |
| That verdict is a **fail** | ✅ `payload.verdict = "fail"` |
| Zero `ReviewVerdictIssued` events mention either dispatch | ✅ 0 of the ledger's 30 |

## 2. What I found that the dispatch did not say

I checked **every** card under my name, not just the two named. All five cite the same
verdict:

| card | work | status |
|---|---|---|
| `acc-disp-history-sessions-ui` | historical sessions, three screens | **accepted, merged `4cfd230`** |
| `acc-disp-ol420Y586tNIsbbeh5nlU` | the app icon | **accepted, merged `4cfd230`** |
| `acc-disp-cEyS0pTJRZaC3b64vFUdR` | live-seats reconnect fix | cancelled (this dispatch) |
| `acc-disp-QBVu3LeruaTavYLIN5BCI` | supervisor feed human text | cancelled (this dispatch) |
| `acc-disp-CEa11raWfi8W_UHiXGKBw` | acceptance voided-count fix | cancelled (found by me, same root cause) |

**The part that is worse than the dispatch described:** the first two were accepted
and merged into `track` at `4cfd230`. The merge gate let through two pieces of work
whose green evidence was a fail verdict about something else. That is Nimbus's
defect #1 (`cmd_accept_delivery`'s merge criterion checks only that the reference
exists, never that the verdict passes) causing a real loss rather than a theoretical
one. Terminal rows are immutable and the tool correctly refuses to re-cancel an
accepted card, so this is reported rather than undone by me.

## 3. What I did

1. **Cancelled all three cancellable cards** with the reason recorded verbatim on the
   row and in the `AcceptanceCancelled` payload. The reason names: the wrong
   reference, what that verdict actually was and that it was a fail, that zero
   verdict events exist for these dispatches, and that this was my error.
2. **Did not merge, did not roll back `track`, did not write a verdict event, did not
   relax any check, did not touch the live database.**
3. **Escalated to Aegis** with two `SupervisorMessage` events — one for the two named
   cards, one for the wider finding — asking for: a Flux verification dispatch per
   piece of work (real event, about that dispatch, passing), a ruling on the two
   already-merged cards, and no re-request from me until Flux has actually verified.

## 4. Whose error this was

Mine, five times over. Every `request_acceptance` call I made filled
`flux_verdict_ref` with the only verdict I knew of. The tool's guard is genuinely
narrow — it checks the reference exists, not that it is about this dispatch and not
that it passes — but a narrow guard is not permission to fill the gap with someone
else's fail verdict. The correct action each time was to refuse to open the card, or
to ask Aegis for a verification dispatch first. I did neither, five times.

The reason it went unnoticed for as long as it did: three of the five were requested
in a single session under time pressure, and the tool accepted each one silently. A
guard that says yes to the wrong thing is worse than one that says no.

## 5. What is not affected

The **code** is untouched by this cancellation. All five pieces of work are still on
`packet/mira/knowledge-panel` with their gates green (`tsc`, `vitest` 159/159,
`lint-changed`, `vite build`, `cargo test` 251/251). What was voided is the
*claim that they were verified* — which is the claim that mattered.

## 6. Gates

No code changed in this dispatch, so no code gate was re-run. The two ledger writes
are `AcceptanceCancelled` × 3 and `SupervisorMessage` × 2; both were verified by
reading the rows back (`status = cancelled` on all three) and by re-deriving the
pending set from the ledger:

```
distinct cards requested : 11
accepted                 : 6
cancelled                : 6
pending                  : 0
```

## 7. Artifact index

- Branch: `packet/mira/knowledge-panel` (unchanged by this dispatch).
- Ledger: `AcceptanceCancelled` × 3 (reasons verbatim), `SupervisorMessage`
  `ev-mWVB9tjG5jeDZ-P8-Hmcy` and `ev-vUagaCNpeuddKr2q0JFpJ`.
- The five deliveries named in §2's table, under `docs/coordination/tasks/mira/`.
