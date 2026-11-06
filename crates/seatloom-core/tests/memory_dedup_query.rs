//! `023` memory-dedup **behavioural** tests — the ones that drive the real query.
//!
//! WHY A SECOND FILE. `tests/memory_dedup_key.rs` pins the key struct: that
//! `title` is not an input, that a NULL line range stays `None`, and that a
//! different substance is a different record. Lyra mutation-tested that file by
//! replacing **both `IS NOT DISTINCT FROM` predicates with plain `=`** — the exact
//! defect the comments argue against — and **the suite stayed green: exit 0, 10
//! passed.** She verified the mutation was present before reading the result
//! (remaining `IS NOT DISTINCT FROM` = 0, new `l2_line_start = $2` = 1), so this
//! was a real mutation that real tests did not notice.
//!
//! Why the green was not a lie but was also not a proof: the key tests never
//! touch a database. They import `memory_dedup_key` and assert on a struct's
//! fields. The SQL binds that struct's fields positionally into `$1..$4`, but
//! nothing connects the struct's shape to the query's **predicates**. So
//! `a_null_line_range_stays_null_rather_than_collapsing_to_zero` proves the key
//! keeps `None` as `None`, not that the query compares it with `IS NOT DISTINCT
//! FROM`. Both can be true while the query is broken.
//!
//! What the mutation actually does: with `= $2` and a NULL `l2_line_start`, SQL
//! compares NULL to NULL, which matches nothing. A repeat write carrying no line
//! range is therefore not found and the insert proceeds — precisely the silent
//! failure the comment names ("would miss exactly the groups that need
//! catching"), and three of the four duplicate groups are in that class. So the
//! mutation defeats the guard for the majority case.
//!
//! THE RULE THIS FILE INSTANCES. A proof that reads source proves the source; a
//! proof that calls a copy proves the copy; only a proof that drives the real
//! thing proves the real thing. Here the copy is sharper than usual because it
//! sits one line above the real thing and its fields are bound into it — which
//! is exactly why ten passing tests and an unguarded predicate are compatible.
//!
//! These tests drive `SeatloomDb::insert_memory` against a real PostgreSQL.
//!
//! HOW THESE ARE GATED, and why not the obvious way.
//!
//! A test that can skip silently is worse than no test: it reports green while
//! measuring nothing, which is the failure this whole file exists to close. But
//! the opposite choice is also wrong — making these **panic** when `DATABASE_URL`
//! is unset turns `cargo test --workspace` red for every seat without a scratch
//! database, and a gate that is red for unrelated reasons is a gate people learn
//! to ignore. That trade was measured, not assumed: the panicking version made
//! the workspace go from 188 passed to 198 passed / 7 failed, and the 7 failures
//! were all "DATABASE_URL is not set".
//!
//! So each test is `#[ignore]`d with a reason naming how to run it — which is
//! **counted** in `cargo test` output as `N ignored`, not hidden — and
//! `scripts/test-memory-dedup-query.sh` is the actual gate. The script runs them
//! with `--include-ignored` and **fails if zero tests ran**, which is the part
//! that closes the silent-skip hole: a configuration where they would all skip
//! is itself a failure, reported as such.
//!
//! ```text
//! bash scripts/test-memory-dedup-query.sh
//! ```
//!
//! WHAT THIS FILE DOES NOT CLAIM. That the key catches all four duplicate groups
//! with zero false positives on live. Those rows are gone and that claim is
//! unverifiable now rather than confirmed. These tests pin the predicate's
//! behaviour on rows they create themselves.

use deadpool_postgres::Pool;
use seatloom_core::db::connection::{create_pool, DATABASE_URL_ENV};
use seatloom_core::db::repositories::DbError;
use seatloom_core::db::SeatloomDb;

/// The project and topic the tests write under. Created by `seed_scope`.
const PROJECT_ID: &str = "dedup-probe-project";
const TOPIC_ID: &str = "dedup-probe-topic";

