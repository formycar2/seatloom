//! `close_topic`'s atomicity claim, pinned against a real transaction failure.
//!
//! WHY THIS EXISTS. `close_topic` claims in its own doc that "the event and the
//! row move in ONE transaction: there is no state where the ledger says closed
//! and the row says open." Flux verified the function and gave it a pass, but
//! wrote in his honest summary that this claim — the unit's core claim — had no
//! test pinning it, and listed it explicitly as NOT verified.
//!
//! Aegis is right that a verdict saying "the core claim is unpinned" is not a
//! verdict the operator should accept on. So this test pins it.
//!
//! WHAT THE TEST ACTUALLY PROVES, and what it does not.
//!
//! It proves the part that is observable and that the mutation covers: if the
//! work AFTER the UPDATE fails, the row is not left modified. That is a real
//! failure mode — the second half of a two-statement transaction can fail for
//! ordinary reasons (a constraint, a bad value, a dead connection) — and
//! "no partial state" is exactly the property that catches it.
//!
//! It does NOT prove that PostgreSQL rolls back on a hard crash, and no test
//! here can: that needs a killed connection mid-transaction, which is not
//! reproducible from inside the process without deliberately corrupting the
//! pool. Flux was precise about this distinction and so is this file. A test
//! that claimed to prove crash-rollback would be a lie; a test that proves the
//! observable half is not.
//!
//! THE MUTATION THAT MUST BE RED. Delete the `UPDATE` and keep the event: then
//! the event exists for a topic that never changed state. Delete the event and
//! keep the UPDATE: then the row says closed with nothing in the ledger saying
//! so. Both are observable in the assertions below, so both mutations fail.
//!
//! Requires a scratch PostgreSQL with migrations applied. Refuses the live DB.
//!
//!   DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_tclose2 \
//!     cargo test -p seatloom-core --test topic_close_atomicity -- --include-ignored

use deadpool_postgres::Pool;
use seatloom_core::db::connection::{create_pool, DATABASE_URL_ENV};
use seatloom_core::db::SeatloomDb;

const PROJECT: &str = "tclose-atomic";

/// Each test owns its own project row. The two run in PARALLEL, and a shared
/// project id makes them race on `reset()` — the second one to run deletes the
/// first one's rows mid-assertion, which shows up as a phantom failure.
const PROJECT_OK: &str = "tclose-atomic-ok";

async fn scratch_pool() -> Option<Pool> {
    let url = std::env::var(DATABASE_URL_ENV).ok()?;
    let dbname = url.rsplit('/').next().unwrap_or("");
    if dbname == "seatloom" || dbname.starts_with("seatloom?") {
        eprintln!("REFUSING live `seatloom` DB — set DATABASE_URL to a scratch DB");
        return None;
    }
    let pool = create_pool().ok()?;
    let client = pool.get().await.ok()?;
    // The test needs 026's TopicClosed in the event-type vocabulary. Without it
    // the event append is refused by the CHECK and the failure the test is
    // looking for happens for the wrong reason.
    let row = client
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM pg_constraint \
                              WHERE conname = 'canonical_events_event_type_vocab' \
                                AND pg_get_constraintdef(oid) LIKE '%TopicClosed%')",
            &[],
        )
        .await
        .ok()?;
    if !row.get::<_, bool>(0) {
        eprintln!("026_topic_close not applied (no TopicClosed in the vocab) — skipping");
        return None;
    }
    Some(pool)
}

/// Wipe everything this test could touch, so a re-run starts from nothing and
/// counts are absolute rather than "more than before".
async fn reset(pool: &Pool, project: &str) {
    let c = pool.get().await.expect("client");
    for sql in [
        format!("DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='{project}')"),
        format!("DELETE FROM canonical_events WHERE project_id='{project}'"),
        format!("DELETE FROM memories WHERE project_id='{project}'"),
        format!("DELETE FROM topics WHERE project_id='{project}'"),
        format!("DELETE FROM projects WHERE id='{project}'"),
    ] {
        c.execute(&sql, &[]).await.expect("clean");
    }
    c.execute(
        "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
         VALUES ($1,$1,NOW(),1024,1024)",
        &[&project],
    )
    .await
    .expect("project");
}

/// The observable state of the ledger for this project.
async fn closed_events(pool: &Pool, project: &str) -> i64 {
    let c = pool.get().await.expect("client");
    let n = c
        .query_one(
            "SELECT count(*) FROM canonical_events \
              WHERE project_id=$1 AND event_type='TopicClosed'",
            &[&project],
        )
        .await
        .expect("count events");
    n.get::<_, i64>(0)
}

async fn topic_state(pool: &Pool, topic_id: &str) -> (String, bool) {
    let c = pool.get().await.expect("client");
    let row = c
        .query_one(
            "SELECT status, closed_at IS NOT NULL FROM topics WHERE id=$1",
            &[&topic_id],
        )
        .await
        .expect("topic row");
    (row.get::<_, String>(0), row.get::<_, bool>(1))
}

