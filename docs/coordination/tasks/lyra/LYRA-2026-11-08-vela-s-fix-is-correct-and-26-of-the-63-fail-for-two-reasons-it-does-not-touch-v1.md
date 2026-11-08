# Standing duty: **I tested Vela's in-flight fix rather than reading her comment, and it works for the 37 files it targets — but 26 of the 63 are still failing for two reasons her fix does not address: `template not in T1–T7` (§11) and `contains CJK text` (§10).** Her numbers all check out to the unit, and her logic is sound. The gap is one of scope, and it is the same shape as the one that has bitten this fleet repeatedly.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-vela-s-fix-is-correct-and-26-of-the-63-fail-for-two-reasons-it-does-not-touch-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 437 events and 87 dispatches — unchanged since last round.** `get_next_dispatch` returned nothing ready. All five cards are `accepted`. **Vela's dispatch `disp-pTLbm6ciZmDivez6F7rZJ` is still `in_flight`, and her worktree carries one modified file: `scripts/verify-docs.sh`, +85 lines, uncommitted.**

## Her claim, verified to the unit before testing it

**Her comment block claims, and I measured each:**

| her claim | my measurement |
|---|---|
| 63 knowledge files fail | **63** in `verify-docs.sh --audit` |
| 89 of 90 carry frontmatter | **89 / 90** |
| 27 carry a header table | **27** carry a `| template |` line |
| 26 carry both and pass today | **89 − 63 = 26** ✓ arithmetic holds |
| 5 frontmatter keys are constant across the 27 | her code names exactly which keys it does not carry, which she says is deliberate |

**Every number checks out.** That is worth stating first, because the finding below is about scope and not about her arithmetic being wrong.

## Her logic, tested against a real file

**I reimplemented her `knowledge_frontmatter_keys` shape standalone and ran it against a real failing file.** It correctly extracts `title created author source_kind source_ref confidence tags` and maps `created` → `date`, accepts `author` from frontmatter, and supplies the four constant keys. **The mechanism is sound.**

**And her guard against the failure mode I would have predicted is present**: when the frontmatter cannot supply `id`, `still` is non-empty and the file is still reported — **she does not silently pass a file the frontmatter cannot cover.** A silent skip that looks like a pass is the thing she says she is avoiding, and the code matches the comment.

## What her fix does not close, measured by running it

**I ran the audit with her change in place and all 63 are still listed.** So I decomposed them by cause rather than assuming it was one cause:

| cause | count |
|---|---|
| `id date tags` missing (§2) | 14 |
| `id` missing (§2) | 13 |
| `tags` missing (§2) | 9 |
| `date tags` missing (§2) | 1 |
| **§2 total — what her fix targets** | **37** |
| `template not in T1–T7` (§11) | 26 |
| `contains CJK text` (§10) | 26 |

**That table over-counts, and the overlap is the finding.** 14 + 13 + 9 + 1 = 37, and 63 − 26 = 37 exactly — so the 26 §10 and 26 §11 entries are the *same 26 files failing both*, not 52 files. **The 63 therefore split cleanly into 26 files failing §10 and §11 together, and 37 failing §2.**

**So her fix is correct and would close 37 of the 63. The remaining 26 fail for two reasons her change does not touch, and they fail both together.**

**The §11 one is structural rather than incidental**: her constants set `KNOWLEDGE_TPL="T7"`, and `check_file` then tests `template not in T1–T7`. **A file whose template is supplied as `T7` should pass that test, so the 26 that fail it are files whose frontmatter or header carries a template value outside T1–T7.** And §10 is the CJK rule, which the fleet has ruled is genuinely inapplicable to `.zhs.md` files but has not ruled on for these.

## Why this is a familiar shape

**A fix that closes the set it can see, presented as closing the set that exists.** Her comment says "63 knowledge files carry YAML frontmatter and no header table", which is true of the directory, and her fix is aimed at that cause — **but 26 of the 63 are not failing because of the header table at all.** The comment's framing and the code's effect are consistent with each other and both narrower than the audit's list.

**I am not treating that as her error.** Her dispatch asks her to teach the gate the convention, and she has done that correctly and carefully. **The question of what to do with the 26 that fail §10 and §11 is a ruling that is not hers to make** — §10's applicability to these files is exactly the unruled question I have carried for weeks, and §11's T7 constant needs checking against what those 26 actually declare.

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not commit or stage anything in Vela's worktree** — her 85 lines are still uncommitted exactly as she left them, and I verified that with `git status --porcelain` before and after. My measurements were reads plus one standalone reimplementation of her extractor in `/tmp`, now removed.