/// A pool for the scratch database, or a **panic** explaining how to supply one.
///
/// This deliberately does not return `Option` and does not skip. The previous
/// convention in this crate — `async fn scratch_pool() -> Option<Pool>` — made
/// every database test `#[ignore]`d or silently skipped, and a silently skipped
/// test is green while measuring nothing.
async fn probe_pool() -> Pool {
    let url = match std::env::var(DATABASE_URL_ENV) {
        Ok(u) if !u.trim().is_empty() => u,
        _ => panic!(
            "{} is not set. These behavioural tests drive the real query and need a real \
             PostgreSQL. Set it to a SCRATCH database, never the live `seatloom` one, e.g.\n  \
             {}=postgresql://seatloom:seatloom@localhost:5432/seatloom_nimbus_dedup_proof \
             cargo test -p seatloom-core --test memory_dedup_query -- --test-threads=1",
            DATABASE_URL_ENV, DATABASE_URL_ENV
        ),
    };

    // Refuse the live database by NAME, not by inference. `dbname == "seatloom"`
    // catches the exact string; `starts_with("seatloom?")` catches a URL whose
    // query string is glued to the name, which the substring check alone misses.
    let dbname = url.rsplit('/').next().unwrap_or("");
    if dbname == "seatloom" || dbname.starts_with("seatloom?") {
        panic!(
            "REFUSING the live `seatloom` database for the 023 dedup tests — they insert and \
             delete rows. Point {} at a scratch database.",
            DATABASE_URL_ENV
        );
    }

    let pool = create_pool().unwrap_or_else(|e| {
        panic!(
            "could not create a pool from {}={url}: {e}",
            DATABASE_URL_ENV
        )
    });

    // Refuse a database that has no `memories` table: the schema is not applied,
    // and a failure here would otherwise surface later as a confusing error
    // about a missing relation.
    let client = pool
        .get()
        .await
        .unwrap_or_else(|e| panic!("could not connect to {url}: {e}"));
    let has = client
        .query_one("SELECT to_regclass('public.memories') IS NOT NULL", &[])
        .await
        .expect("could not query to_regclass");
    if !has.get::<_, bool>(0) {
        panic!(
            "{url} has no `public.memories` table — apply infra/postgres/schema/ first \
             (scripts/apply-db.sh)"
        );
    }
    pool
}

/// Create the scope rows the memories reference, and clear any memories left by
/// an earlier run so each test starts from a known state.
async fn seed_scope(pool: &Pool) {
    let mut client = pool.get().await.expect("pool");
    let tx = client.transaction().await.expect("tx");
    tx.execute(
        "INSERT INTO projects (id, name, created_at, worker_budget_tokens, supervisor_budget_tokens) \
          VALUES ($1, $2, now(), 1000, 1000) \
          ON CONFLICT (id) DO NOTHING",
        &[&PROJECT_ID, &"dedup probe"],
    )
    .await
    .expect("project");
    // Column names read from the schema, not guessed: `topics` has `opened_at`,
    // not `created_at`. The first version of this INSERT guessed, and every test
    // failed with E42703 — a fixture that fails on its own proves nothing about
    // the code under test, the same failure family as a silently skipped test.
    tx.execute(
        "INSERT INTO topics (id, project_id, title, status, opened_by, opened_at) \
          VALUES ($1, $2, $3, 'open', 'seat:seat-nimbus-001', now()) \
          ON CONFLICT (id) DO NOTHING",
        &[&TOPIC_ID, &PROJECT_ID, &"dedup probe topic"],
    )
    .await
    .expect("topic");
    // Only this project's memories, so a shared scratch database is not emptied.
    tx.execute("DELETE FROM memories WHERE project_id = $1", &[&PROJECT_ID])
        .await
        .expect("clear");
    tx.commit().await.expect("commit");
}

/// One call to the real `insert_memory`, with the probe's scope filled in.
async fn write_memory(
    db: &SeatloomDb,
    memory_id: &str,
    l0_summary: &str,
    l2_file_path: Option<&str>,
    l2_line_start: Option<i32>,
    l2_line_end: Option<i32>,
) -> Result<(), DbError> {
    db.insert_memory(
        memory_id,
        PROJECT_ID,
        TOPIC_ID,
        "a title that will be renamed", // deliberately unstable: see the rename test
        &[],
        l0_summary,
        None,
        "file",
        None,
        l2_file_path,
        None,
        "canonical_event",
        "disp-OJpQBzZ75CV9Oqk7L0DFA",
        "fact",
        "inferred",
        None,
        l2_line_start,
        l2_line_end,
        &[],
        "seat:seat-nimbus-001",
    )
    .await
}

// ─── the direction the mutation defeated ───────────────────────────────────

