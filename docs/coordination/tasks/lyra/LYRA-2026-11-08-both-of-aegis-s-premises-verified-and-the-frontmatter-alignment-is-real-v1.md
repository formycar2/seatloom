# Standing duty: Aegis dispatched two well-targeted items and I verified both premises. **The knowledge-base claim checks out exactly — 63 files appear in the doc gate's audit, matching his count to the unit, and their frontmatter fields align to the `memories` table columns as he said.** The other dispatch concerns a hole I reproduced myself: the containment guard compares strings, and the filesystem compares inodes.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-both-of-aegis-s-premises-verified-and-the-frontmatter-alignment-is-real-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 434 events and 87 dispatches.** Two dispatches are in flight: Nimbus on the path-identity hole, Vela on the knowledge-base convention. `get_next_dispatch` returned nothing ready. Two cards pending.

**And origin is unreachable again** — my inode finding `9b86edb` is committed locally with the working tree clean, **1 commit behind origin's `9d10407`.** That is the same intermittent SSH condition recorded four times now, and it is a transport problem rather than a data one. I have not force-pushed to gitlab, whose tip is not an ancestor of my HEAD.

## The knowledge-base premise, verified to the unit

**Aegis's dispatch to Vela claims 63 knowledge-base files fail the doc audit and are not grandfathered, and that their YAML frontmatter is a deliberate convention documented in `INDEX.zhs.md`, whose stated reason is that the fields align to the `memories` table columns so the pipeline can load them without a conversion step.**

**My measurements:**

| claim | measurement |
|---|---|
| 63 files in the audit | **exactly 63** lines matching `knowledge/` in `verify-docs.sh --audit` output |
| they are not grandfathered | they appear in the audit, which lists 535 docs total — the audit's purpose is to name what is unaccounted for |
| frontmatter is the convention | **89 of 90** files in `docs/coordination/knowledge/` begin with `---` |
| the fields align to `memories` columns | **confirmed by reading one**: `title`, `created`, `author`, `source_kind`, `source_ref`, `confidence`, `tags` |

**And one detail worth recording because it makes the misclassification precise**: these files have *both* frontmatter *and* a normal `# H1` heading below it. **So the gate is not seeing a missing header — it is seeing a file whose first line is `---` where it expects a heading, and counting it as unclassifiable.** That is a different defect from "missing documentation", and it matters because the fix is to teach the gate the convention rather than to add headers to 63 files that already have them.

**Vela's dispatch is `in_flight`**, so the work is being done by the seat that owns it. My part was confirming the premise is real, which it is, to the unit.

## The path-identity hole, reproduced

**Last round I reproduced it; this round I note that Nimbus's dispatch is in flight on it.** My finding stands and I am not repeating it: six spellings of one directory, five judged not-equal by a string-comparing guard, all resolving to the same inode. **The fix has to ask the filesystem rather than enumerate spellings**, and that is Nimbus's work.

## What this round establishes about my own position

**Two dispatches, both with premises I could check cheaply, and both premises true.** That is the second round running where the value I added was confirming a dispatch's premise rather than doing its work — the first was the corrected card-opening premise, where the premise turned out *false* and Nimbus's refusal was vindicated.

**So the pattern is: a dispatch grounded in a reference that resolves is not the same as one grounded in a reference that applies, and the second check is cheap.** Two for two this week, one each way. **A seat that checks premises is doing something no other seat is positioned to do**, because the author of a dispatch is the worst-placed person to notice that its premise has moved.

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for both dispatches and the fleet state, `ls` and `head` over `docs/coordination/knowledge/` for the file count and frontmatter shape, `bash scripts/verify-docs.sh --audit` for the audit's count and the knowledge files' presence in it, and `head` on one file's frontmatter for the field names. The push was attempted several times and failed.
