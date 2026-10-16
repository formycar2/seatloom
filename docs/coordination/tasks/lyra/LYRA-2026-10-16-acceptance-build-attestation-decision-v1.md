# Decision — the acceptance card and the build must provably arrive together, measured not promised

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-16-acceptance-build-attestation-decision-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-16 |
| version | v1 |
| to | aegis |
| depends_on | docs/coordination/ROADMAP.md (2026-10-16 snapshot, §3 and §5), scripts/prepare-verification-build.sh |
| tags | decision, acceptance-gate, build-attestation, product-truth, heartbeat |

Raised by the LYRA heartbeat protocol (§3.1 and §3.2), not by a dispatch. Facts, findings and
decisions are kept in separate sections on purpose.

---

## 1. Facts (measured, each reproducible)

| # | Fact | How it was measured |
|---|---|---|
| F1 | Two acceptance cards are pending the operator's click **right now**: `acc-disp-QBVu3LeruaTavYLIN5BCI` (the supervisor activity feed showing a bare English type name on ~80% of rows) requested 07:37 UTC, and `acc-disp-cEyS0pTJRZaC3b64vFUdR` (Live Seats demanding a Connect on every menu switch) requested 07:23 UTC | `query_ledger`, `AcceptanceRequested` payloads, both on `packet/mira/knowledge-panel` |
| F2 | The running SeatLoom is **byte-identical** to `seatloom/target/release/bundle/macos/SeatLoom.app`, built 15:05:07 local | `shasum -a 256` on `/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri` and the worktree's release bundle: both `3bf9037e…` |
| F3 | That bundle was built from `ui/dist` produced at **15:03:20** from `track/infra-foundation` @ `8a2d733` (15:20:13) | `stat` on `ui/dist`; `git log -1 track/infra-foundation` |
| F4 | **Neither fix is on `track/infra-foundation`.** `ui/src/app-v2/lib/eventText.ts` and `ui/src/stores/useLiveSeatsMirrorStore.ts` do not exist there; both exist on `packet/mira/knowledge-panel` | `git cat-file -e <ref>:<path>` on both refs |
| F5 | Both fixes were committed **after** the running build existed: `18aa027` at 15:20:58 and `62e535a` at 15:35:28, against a binary built at 15:05:31 | `git show -s --format=%ci`; `stat` on the release binary |
| F6 | **No build on this machine contains either fix.** The newest `seatloom-tauri` release binary anywhere is 15:05:31 (main checkout); Mira's worktree's is 14:38:34 | `find … -path '*/release/seatloom-tauri'`, sorted by mtime |
| F7 | Verified at the artifact level, not by inference: the fix's marker strings are **PRESENT** in Mira's branch `ui/dist` (rebuilt 15:44:31) and **ABSENT** from the main checkout's `ui/dist` (15:03:20) that produced the running app. The control string, a long-standing UI label, is present in both, so the method discriminates | `grep -rlF` over both dists |
| F8 | `scripts/prepare-verification-build.sh` step 4, whose own header calls it "prove this build really carries the work" — **cannot report for any frontend change.** It greps the binary with `strings`, and the frontend is embedded compressed: on this very binary `strings` finds a long-standing UI label **0 times**, while the dist it was built from contains it | `strings -a <binary> \| grep -cF <label>` → 0, against `grep -rlF <label> ui/dist` → present |

**What F1–F8 add up to.** The operator is being asked to Accept two fixes that are not in the
program he is running. If he follows either card he will still see bare English type names in the
supervisor feed and still be asked to reconnect on every menu switch. He will either conclude the
fix failed, or click Accept on something he never saw.

### 1.1 The timeline, because it changes what the fix is

All times local. This is the difference between "the rule was ignored" and "the rule had nothing
enforcing it", and only one of those is fixable by a gate.

| Time | What happened |
|---|---|
 10-15 17:52 | `packet/vela/washing-pipeline` merged to track (`dea224b`) |
 14:16 | `prepare-verification-build.sh` committed (`a505558`) — today's new rule, in prose |
 14:33 | Mira's branch tip is `0e1a184`-era; her worktree bundle is built at **14:38:11** |
 14:44:41 | `e48ef7e` — track synced into Mira's branch before the icon card |
 **14:54** | **Five cards granted by the operator.** Verified below: those were sound |
 14:54:27 | `4cfd230` — Mira's branch merged to track |
 **15:05:07** | A release bundle is built from **`track/infra-foundation`** and installed at 15:05:32 — correct, because every then-pending card's work was on track |
 **15:20:58** | `18aa027` — the Live Seats fix, committed to **Mira's branch** |
 **15:23** | Its acceptance card is opened |
 **15:35:28** | `62e535a` — the supervisor feed fix, committed to **Mira's branch** |
 **15:37** | Its acceptance card is opened |

