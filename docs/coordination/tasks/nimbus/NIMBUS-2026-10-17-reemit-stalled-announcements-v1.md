# Delivery: my half of the re-emission — one edge released, one structurally not mine

[Nimbus -> Aegis] The dispatch `disp-reemit-stalled-announcements` asked for two
re-emissions. I did the one that is mine. The other is **structurally not mine**, and
that is a finding, not an omission.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-17-reemit-stalled-announcements-v1 |
| status | delivered, half done |
| author | nimbus |
| date | 2026-10-17 |
| version | v1 |
| to | aegis |
| dispatch | `disp-reemit-stalled-announcements` |
| contract | `LYRA-2026-10-17-next-round-plan-v1` §5.2 slice 1 |
| branch | `packet/nimbus/harness-pi` |
| tags | ledger, re-emission, dependency-edge, actor-identity, boundary |

---

## 1. The defect, measured before touching anything

Both edges and both dispatches, straight from live:

| dispatch | seat | status before | dep_kind | dep_value | target exists? |
|---|---|---|---|---|---|
| `disp-blocked-on-whom-ui` | seat-mira-001 | `blocked` | `event` | `ev-blocked-on-whom-dto-landed` | **no** |
| `disp-first-screen-verify` | seat-flux-001 | `blocked` | `event` | `ev-first-screen-default-tab-landed` | **no** |

And what actually exists:

| event | actor | emitted |
|---|---|---|
| `ev-blocked-on-whom-dto-delivered` | seat-nimbus-001 | 2026-10-16T11:03:27 — **mine, wrong id** |
| `ev-vrg-SFOoMv9BBj2jx_uSG` | seat-mira-001 | 2026-10-16T10:17:41 — **Mira's, server-minted** |
| `ev-blocked-on-whom-dto-landed` | — | absent |
| `ev-first-screen-default-tab-landed` | — | absent |

Exactly as the dispatch described. No re-measurement was needed or done.

## 2. What I did

Emitted `ev-blocked-on-whom-dto-landed`, `type=ArtifactCreated`, with `event_id` set
**explicitly**. The payload carries `reissue_of` (`ev-blocked-on-whom-dto-delivered`),
`reissue_reason` (mechanical), `original_emitted_at`, `not_a_second_verification`, and
the **entire original payload restated verbatim** — files, gates, branch, commit,
`proved_twice`, `three_states`, `does_not_assert`, `the_dto_addition`,
`read_at_read_time`, `no_invented_producer`, `measured_against_live`. Plus
`what_i_got_wrong`, naming my own error rather than burying it.

Mirrors Flux's correct pattern on `ev-NFPr-zN36by-i9SE-0FPH` (reissue_of
`ev-0idwakl17abEU75LNED24`).

**Measured result:**

```
disp-blocked-on-whom-ui|seat-mira-001|ready|event|ev-blocked-on-whom-dto-landed|1
```

`blocked` → `ready`. The edge releases on **existence** — `referent_exists`
(`dispatch.rs:681`) resolves a `dep_kind='event'` edge with
`SELECT 1 FROM canonical_events WHERE id = $1`, and `gate_qualified` returns `true`
vacuously because `is_gate_id` is false for this id (it neither ends in `-live` nor
contains `-live-`).

## 3. 🔴 The other half is structurally not mine — this is the finding

**`ev-first-screen-default-tab-landed` cannot be emitted by me honestly.**

`actor_ref` is resolved from the server's `SEATLOOM_ACTOR_REF`, **never from a
parameter** — `dispatch.rs:2044` and the tool's own schema ("Never invented"). Proven
empirically, not read off code: I passed **no** `actor_ref` when emitting my
re-emission, and the row came back `seat:seat-nimbus-001`.

So if I emitted Mira's event, the canonical `actor_ref` column would read
`seat-nimbus-001` for **Mira's announcement of Mira's slice**. Any consumer reading
`actor_ref` — the authoritative attribution, and the one the merge gate and every
verdict-naming check rely on — would conclude Nimbus announced Mira's work. That is a
false statement in the canonical ledger, and it is **not reversible**: an emitted
event is a recorded fact.

Lyra's plan §5.2 slice 1 says it plainly: *"**Each seat** re-emits **its own**
already-delivered announcement."* The dispatch to me listed both because it was
describing the slice; the slice is per-seat by design.

