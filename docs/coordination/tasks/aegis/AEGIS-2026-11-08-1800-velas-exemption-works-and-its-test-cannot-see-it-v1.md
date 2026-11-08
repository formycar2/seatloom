| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-1800-velas-exemption-works-and-its-test-cannot-see-it |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, docs-gate, measurement, stuck-dispatch |

# Vela's exemption works, and its own test is structurally incapable of seeing that

## What I measured

Vela advanced the work after my last report. The `author` defect I found is fixed — it is now a
declared measured default rather than something the frontmatter must supply. I verified the
exemption does what it claims:

| population | before | after |
|---|---|---|
| knowledge files failing the **header** rule | 63 | **0** |

That is the fix, and it works.

The 63 files that still fail, fail on **other rules**:

| rule | count |
|---|---|
| CJK text in a non-`.zhs` file (§10) | 26 |
| frontmatter genuinely missing `id` | 27 |
| frontmatter genuinely missing `tags` | 9 |
| frontmatter genuinely missing `date` | 1 |

So the header problem is solved and 37 files remain that need a frontmatter key added. That is a
decision about the **convention** — whether `id`/`tags`/`created_at` are required keys — not about
the gate. I am not going to edit 37 files' frontmatter on my own judgement.

## The test cannot see its own success

`scripts/test-knowledge-convention.sh` prints:

```
knowledge files failing: BEFORE=0  AFTER=63
✗ the exemption did not reduce failures (0 -> 63)
```

`BEFORE` is measured inside `scratch_repo()`, which does `mkdir -p "$d/$KD"` — an **empty**
knowledge directory. So the before-count is structurally always zero, and the comparison is
meaningless. The change succeeded and the test reports it as a failure.

I nearly took that line at face value in the other direction too: it reads like the change is
broken, when the header part is fine. **A test whose fixture cannot produce the case it measures
will report a false result in either direction.** I hit this exact shape myself earlier tonight
when I passed an absolute path to a function whose pattern is relative.

## The dispatch is still stuck

`disp-pTLbm6ciZmDivez6F7rZJ` has been `in_flight` for **15 hours** with a `SeatDispatchDelivered`
event from 19:13 yesterday. I poked Vela again with both findings and an explicit request to close
it. I have not touched their ledger row — that is not mine to change.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Zero pending cards.
- Sandbox: 95-byte fake, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.
- Push: 12 commits local-only; the retry loop is still working.

## Needs you

- **The sandbox rebuild** is still the only thing standing between the remaining verified work and
  you.
- **Whether `id`/`tags`/`created_at` are required keys of the knowledge convention.** 37 files
  lack at least one. Either the convention requires them and the files get fixed, or the gate
  should default them too — but that second option means the gate supplies the file's identity,
  which I do not think is right and want your call on rather than mine.
