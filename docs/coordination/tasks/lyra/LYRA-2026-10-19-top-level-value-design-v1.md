# SeatLoom top-level value design — value the operator can perceive, not capability the product claims

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-19-top-level-value-design-v1 |
| status | delivered |
| author | lyra |
| date | 2026-10-19 |
| version | v1 |
| tags | value-design, operator-perceivable, first-screen, roadmap |
| depends_on | LYRA-2026-10-19-whole-product-audit-v1, LYRA-2026-10-19-top-level-design-v1, docs/coordination/ROADMAP.md |

## 1. What this document is

The top-level design answered **who is responsible for what**. This document answers the question
that decides whether any of it matters to the only paying user:

> **When Mr. Zhang opens SeatLoom, what does he get that he cannot get anywhere else?**

It is organised by value he can perceive, not by feature, layer or tab. Every value below carries
three things a roadmap entry must carry and most lack:

- **The perceivable change** — what he sees or does differently, in his words.
- **The proof of presence** — the measurement or artefact that shows the value exists *today*.
- **The proof of absence** — what it looks like when the value is missing, so a fake cannot pass.

Roadmap alignment: ROADMAP §2 already states the governing principle — *every milestone ships a
user-perceivable capability, and pure plumbing is a sub-task inside a milestone, never a milestone*.
§2b's acceptance lens says a rung is done only when the capability is perceivable. This document is
that lens applied honestly to the current product, and it corrects §2b where the rungs have drifted
from what he can actually perceive.

## 2. The value thesis

**SeatLoom's value to Mr. Zhang is not "a window into terminals". It is that the work his team does
becomes something he can inspect, question, and decide on — without asking anyone.**

Three consequences follow, and they discipline everything below:

- **Terminals are not the product.** The mirror shows a process happening. It does not show what
  was decided, what was produced, or what needs him. He already told us the mirror is transitional.
- **Counters are not progress.** A number without a declared population, scope and as-of time is
  decoration. He has seen several and concluded the product was lying.
- **Capture is not value until it is reachable.** An event nobody can open, from a decision nobody
  can see, is inventory. The ledger is the means; inspection and decision are the value.

The one-sentence value proposition, in the form he can check:

> **每一次派活、每一个结论、每一次他点头或打回,都在产品里留得下、点得开、对得上。**

## 3. The five values he can perceive — and only five

Five, because a value that is not perceivable is a hope, and a value that duplicates another is
marketing. Each is stated as *before → after*, with the evidence that distinguishes them today.

| # | Value | Before SeatLoom (his pain) | After (what he perceives) | Status measured at `track@cfda740` |
|---|---|---|---|---|
| **V1** | **看一次就知道全队在干什么** | 6 个 tmux 会话挨个看,还得记得谁是准 | One screen answers who is on what, who is stuck, what is queued | **Partially real.** Dispatch page groups by seat with live status and polls a real table. Its acceptance section does real decisions. Two competing progress views still count different objects. |
| **V2** | **点任何一样东西,看懂它为什么这么做** | 结论散在聊天、记忆和脑子里 | Click a row — a dispatch, an event, a card, a past session — and read the ordered facts: what it was, who did it, when, what it produced, what blocked it, where the evidence is | **Partially real.** Lineage read model built and verified against the live database; session↔work join exists as a view and read model. The UI that renders it is not integrated, so he cannot yet click into it. |
| **V3** | **只需他点头的事,自己走到他面前** | 他不知道有东西在等他;等他知道时已经晚了 | Pending things that only he can decide arrive with what they are, what they affect, and what happens either way | **Partially real.** Acceptance cards do this for delivery merges, with a plain-language how-to-verify. Product rulings, scope questions and operator-only prerequisites have no structured pending state — they exist only as messages. |
| **V4** | **他说过的话和做过的活,对得上** | 「我上次怎么说的」没有答案 | Any present claim resolves to the past record that produced it — the original sentence, the commit, the line range | **Partially real.** History replays archived sessions with source paths and line anchors; knowledge carries provenance. No screen yet walks from a current decision back to the sentence that caused it. |
| **V5** | **产品说的和产品做的,是同一件事** | He cannot tell which pages are real | No page claims more than it measures; a failed read says so and names the next step | **Repaired in the working tree, unverified in the build he runs.** Provenance now depends on the read; failed hydration is explained rather than filled with fixtures; four local-only forms say they cannot store. Not confirmed installed. |

V5 is deliberately a value, not hygiene. He has said "很多模块都是坏的" — and the reason he cannot
tell which is the absence of V5. It is the value that makes the other four trustworthy, which is
why it is listed last but ranked first in §6.

## 4. What each value is NOT — the boundaries that keep it honest

A value definition that cannot fail is not a value. Each has an explicit exclusion, because the
commonest way this product has lost trust is by claiming the adjacent thing.

- **V1 is not terminal monitoring.** A seat typing furiously with nothing dispatched is not progress.
  A seat idle at a terminal holding an in-flight dispatch is not idle. The value is governed work
  state, with the terminal available as detail.
- **V2 is not search.** Finding a file is not understanding why a decision was made. The value is the
  ordered chain of facts about one object, with every "we do not know" stated as such.
- **V3 is not a notification feed.** A message is not a decision obligation. Most blocked rows belong
  to seats or Aegis; deriving his obligations from every blocked dispatch would be the same lie in a
  new coat. The value is obligations that are *only* his, made explicit.
- **V4 is not a log.** A chronological list is not a resolved chain. The value is traversal: from the
  claim in front of him back to the record that supports it, both directions.