/// A repeat write carrying **no line range at all** must be refused.
///
/// This is the test Lyra's mutation survived. With the predicates as `= $2`, SQL
/// compares NULL to NULL, matches nothing, and the insert proceeds — so this
/// test fails, which is the point. With `IS NOT DISTINCT FROM` it passes.
///
/// Three of the four duplicate groups on live are in this class, so this is the
/// majority case, not an edge case.
#[tokio::test]
#[ignore = "needs DATABASE_URL pointing at a SCRATCH database — run scripts/test-memory-dedup-query.sh"]
async fn a_repeat_write_with_no_line_range_is_refused() {
    let pool = probe_pool().await;
    seed_scope(&pool).await;
    let db = SeatloomDb::new(pool.clone());

    let first = write_memory(
        &db,
        "dedup-null-range-1",
        "the same substance",
        Some("docs/x.md"),
        None,
        None,
    )
    .await;
    assert!(
        first.is_ok(),
        "the first write must succeed — otherwise this test proves nothing: {first:?}"
    );

    let second = write_memory(
        &db,
        "dedup-null-range-2",
        "the same substance",
        Some("docs/x.md"),
        None,
        None,
    )
    .await;
    match second {
        Err(DbError::MemoryDuplicate { existing_id, .. }) => {
            assert_eq!(
                existing_id, "dedup-null-range-1",
                "the refusal must name the row that is already there, so a retry is recognisable"
            );
        }
        other => panic!(
            "a repeat write with NO line range was NOT refused — it produced {other:?}. \
             This is exactly what replacing `IS NOT DISTINCT FROM` with `=` does: NULL = NULL \
             matches nothing, so the duplicate is let in. The key tests cannot catch this; only \
             driving the query can."
        ),
    }
}

/// The same predicate failure seen from the other side: a repeat write that
/// differs **only** in having a line range where the first had none must not be
/// treated as a duplicate. `IS NOT DISTINCT FROM` makes `None` and `Some(1)`
/// unequal, so this write is legitimate.
#[tokio::test]
#[ignore = "needs DATABASE_URL pointing at a SCRATCH database — run scripts/test-memory-dedup-query.sh"]
async fn a_null_range_and_a_real_range_are_different_records_against_the_query() {
    let pool = probe_pool().await;
    seed_scope(&pool).await;
    let db = SeatloomDb::new(pool.clone());

    let first = write_memory(
        &db,
        "dedup-none-1",
        "the same substance",
        Some("docs/y.md"),
        None,
        None,
    )
    .await;
    assert!(first.is_ok(), "first write must succeed: {first:?}");

    let second = write_memory(
        &db,
        "dedup-none-2",
        "the same substance",
        Some("docs/y.md"),
        Some(10),
        Some(20),
    )
    .await;
    assert!(
        second.is_ok(),
        "a row with NO line range and a row with a real line range are different records; \
         refusing the second would be the false-positive direction: {second:?}"
    );
}

// ─── the same predicate, with ranges present on both sides ─────────────────

/// With a line range on both writes, a repeat must be refused. This passes with
/// `=` as well as with `IS NOT DISTINCT FROM`, which is exactly why it is not
/// sufficient on its own — but it pins the behaviour the range-carrying groups
/// depend on.
#[tokio::test]
#[ignore = "needs DATABASE_URL pointing at a SCRATCH database — run scripts/test-memory-dedup-query.sh"]
async fn a_repeat_write_with_identical_ranges_is_refused() {
    let pool = probe_pool().await;
    seed_scope(&pool).await;
    let db = SeatloomDb::new(pool.clone());

    let first = write_memory(
        &db,
        "dedup-range-1",
        "the same substance",
        Some("docs/z.md"),
        Some(1),
        Some(9),
    )
    .await;
    assert!(first.is_ok(), "first write must succeed: {first:?}");

    let second = write_memory(
        &db,
        "dedup-range-2",
        "the same substance",
        Some("docs/z.md"),
        Some(1),
        Some(9),
    )
    .await;
    assert!(
        matches!(second, Err(DbError::MemoryDuplicate { .. })),
        "a repeat write with identical ranges must be refused, got {second:?}"
    );
}