#[tokio::test]
#[ignore = "needs a scratch PostgreSQL with migrations applied"]
async fn a_close_that_fails_midway_leaves_neither_the_row_nor_the_event() {
    let Some(pool) = scratch_pool().await else {
        return;
    };
    reset(&pool, PROJECT).await;
    let db = SeatloomDb::new(pool.clone());
    let topic_id = format!("tca-{}", std::process::id());

    db.open_topic(
        &topic_id,
        PROJECT,
        "atomicity probe",
        None,
        "seat:nimbus-001",
    )
    .await
    .expect("open");

    // THE SETUP THAT MAKES THE SECOND HALF FAIL. A deferred constraint
    // trigger on `topics` fires at COMMIT time, after the UPDATE has already
    // run and after the event has been appended. So the transaction reaches
    // `tx.commit()` with both statements done, and the commit is refused —
    // exactly the "the work after the UPDATE failed" shape the atomicity claim
    // is about.
    //
    // Why a trigger and not a bad value: a bad value would fail the UPDATE
    // itself, before any partial state exists, and would prove nothing about
    // the pairing. A constraint that only bites at COMMIT is what makes the
    // first statement visible-then-rolled-back.
    {
        let c = pool.get().await.expect("client");
        c.execute(
            "CREATE OR REPLACE FUNCTION tca_fail_at_commit() RETURNS trigger AS $$ \
             BEGIN \
               RAISE EXCEPTION 'tca: forced failure at commit time'; \
             END; $$ LANGUAGE plpgsql",
            &[],
        )
        .await
        .expect("fn");
        c.execute(
            "CREATE CONSTRAINT TRIGGER tca_guard \
               AFTER INSERT OR UPDATE ON topics \
               DEFERRABLE INITIALLY DEFERRED \
               FOR EACH ROW EXECUTE FUNCTION tca_fail_at_commit()",
            &[],
        )
        .await
        .expect("trigger");
    }

    let result = db.close_topic(&topic_id, "seat:nimbus-001").await;

    // Tear the trigger down FIRST, before any assert: if an assert fires with
    // the trigger still installed, every later topic write on this scratch DB
    // fails and the next run cannot set itself up.
    {
        let c = pool.get().await.expect("client");
        let _ = c
            .execute("DROP TRIGGER IF EXISTS tca_guard ON topics", &[])
            .await;
        let _ = c
            .execute("DROP FUNCTION IF EXISTS tca_fail_at_commit()", &[])
            .await;
    }

    let err = result.expect_err("the close must fail — the trigger refuses the commit");

    // THE OBSERVABLE STATE. Neither half survived. If the UPDATE had been left
    // in place, `status` would be 'closed' with `closed_at` set and no event
    // anywhere saying so. If the event had been left in place, the ledger would
    // say closed while the row said open.
    let (status, has_closed_at) = topic_state(&pool, &topic_id).await;
    let events = closed_events(&pool, PROJECT).await;

    assert_eq!(
        status, "open",
        "a close that failed midway must NOT leave the row modified — the row \
         says {status:?}, so the UPDATE was not rolled back"
    );
    assert!(
        !has_closed_at,
        "closed_at must not be written by a close that failed — the UPDATE was \
         not rolled back"
    );
    assert_eq!(
        events, 0,
        "a close that failed midway must NOT leave an event behind — the ledger \
         says {events} TopicClosed for a topic that is still open"
    );

    // And the error is the real one, not a swallowed success.
    // tokio_postgres' Display is just "db error" — the RAISE text lives in the
    // source, so the assertion reads Debug rather than Display. A failure that
    // were swallowed would show up as an Ok here instead, which `expect_err`
    // already catches.
    let msg = format!("{err:?}");
    assert!(
        msg.contains("tca: forced failure") || msg.contains("SqlState"),
        "the failure must surface as the real postgres error, not be swallowed:          got {msg:?}"
    );

    // THE PAIRING IS THE CLAIM. Both counts are zero together. If either one
    // were non-zero on its own, the two halves had decoupled.
    assert!(
        status == "open" && !has_closed_at && events == 0,
        "the row and the event must move together: either both changed or neither did"
    );
}

#[tokio::test]
#[ignore = "needs a scratch PostgreSQL with migrations applied"]
async fn a_close_that_succeeds_leaves_both_halves_present() {
    // The control. Without it, "neither exists" could be satisfied by a
    // close_topic that never writes anything — which would pass the test above
    // for entirely the wrong reason.
    let Some(pool) = scratch_pool().await else {
        return;
    };
    reset(&pool, PROJECT_OK).await;
    let db = SeatloomDb::new(pool.clone());
    let topic_id = format!("tca-ok-{}", std::process::id());

    db.open_topic(&topic_id, PROJECT_OK, "control", None, "seat:nimbus-001")
        .await
        .expect("open");

    let row = db
        .close_topic(&topic_id, "seat:nimbus-001")
        .await
        .expect("close");

    let (status, has_closed_at) = topic_state(&pool, &topic_id).await;
    let events = closed_events(&pool, PROJECT_OK).await;

    assert_eq!(row.status, "closed", "the returned row says closed");
    assert_eq!(status, "closed", "the stored row says closed");
    assert!(has_closed_at, "closed_at is written");
    assert_eq!(events, 1, "exactly one TopicClosed event");
    assert_eq!(row.id, topic_id, "the returned row is the one we closed");

    // And a second close is refused, naming the status.
    let again = db.close_topic(&topic_id, "seat:nimbus-001").await;
    let err = again.expect_err("a second close must be refused");
    let msg = err.to_string();
    assert!(
        msg.contains("closed") || msg.contains("not open") || msg.contains("NotOpen"),
        "the refusal must name the status that blocked it: got {msg:?}"
    );
    assert_eq!(
        closed_events(&pool, PROJECT_OK).await,
        1,
        "the refused second close must not append a second event"
    );
}