- **V5 is not green badges.** A badge that does not depend on the read it describes is decoration.
  The value is that a failure is visible *and names the next step*, and that an incomplete capability
  says so instead of offering a button.

## 5. Where each value lands in the layered design

Mapping to the five responsibility layers, so this value view and the architecture view cannot
drift apart. No layer is introduced or removed here.

| Value | Primary layer | What the layer must guarantee for this value |
|---|---|---|
| V1 whole-team work | **C** (interpretation) presented by **E** | One metric contract per meaning, scope and as-of time. Competing formulas forbidden. |
| V2 click-to-understand | **B** (record/evidence) queried through **C** | Facts ordered, evidence resolvable, unknown stated not filled. No fabricated joins. |
| V3 only-his decisions | **D** (governance) interpreted by **C** | Explicit target, choices, decider, pending state, consequence. Message ≠ obligation. |
| V4 claim-to-record | **B** with **C** traversal | Every displayed claim resolves to its supporting record or is marked inferred. |
| V5 truthfulness | **E** reads **C**/**B** honestly | Failure, empty, stale and incomplete are four different states, never merged. |

## 6. Priority: what to make perceivable next

Ordered by value-per-unit-of-risk, not by engineering elegance. Each row states what he will newly
perceive, which is the only acceptance test that matters here.

| Rank | Make perceivable | He newly perceives | Why this rank | Cost / risk |
|---|---|---|---|---|
| **1** | **V5 in the build he runs** | Opening the app he either sees the real fleet or is told exactly why not | Every other value is untrustworthy until this holds. It is the value he has asked for most often and least received. | Rebuild/reinstall plus a click test. No new feature. |
| **2** | **V3 beyond merge acceptance** | Rulings and operator-only prerequisites appear beside acceptance cards, with owner and consequence | He currently learns these by reading messages or being told. Highest-cost failure mode is a silent miss. | Medium contract work: a structured pending-decision object with owner/status/evidence. No new navigation required. |
| **3** | **V2 rendered end to end** | Clicking a row opens its ordered facts and evidence, in the product | Built and verified in the backend; the missing half is presentation. Without it, lineage is inventory. | Medium UI work against existing commands; needs installed-build verification. |
| **4** | **V1 one meaning** | One progress number he can trust, with the other populations labelled rather than competing | Currently several screens answer "progress" with different objects. Must not silently merge dispatch and workitem. | Medium semantic contract; label distinct populations until links exist. |
| **5** | **V4 two-directional traversal** | From any current claim back to the sentence, commit or line that produced it | This is the compounding value — the one that makes the other four accumulate instead of reset. | Larger; depends on ranks 2–4 and on topic/assignment modelling. |

Not ranked, deliberately: capability library, budget/cost, cross-project fleet aggregation. They may
be valuable later, but none is perceivable today and none should displace a repair of trust.

## 7. Roadmap corrections this value view forces

The ROADMAP's §2b rungs were written before these measurements. Three corrections, stated as
corrections rather than rewrites, because §2b is the acceptance lens and must not overstate:

1. **UM0 "one screen for the whole team" was never perceived as described.** It described terminals
   plus IM. He experienced the mirror as transitional and told us so. The rung's perceivable
   content is now V1 — governed work state — with the mirror retained as detail. UM0 as written
   should be marked superseded rather than done or available.
2. **UM2 "progress at a glance" is not a rung while competing counters exist.** A work-graph over a
   mock feed is not progress at a glance. The rung's precondition is rank 4 above: one declared
   metric contract. Until then UM2 is blocked on truth, not on rendering.
3. **No rung yet names V5.** Trustworthiness is absent from §2b entirely, which is why a page could
   carry a fixed 72% health figure and a render clock labelled as last sync while the rungs reported
   progress. V5 should become an explicit rung with a falsifiable acceptance clause.

These are recommendations to the roadmap owner. This document does not edit the ROADMAP.

## 8. Operator decisions — separate from the engineering

Kept apart because they are his, not ours, and mixing them into a plan is how a seat accidentally
decides them:

- **Default scope on open:** whole-team with project identity retained, or stay project-local. The
  current anchor is project-local. V1 is only fully perceived with his preferred scope.
- **How much internal bookkeeping to show:** whether memory/topic activity appears on the first
  screen or stays one click away. V1's usefulness depends on his answer.
- **What counts as "only he can decide":** the boundary between a ruling he must make and one Aegis
  or a seat should make. V3's definition is incomplete without it.
- **Which duplicated entry remains visible,** and ownership of the current shortcut, before any
  navigation is removed or reassigned.

## 9. Evidence basis and limits

Baseline `track/infra-foundation@cfda740`, shared integration worktree, clean. Sources: the
delivered capability audit (`9d3dbf7`), the top-level design (`274fad9`), ROADMAP §2/§2b, and
source reads of the surfaces named. No live-database access, no rendered-app click, no new IPC
measurement was performed for this document.

What is measured: dispatch grouping and polling with real data; acceptance decisions with real
commands; history replay with source paths and line anchors; knowledge probing real commands;
provenance now depending on the read; four local-only forms disclosing they cannot store.

What is explicitly not claimed: that the installed build contains the V5 repairs; that lineage is
reachable in a rendered screen; that any structured product-ruling object exists; that the five
values are complete for all time. Each value's status is a statement about the working tree at one
commit, not about what he is running.

No code, live-database, gate, merge, acceptance card or sub-task issuance. Routing and integration
remain Aegis's; default presentation and navigation remain the operator's.