/// A different line range at the same file is a different record.
#[tokio::test]
#[ignore = "needs DATABASE_URL pointing at a SCRATCH database — run scripts/test-memory-dedup-query.sh"]
async fn a_different_range_at_the_same_file_is_not_refused() {
    let pool = probe_pool().await;
    seed_scope(&pool).await;
    let db = SeatloomDb::new(pool.clone());

    let first = write_memory(
        &db,
        "dedup-range-a",
        "the same substance",
        Some("docs/w.md"),
        Some(1),
        Some(9),
    )
    .await;
    assert!(first.is_ok(), "first write must succeed: {first:?}");

    let second = write_memory(
        &db,
        "dedup-range-b",
        "the same substance",
        Some("docs/w.md"),
        Some(1),
        Some(99),
    )
    .await;
    assert!(
        second.is_ok(),
        "a different line range is different evidence, not a duplicate: {second:?}"
    );
}

// ─── the false-positive direction, driven against the query ────────────────

/// Four rows sharing a pointer and a source_ref but differing in substance must
/// all land. This is the direction that would silently break real writes, so it
/// is pinned against the real query and not only against the key struct.
#[tokio::test]
#[ignore = "needs DATABASE_URL pointing at a SCRATCH database — run scripts/test-memory-dedup-query.sh"]
async fn four_distinct_substances_at_one_pointer_all_land() {
    let pool = probe_pool().await;
    seed_scope(&pool).await;
    let db = SeatloomDb::new(pool.clone());

    let substances = [
        "the read model resolves obligations from the ledger tables, not from events",
        "a snapshot property needs a concurrent observation, not a static marker count",
        "a build failure must not be reported as a successful build",
        "the UI slice is unblocked but is kept as its own bounded slice",
    ];
    for (i, s) in substances.iter().enumerate() {
        let r = write_memory(
            &db,
            &format!("dedup-four-{i}"),
            s,
            Some("docs/ONYX-2026-10-14.md"),
            Some(1),
            Some(326),
        )
        .await;
        assert!(
            r.is_ok(),
            "substance {i} was refused as a duplicate — sharing a pointer does not make two \
             records one, and refusing a legitimate write is worse than admitting a duplicate: \
             {r:?}"
        );
    }
}

/// The rename direction, driven against the query: the same pointer and
/// substance with a different title must still be refused, because the query
/// does not select on `title` at all.
#[tokio::test]
#[ignore = "needs DATABASE_URL pointing at a SCRATCH database — run scripts/test-memory-dedup-query.sh"]
async fn a_title_only_difference_is_refused_by_the_query() {
    let pool = probe_pool().await;
    seed_scope(&pool).await;
    let db = SeatloomDb::new(pool.clone());

    // `write_memory` always uses the same title, so drive this one directly with
    // a different one to prove the title is not in the predicate.
    let mut first_ok = write_memory(
        &db,
        "dedup-title-1",
        "the same substance",
        Some("docs/t.md"),
        None,
        None,
    )
    .await;
    assert!(first_ok.is_ok(), "first write must succeed: {first_ok:?}");

    let second = db
        .insert_memory(
            "dedup-title-2",
            PROJECT_ID,
            TOPIC_ID,
            "A COMPLETELY DIFFERENT TITLE",
            &[],
            "the same substance",
            None,
            "file",
            None,
            Some("docs/t.md"),
            None,
            "canonical_event",
            "disp-OJpQBzZ75CV9Oqk7L0DFA",
            "fact",
            "inferred",
            None,
            None,
            None,
            &[],
            "seat:seat-nimbus-001",
        )
        .await;
    assert!(
        matches!(second, Err(DbError::MemoryDuplicate { .. })),
        "a write differing only in title must still be refused — the query does not select on \
         title, so a rename must not let the duplicate back in. Got {second:?}"
    );
    let _ = &mut first_ok;
}

/// A different file at the same substance is a different record, driven against
/// the query's `coalesce(l2_file_path, '')` predicate.
#[tokio::test]
#[ignore = "needs DATABASE_URL pointing at a SCRATCH database — run scripts/test-memory-dedup-query.sh"]
async fn a_different_file_is_not_refused_by_the_query() {
    let pool = probe_pool().await;
    seed_scope(&pool).await;
    let db = SeatloomDb::new(pool.clone());

    let first = write_memory(
        &db,
        "dedup-file-1",
        "the same substance",
        Some("docs/a.md"),
        None,
        None,
    )
    .await;
    assert!(first.is_ok(), "first write must succeed: {first:?}");

    let second = write_memory(
        &db,
        "dedup-file-2",
        "the same substance",
        Some("docs/b.md"),
        None,
        None,
    )
    .await;
    assert!(
        second.is_ok(),
        "a different file is different evidence: {second:?}"
    );
}