**The five 14:54 grants were sound, so this is not systemic.** Two were for
`packet/vela/washing-pipeline`, whose work was on track since 10-15 and therefore inside the
14:38 bundle through the sync; three were for Mira's own work, which was on her branch. Each was
verified against a build that carried it.

**What actually went wrong is narrow.** At 15:05 the running program was *replaced* with a build of
track — the right thing to do, since every pending card's work was on track. Two fixes were then
committed to a seat's branch *after* that replacement, and cards were opened for them **without
anyone running the build script again**. At no point did any build on this machine contain either
fix. Nothing in the process could notice, because nothing checks.

## 2. Findings (inference, labelled as such)

**2.1 The rule was violated 32 minutes after it was written, and nothing noticed.** Aegis's
roadmap §3 records today's new rule — *"the card and the build arrive together"*, and that a card
arriving without its build is the same as not telling him — and §5 asserts that all four pending
cards are verifiable with the currently installed program. Both were true when written. Two cards
then arrived whose builds were never prepared (F6). §5 is now stale for exactly those two.

**2.2 The one control that exists to catch this is a check that cannot report.** F8 is the
structural finding, and it is the same family as the two checks Aegis fixed today: `fleet-status`
treating a wrapper process as the app window, and `must-have` sitting in `$3` so it was always
empty while the script printed a warning urging that one be supplied, and continued. **Three checks in one day that look
like they verify and cannot.** The `strings`-on-a-compressed-binary check is the worst of the
three, because it is the one standing between the operator and accepting an unverifiable card.

**2.3 The gate validates everything except the thing the card is about.** `request_acceptance`
checks the plan's shape, that the verdict exists, that the branch contains the base, and that the
operator identity is set. It never asks whether a build carrying the fix exists, let alone whether
it is the one running. A card can be perfectly well-formed and still describe a program the
operator does not have.

**2.4 Why this is the highest-value gap and not a detail.** The acceptance card is the product's
only human-facing surface: a seat does work, a card appears, he clicks. That loop just failed
silently in the direction that matters most — it would have had him approve two things that do not
work. He has already said, in his own words, that many modules are broken. Two of the modules he
would have hit are these two cards.

## 3. Decisions (mine, as PO)

**D1. These two cards must not be clicked as they stand.** Not a preference — a fact: the thing
they ask him to verify is not in any build on this machine (F6). The correct next action is to
build from `packet/mira/knowledge-panel` and re-offer, or to withdraw and re-issue. Which of those
is Aegis's call, not mine.

**D2. The next round's milestone: before clicking an acceptance card, you can see whether the
program in your hands actually carries this fix.**
The acceptance request carries a **measured build attestation** — branch, commit, when the build
was made, and a marker string proven present in the artifact the operator is actually running. The
card renders it. No attestation, or an attestation that does not verify, and the card says plainly
that this build does not carry the fix and the build must come first. It is user-perceivable by
construction: he reads it on the card, before he clicks.

**D3. The attestation's marker must be proven in the artifact he is running, not in a build
directory.** F8 kills `strings`-on-the-binary for anything frontend. The proof has to run against
something that can actually be searched — the built `ui/dist` the bundle was produced from, or the
decompressed embedded asset — and it must fail closed. A check that cannot fail is not a check.

**D4. The roadmap's §5 claim must name its build.** "Verifiable with the currently installed
program" is exactly the
kind of sentence that goes stale silently. From now on that claim carries the install timestamp
and the commit it was built from, so staleness is visible rather than assumed away.

**D5. What I am deliberately not deciding.** Whether to cancel or re-issue the two cards, and
whether the acceptance gate itself changes — those are Aegis's (gate conditions and merges are
his, per the heartbeat). Nothing here asks the operator to choose anything; the only thing he
needs to do is not click two cards yet, and that is a defect report, not a decision.

## 4. What I checked and did not find broken

So this is not "the gate is broken": the plan-shape validation, the verdict reference, the
branch-contains-base guard, the operator-identity requirement (which correctly refuses a merge
with no identity rather than inventing one), and the ledger's record of every request and grant
all work. The gap is one missing measurement, not a broken mechanism.

## 5. Out of scope

Changing the acceptance gate's conditions (Aegis's). Cancelling or re-issuing the two cards
(Aegis's, with Mira as the requesting seat). Merging `packet/mira/knowledge-panel` to track.
Touching the live database.
