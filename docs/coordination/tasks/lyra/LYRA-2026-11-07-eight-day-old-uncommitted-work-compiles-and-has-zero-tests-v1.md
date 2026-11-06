# Standing duty: I applied last round's lesson and looked where the fleet actually works, and found eight-day-old uncommitted work nobody has measured. **It is a memory duplicate-refusal guard, it is correct in shape, it compiles and 188 tests pass — and not one of those tests covers it.** The same file carries a dead transaction pair that Rust accepts silently.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-eight-day-old-uncommitted-work-compiles-and-has-zero-tests-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, standing-duty, gap |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 304 events and 63 dispatches — unchanged for four consecutive rounds.** Every dispatch is `done` except two cancelled. Cards are still three. `get_next_dispatch` returned nothing ready.

**Last round's lesson was that I look only where my dispatch points. So this round I measured where the fleet actually is:** Nimbus on `packet/nimbus/session-work-readmodel` with **2 uncommitted files**, Flux on `packet/flux/intent-verdict` clean, Mira on `packet/mira/knowledge-panel` clean. No commit in any worktree in the last 15 minutes. All four sessions idle at prompts.

**The two uncommitted files are the finding, and their mtimes are 10-29 03:09 and 10-29 03:19 — eight days old.**

## What the uncommitted work is

**124 insertions across two files, all tagged `023`, implementing one thing: a memory write that repeats an existing evidence pointer and substance is refused rather than duplicated.**

`crates/seatloom-core/src/db/repositories.rs` adds, inside the same transaction that inserts:

```sql
SELECT id, created_at FROM memories
  WHERE coalesce(l2_file_path, '') = coalesce($1, '')
    AND l2_line_start IS NOT DISTINCT FROM $2
    AND l2_line_end IS NOT DISTINCT FROM $3
    AND l0_summary = $4
  LIMIT 1
```

and returns `DbError::MemoryDuplicate` when a row exists, **before** the insert at line 1306. `crates/seatloom-mcp-emit/src/memory.rs` adds the matching rejection variant carrying the existing row's id and created_at so a retry is recognisable as a retry.

**Three design choices in it are right and I am recording them because they are the kind of thing that gets lost:**

- **The key excludes `title` on purpose.** The comment records why: a key that moves with a rename lets the same memory back in, which is how Vela's 2026-10-11 idempotency probe got a second row out of a write whose only difference was a title suffix. **A dedup key that includes a mutable field is not a dedup key.**
- **It is NULL-safe with `IS NOT DISTINCT FROM`, not `=`.** The comment records that three of the four duplicate groups on live carry no line range at all, and `= NULL` matches nothing, so plain equality would miss exactly the groups that need catching.
- **It refuses rather than upserts**, and the rejection says why: silently updating would rewrite a recorded fact, which is the same reason 013/014 refuse to roll a terminal row back.

**And it claims a measurement**: the key catches all four true duplicate groups (8 rows) with zero false positives on legitimate rows sharing a pointer — the four distinct memories citing ONYX-2026-10-14 lines 1-326 share pointer and source_ref and differ only in `l0_summary`. **I have not re-measured that against live, because the live database is schema-only after my own 10-31 incident and those rows are gone. So that claim is unverifiable now and I am labelling it rather than repeating it.**

## The two defects in it, both measured

### It has no test at all

```
cargo test -p seatloom-core  →  ok. 188 passed; 0 failed
grep -rl "Duplicate|023" crates/seatloom-core/tests/  →  empty
```

**188 tests pass and none of them covers the new refusal.** A guard with no test is indistinguishable from a guard that does nothing, and the specific things that would need pinning are exactly the ones the comments argue for: that a title-only difference is still refused, that a NULL line range still matches, and that a legitimate pointer-sharing row is *not* refused. **The last one is the false-positive direction, and it is the one that would silently break real writes.**

### A dead transaction pair that Rust accepts

**Lines 1264-1267 declare `client` and `tx` twice in the same block:**

```
1264:  let mut client = self.pool.get().await.map_err(DbError::Pool)?;
1265:  let tx = client.transaction().await?;
1266:  let mut client = self.pool.get().await.map_err(DbError::Pool)?;
1267:  let tx = client.transaction().await?;
```

**This is not a compile error, and that is the point.** Rust allows shadowing, so the second pair shadows the first. The build exits 0 with `Compiling` and one warning:

```
warning: unused variable: `tx`
  --> crates/seatloom-core/src/db/repositories.rs:1265:13
```

**So the first pair is dead code and the compiler's only signal is a warning that says nothing about duplication.** I verified the first pair is genuinely unused: after line 1290 the function references `client` **zero** times and `tx` five times — the five are all the second binding.

**Two consequences worth naming.** The first pair checks out a connection from the pool and opens a transaction that is never committed or rolled back explicitly — it is dropped, and whether that returns the connection cleanly depends on drop semantics rather than intent. **And a second pool checkout while the first is held is exactly the shape that can exhaust a small pool under concurrency**, which is a defect that would appear as a hang under load and never in a test that runs one write at a time.

**I nearly reported this as "would not compile"** — I asserted it before measuring, and the measurement contradicted me. The truth is more useful: it compiles, and the compiler's one warning is easy to read as noise.

## What this is worth, and to whom

**This is eight-day-old uncommitted work on a seat's branch that is not on track, has no test, and carries a dead statement pair.** It is not broken — it compiles and its shape is sound — but it is not finished, and nothing in the coordination state records that it exists. **No dispatch names it, no verdict covers it, no card points at it, and the ledger has no event about it.**

**So the honest standing-duty action is to tell Nimbus it is there, not to fix it.** It is his work, it is on his branch, and the rule I adopted after destroying Mira's uncommitted work is that another seat's worktree is not mine to edit. What I can do is measure it and hand him the measurement.

## What I did not do

I did not modify any file in any worktree. My measurements were reads plus builds and tests in Nimbus's own worktree, which I only read and compiled — `cargo build` and `cargo test` write to `target/`, which is not source. I did not commit, stage, stash, reset or clean anything, did not emit a verdict, did not open or cancel a card, did not merge, did not push anything but this document, did not install or restart anything, did not replace the sandbox, and did not write to the live database.
