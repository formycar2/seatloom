# Standing-duty finding: the memory-dedup proof fails on a stale binary, and it is the same "instrument not verified current" failure in its seventh costume

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-dedup-proof-fails-on-stale-binary-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, blocking, instrument, proof-script, build-hygiene |

## Why I ran this

I asked myself the question I owed from yesterday: **is the line healthy, not just is the record right.**
Rust on track is green — 357 passed / 0 failed. UI on track is red — 2 failed / 370 passed. Both are now
known. This document is about a third thing I found while closing out that question.

`crates/seatloom-mcp-emit/proof/memory_dedup_proof.sh` is the proof for the memory-dedup guard — the
guard that migration 023 exists to backstop, and the one whose landing script I reported as half-fixed
yesterday. I ran it.

```
$ bash crates/seatloom-mcp-emit/proof/memory_dedup_proof.sh
  first write: accepted
  rows after first write: 1
  repeat write: REFUSED
  refusal: memory tool rejected: postgres error: db error
PROOF FAILED (line 152)
```

## What it looked like, and what it was

It looked like a real defect: the guard refuses the write correctly, but the error surfaces as an opaque
`postgres error: db error` instead of the rich message the source clearly intends. That message is
excellent and it is in the source twice:

```
memory already exists at {where_} (row {existing_id}, created {created_at}) with this l0_summary —
a repeat write is refused, never duplicated. If this is a retry, the row is already there; if it is a
genuinely new record, change its l0_summary or its evidence pointer, because the title is
deliberately NOT part of the key
```

**It was not a defect. The binary was stale.**

```
$ ls -l target/debug/seatloom-mcp-emit        -> Oct 29 11:29
$ ls -l crates/seatloom-mcp-emit/src/memory.rs -> Oct 30 10:29
$ strings target/debug/seatloom-mcp-emit | grep -c "a repeat write is refused, never duplicated"
  0
```

The proof invokes `$ROOT/target/debug/seatloom-mcp-emit` — a **debug build with no build step**. The
source was updated on 10-30 (track's merge at 10:29), the binary was not rebuilt, and the guard's
`Display` arm carrying the good message simply does not exist in the binary the proof called.

After `cargo build -p seatloom-mcp-emit --bin seatloom-mcp-emit`:

```
$ strings target/debug/seatloom-mcp-emit | grep -c "a repeat write is refused, never duplicated"
  2
$ bash crates/seatloom-mcp-emit/proof/memory_dedup_proof.sh
  refusal: memory already exists at docs/x.md lines 10-20 (row mem-Co8Ec--EkOF3LarKusO8s,
           created 2026-10-30 03:36:31) with this l0_summary — a repeat write is refused, never
           duplicated. If this is a retry, the row is already there; if it is a genuinely new record,
           change its l0_summary or its evidence pointer
ALL POINTS PASS
EXIT=0
```

**All six points pass, including point 6, which drops the index and proves the SQL guard alone refuses
the repeat.** The guard is correct, the message is correct, and the proof proves it. Nothing is broken.

## Why this is worth a document anyway

**Because the failure mode is the exact one I have recorded seven times today, and this time it is
wearing a costume I had not seen: a proof that fails because the thing it proves was updated after the
thing that proves it was built.**

The seven, for the record:

| # | The error |
|---|---|
| 1 | A dead-stripped MCP binary probed for content that was never in it |
| 2 | A column-shifted read of a table |
| 3 | An empty table read without first reading the tool's rules |
| 4 | An unfetched git ref read as a commit |
| 5 | The wrong worktree, on the wrong branch (Aegis) |
| 6 | `infra/postgres/migrations/` queried for a file that lives under `schema/` |
| 7 | **A proof script calling a debug binary older than the source it is proving** |

Every one is the same sentence: **a measurement taken at a location that was not verified to be the
location the claim was about.** Number 6 and number 7 are mine within the same hour, which is the part
that should worry me: I wrote "knowing the failure mode does not immunize against it" yesterday, and
then did it again.

## The specific trap in number 7

A proof script is a *test*, and the instinct on a failing test is "the code is wrong." But a proof script
that shells out to a compiled binary is a test **of a build artifact**, and nothing in the script checks
that the artifact is current. So the failure is reported in the language of a code defect — `db error`,
a missing message — when the actual defect is in the build.

**The cost is not the wasted cycle. It is that the failure is convincing.** "The guard's error message
is opaque" is a plausible, well-formed finding that would have produced a document, a dispatch, and
possibly a fix to code that was already correct. The only thing that stopped it was checking the binary
against the source before believing the output.

## What should change, and it is small

**One line at the top of any proof that calls a compiled binary:**

```bash
cargo build -p seatloom-mcp-emit --bin seatloom-mcp-emit || exit 1
```

Six seconds, and it converts this whole class of failure from "convincing wrong finding" into
"impossible." `memory_dedup_proof.sh` is one of at least fourteen scripts in `proof/`, and I have not
audited the others for the same shape — that audit is the actual ask, and it is not mine to do unasked
because those scripts belong to the seats that wrote them.

## What I did not do

I did not edit the proof script — it belongs to whoever wrote it and I have already asked Aegis for one
script change this round. I did not "fix" the guard, which was correct. I dropped the scratch database
`seatloom_dedup` that the proof creates, and verified live is untouched by my work: the three memories
created between 03:00 and 03:16 are Flux's, written through the normal path at 03:14–03:16, and the
proof's own guard (`[ "$SCRATCH" != "seatloom" ] || exit 1`) refuses the live DB outright.
