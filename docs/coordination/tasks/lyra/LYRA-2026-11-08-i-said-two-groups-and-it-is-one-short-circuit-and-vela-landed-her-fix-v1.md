# Standing duty: **I corrected my own measurement from last round. I said the 63 knowledge files split into 26 failing §10 and 37 failing §2 as two mutually exclusive groups. They are not two groups — with the exemption off, all 63 fail §2 first and `return`, so 26 files never reach the CJK rule at all.** Vela's own test measures this per-rule and states it correctly. **Her work has now landed as `4927d34`, and the split is exactly 37 + 26.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-i-said-two-groups-and-it-is-one-short-circuit-and-vela-landed-her-fix-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 437 events and 87 dispatches.** `get_next_dispatch` returned nothing ready. All five cards `accepted`.

**Vela's dispatch is still `in_flight`, but her work has landed**: worktree `packet/vela/washing-pipeline` is now **clean**, carrying `4927d34` "feat(docs): teach the doc gate the knowledge-base convention" on top of a merge of `track/infra-foundation`.

## The correction, stated before anything else

**Last round I wrote that the 63 split cleanly into 26 failing §10 and §11 and 37 failing §2, and I called them mutually exclusive.** The §11 part was wrong — there is no §11 failure — but the deeper error was the framing.

**What is actually true, and I measured it by neutralising the exemption the way her test does:**

| exemption | header-rule failures | CJK-rule failures |
|---|---|---|
| **OFF** | **63** | **0** |
| **ON** | **37** | **26** |

**So the 26 are not a separate group. They are 26 of the 63 that, with the exemption off, fail §2 first and hit `return` before the CJK rule is ever evaluated.** Her comment in the test says exactly this and explains why a file-level before/after is misleading: it prints `63 -> 63` for a change that took the header rule from 63 to 37.

**"Count the rule, not the file" is the right instruction and I did not follow it last round.** I counted files by their reported reason, which is a reading of the short-circuit rather than of the underlying state.

## Why this matters beyond my arithmetic

**A short-circuited check is invisible.** Those 26 files have carried CJK text the whole time and no instrument ever said so, because a broken header stopped the evaluation first. **That is the same shape as "a check that dies before evaluating is indistinguishable from a check that passes", one layer down: the check did not die, it was never reached.**

**And it means the exemption does not merely clear a false positive — it exposes 26 previously-hidden CJK failures.** Her test names this as a pass condition, which is the honest framing: the change is correct and it makes the remaining work visible rather than hiding it.

## What her test measures, which is better than what I did

**`scripts/test-knowledge-convention.sh`, 225 lines, 8 assertions, currently 8 passed 0 failed.** What it does that I did not:

- **neutralises the exemption by rewriting `KNOWLEDGE_DIR` to a path that never matches**, then counts per rule — so the before/after is about the rule, not the file
- **refuses a file with neither form**, and a file whose frontmatter cannot supply `id` — both directions
- **confirms a complete header table still satisfies the gate with odd frontmatter** — the two forms are alternatives, not cumulative
- **confirms the exemption does not leak outside `knowledge/`**
- **re-runs the pre-existing `test-verify-docs-exemptions.sh`** so this change cannot silently break the other exemption

**And while I was measuring, I watched her fix a real defect in her own test**: it first reported `BEFORE=0 AFTER=0` because it counted files instead of rules, then a syntax error, then it reached 8/8. **A test that reports 0 -> 0 for a change that took a rule from 63 to 37 is a test that cannot fail, and she replaced it with one that can.**

## What remains, and it is not hers

**37 files fail §2 because their frontmatter genuinely cannot supply `id`, `date` or `tags`.** Those need either the header table added or the frontmatter key added — real work on real files, and a decision about which form is canonical.

**26 files fail §10 for CJK text, and that is the unruled question I have carried for weeks**: whether §10 applies to knowledge files at all, given that `.zhs.md` is exempt and the directory is documented as a deliberate convention. **Her dispatch asked her to teach the gate the convention, and she did. What to do about the 26 is a ruling that belongs to the operator.**

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not commit, stage or edit anything in Vela's worktree** — I verified it clean before and after, and every probe I ran was a copy under `scripts/.lyra-*.sh` that I removed. My measurements were reads plus patched copies of `verify-docs.sh` in her worktree that I deleted.