**I did not do it, and I did not paper over the gap.** The dispatch's done definition
says "both events emitted"; **that is not met, and I am not claiming it is.**

**What unblocks it, cheapest first:** Aegis dispatches Mira
(`disp-reemit-stalled-announcements`-shaped, or a direct ask). She emits
`ev-first-screen-default-tab-landed` with `event_id` explicit, `reissue_of`
`ev-vrg-SFOoMv9BBj2jx_uSG`. `disp-first-screen-verify` then flips on its own, exactly
as mine did.

**The alternative I rejected:** emit it myself with a payload disclosing "re-emitted by
Nimbus on Mira's behalf". The disclosure would be honest in the payload while
`actor_ref` still said me — a lie with a convenient shape, which is the exact defect
class this whole round has been about. If Aegis wants that path anyway, it is his call
and it should be his dispatch, not my improvisation.

## 4. The lesson, and what I actually got wrong

The ledger already carries it verbatim and it has now happened **twice in one chain**:
a dispatch that names an event id must be emitted with that id, because the dependency
edge recognises the id and not the artifact id. `complete_dispatch` closes a dispatch
whether or not its required announcement was emitted under the required id, and
nothing detects the mismatch.

**My specific error, named rather than generalised away:** I *chose* the announcement's
id (`-delivered`) because it described the event, instead of **reading it off the edge
that would consume it**. The id an event needs is not the id that describes it — it is
the id its consumer names. And I had the edge in hand: `disp-blocked-on-whom-ui`'s
dependency was written before I announced, and I never checked my own announcement
against it.

**The detection gap, which is the real defect:** no tool compares a dispatch's required
announcement against what was actually emitted. `complete_dispatch` has no such guard.
Two slices sat dead for a day and nothing surfaced it — same shape as §0 of Lyra's
plan: *a check that passes while the thing it stands for is absent.*

## 5. Gates run

| Gate | Result |
|---|---|
| live read-only queries (edge, dispatch, event existence) | consistent with the dispatch's description |
| re-emission emitted with explicit `event_id` | `ev-blocked-on-whom-dto-landed` |
| edge target existence after emission | `1` |
| `disp-blocked-on-whom-ui` status after emission | `blocked` → **`ready`** |
| `disp-first-screen-verify` status | still `blocked` (its edge's target is still absent) |
| live writes other than the one event | **none** |
| merges | **none** |

## 6. Known limits, stated rather than hidden

- **The dispatch's done definition is half met.** One of two events emitted; one of two
  dispatches flipped. The other is blocked on seat identity, not on effort.
- **I did not re-measure anything**, per the hard rules. The original measurement stands
  and is restated, not repeated.
- **`disp-first-screen-verify` is still `blocked`** and will stay that way until Mira
  emits her half. Reported, not hidden.
- **Nothing was merged and no live row was written** beyond the single event the
  dispatch asked for.
- **The detection gap is still open.** This dispatch fixed two symptoms; the missing
  guard in `complete_dispatch` is untouched and is a real slice for someone.

## 7. Forward recording (D5 Phase 2 Lane B)

Two memories on topic `topic-announcement-id-contract`, both
`source_kind=canonical_event`, `source_ref` = `ev-blocked-on-whom-dto-landed`,
`confidence=quoted`:

| Memory | Kind | Why it passes the three-way test |
|---|---|---|
| `mem-sD0B3GjYAJOqOBkKu8vDU` | decision | An announcement's id must be read off the consuming edge, not chosen. Changes a concrete future action (query `dispatch_dependencies` before emitting any announcement); the ledger carries the general lesson but not this actionable form; evidenced by the blocked edge and by my own wrong id. |
| `mem-63WlxZjLChecCekkjXS9v` | decision | A re-emission is per-seat because `actor_ref` is resolved server-side from `SEATLOOM_ACTOR_REF`. Changes how I answer a dispatch that asks for another seat's announcement; evidenced by the tool schema plus the empirical check that my no-actor_ref emission still came back as my own seat. |

No third memory: the missing guard in `complete_dispatch` is a gap, not a decision, and
it is named in §4 and §6 for whoever picks it up. No preference was touched — Mr Zhang's
corrections are Aegis's to record.
