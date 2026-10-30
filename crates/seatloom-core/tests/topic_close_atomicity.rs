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
// An ASYNC mutex, not std::sync::Mutex: the guard is held across the awaits in
// each test (pool queries, the close under test), and clippy is right that a
// blocking guard across an await can deadlock the runtime. The first version
// used std::sync::Mutex and clippy refused it — a refusal that was correct.
use tokio::sync::Mutex;

/// The two tmux... er, DB-touching tests must not overlap.
///
/// WHY. The failure trigger is a TABLE-level constraint trigger on `topics` with
/// a single shared name, so two tests creating and dropping it at once collide:
/// Flux reproduced exactly that, with the second test failing on the FIRST test's
/// trigger ("tca: forced failure at commit time"). He hit it once in five runs —
/// a narrow window, but it is a real race, and it surfaces as a phantom failure.
///
/// Fixed two ways, because either alone is not enough: each test now uses its OWN
/// trigger and function names, AND this mutex serialises them so the table is
/// never carrying another test's trigger while a test is mid-transaction.
static DB_LOCK: std::sync::LazyLock<Mutex<()>> = std::sync::LazyLock::new(|| Mutex::new(()));

const PROJECT: &str = "tclose-atomic";

/// Each test owns its own project row. The two run in PARALLEL, and a shared
/// project id makes them race on `reset()` — the second one to run deletes the
/// first one's rows mid-assertion, which shows up as a phantom failure.
const PROJECT_OK: &str = "tclose-atomic-ok";

/// A scratch pool, or a reason there is not one.
///
/// THE DISTINCTION THAT MATTERS. `Ok(None)` means "no DATABASE_URL was given", so
/// the crate's convention of skipping DB-backed tests without a database holds
/// and `cargo test` still works. `Err` means a database WAS named and something
/// is wrong with it — and that is a FAILURE, not a skip.
///
/// WHY THAT CHANGE. The first version returned `None` for every refusal, which
/// made a scratch DB missing 026 look exactly like a passing run: Flux ran it,
/// saw `test result: ok` in 0.04s, and only noticed because that duration is
/// impossible for a test that really does work. A guard that prints "skipping"
/// and returns None shows "I did not run" as "I passed". A test that can skip
/// silently is worse than no test.
async fn scratch_pool() -> Result<Option<Pool>, String> {
    let Ok(url) = std::env::var(DATABASE_URL_ENV) else {
        return Ok(None);
    };
    let dbname = url.rsplit('/').next().unwrap_or("");
    if dbname == "seatloom" || dbname.starts_with("seatloom?") {
        return Err(format!(
            "DATABASE_URL names the live `{dbname}` database — point it at a scratch DB"
        ));
    }
    let pool = create_pool().map_err(|e| format!("cannot connect to {url}: {e}"))?;
    let client = pool.get().await.map_err(|e| format!("pool: {e}"))?;
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
        .map_err(|e| format!("cannot read the event-type vocabulary: {e}"))?;
    if !row.get::<_, bool>(0) {
        // The database was NAMED and it is missing the migration this test is
        // about. That is a misconfiguration, not an absence, so it FAILS.
        return Err(format!(
            "DATABASE_URL names `{dbname}` but 026_topic_close is not applied \
             (no TopicClosed in canonical_events_event_type_vocab). Apply it, or \
             unset DATABASE_URL to skip this suite."
        ));
    }
    Ok(Some(pool))
}

/// Wipe everything this test could touch, so a re-run starts from nothing and
/// counts are absolute rather than "more than before".
async fn reset(pool: &Pool, project: &str) {
    let c = pool.get().await.expect("client");
    // Drop any trigger a PREVIOUS run left behind before doing anything else.
    //
    // Found by hitting it: a mutation that panicked between installing its
    // trigger and tearing it down left `tca_guard_<pid>` on `topics`, and every
    // later run then failed in `open_topic` with "tca: forced failure at commit
    // time" — a failure that has nothing to do with the code under test. A test
    // that cannot recover from its own crashed run is a trap for the next person.
    for leftover in [
        "DROP TRIGGER IF EXISTS tca_guard ON topics",
        "DROP FUNCTION IF EXISTS tca_fail_at_commit()",
    ] {
        let _ = c.execute(leftover, &[]).await;
    }
    let stale: Vec<String> = c
        .query(
            "SELECT tgname FROM pg_trigger \
              WHERE tgrelid = 'topics'::regclass AND NOT tgisinternal \
                AND tgname LIKE 'tca_%'",
            &[],
        )
        .await
        .map(|rows| rows.iter().map(|r| r.get::<_, String>(0)).collect())
        .unwrap_or_default();
    for name in stale {
        let _ = c
            .execute(&format!("DROP TRIGGER IF EXISTS {name} ON topics"), &[])
            .await;
        let fname = name.replace("tca_guard_", "tca_fail_");
        let _ = c
            .execute(&format!("DROP FUNCTION IF EXISTS {fname}()"), &[])
            .await;
    }
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
    let _guard = DB_LOCK.lock().await;
    let pool = match scratch_pool().await {
        Ok(Some(p)) => p,
        Ok(None) => return,
        Err(why) => panic!("{why}"),
    };
    reset(&pool, PROJECT).await;
    let db = SeatloomDb::new(pool.clone());
    let topic_id = format!("tca-{}", std::process::id());
    let tag = std::process::id().to_string();
    let trigger_name = format!("tca_guard_{tag}");
    let function_name = format!("tca_fail_{tag}");

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
        // Per-test names, so a concurrent test cannot drop this one's trigger.
        let fname = &function_name;
        let tname = &trigger_name;
        c.execute(
            &format!(
                "CREATE OR REPLACE FUNCTION {fname}() RETURNS trigger AS $$ \
                 BEGIN \
                   RAISE EXCEPTION 'tca: forced failure at commit time'; \
                 END; $$ LANGUAGE plpgsql"
            ),
            &[],
        )
        .await
        .expect("fn");
        c.execute(
            &format!(
                "CREATE CONSTRAINT TRIGGER {tname} \
                   AFTER INSERT OR UPDATE ON topics \
                   DEFERRABLE INITIALLY DEFERRED \
                   FOR EACH ROW EXECUTE FUNCTION {fname}()"
            ),
            &[],
        )
        .await
        .expect("trigger");

        // THE ASSERTION WHOSE ABSENCE LET A SURVIVING MUTATION THROUGH. The
        // deferred-ness is not a detail of the setup — it is the entire reason
        // the test can observe a partial state at all. Change
        // `DEFERRABLE INITIALLY DEFERRED` to `INITIALLY IMMEDIATE` and the
        // UPDATE itself is refused before anything partial exists, so both
        // tests still pass, for a weaker reason, and the property the test
        // claims to prove is no longer being tested. I wrote a comment
        // explaining why it must be DEFERRED; a comment is not an assertion,
        // and Aegis measured the mutation green after trusting my comment.
        //
        // Read back from pg_trigger rather than assumed, so this fails if
        // anyone relaxes the trigger.
        let defer = c
            .query_one(
                "SELECT tgdeferrable, tginitdeferred FROM pg_trigger \
                  WHERE tgname = $1 AND tgrelid = 'topics'::regclass",
                &[&tname],
            )
            .await
            .expect("trigger row");
        assert!(
            defer.get::<_, bool>(0) && defer.get::<_, bool>(1),
            "the failure trigger must be DEFERRABLE and INITIALLY DEFERRED — \
             otherwise the UPDATE is refused on its own and this test proves \
             nothing about the UPDATE/event pairing"
        );
    }

    let result = db.close_topic(&topic_id, "seat:nimbus-001").await;

    // Tear the trigger down FIRST, before any assert: if an assert fires with
    // the trigger still installed, every later topic write on this scratch DB
    // fails and the next run cannot set itself up.
    {
        let c = pool.get().await.expect("client");
        let _ = c
            .execute(
                &format!("DROP TRIGGER IF EXISTS {trigger_name} ON topics"),
                &[],
            )
            .await;
        let _ = c
            .execute(&format!("DROP FUNCTION IF EXISTS {function_name}"), &[])
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
    let _guard = DB_LOCK.lock().await;
    let pool = match scratch_pool().await {
        Ok(Some(p)) => p,
        Ok(None) => return,
        Err(why) => panic!("{why}"),
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

#[tokio::test]
#[ignore = "needs a scratch PostgreSQL with migrations applied"]
async fn a_failure_in_the_second_half_alone_leaves_neither_behind() {
    // THE OTHER SHAPE, which the first test cannot produce.
    //
    // The first test fails the transaction AT COMMIT, after both statements have
    // run. That is one way a partial state could arise, but not the one the
    // atomicity claim is most often read as: "the UPDATE lands, then the event
    // append fails". A commit-time trigger on `topics` cannot produce it — the
    // trigger is deferred, so it also blocks any UPDATE on its own connection,
    // which is why moving the UPDATE outside the transaction (a mutation that
    // should be catastrophic) passes the first test. Recorded rather than hidden.
    //
    // So: an IMMEDIATE failure on the event side. A CHECK on `canonical_events`
    // that rejects the insert outright makes the UPDATE succeed inside the
    // transaction and the append fail immediately after it — the exact shape.
    let _guard = DB_LOCK.lock().await;
    let pool = match scratch_pool().await {
        Ok(Some(p)) => p,
        Ok(None) => return,
        Err(why) => panic!("{why}"),
    };
    reset(&pool, PROJECT).await;
    let db = SeatloomDb::new(pool.clone());
    let topic_id = format!("tca-half-{}", std::process::id());
    let tag = std::process::id().to_string();

    db.open_topic(
        &topic_id,
        PROJECT,
        "second-half failure",
        None,
        "seat:nimbus-001",
    )
    .await
    .expect("open");

    {
        let c = pool.get().await.expect("client");
        // NOT deferred: it fires on the INSERT itself, so the UPDATE has already
        // run by the time the append is refused.
        c.execute(
            &format!(
                "CREATE OR REPLACE FUNCTION tca_reject_event_{tag}() RETURNS trigger AS $$ \
                 BEGIN \
                   RAISE EXCEPTION 'tca: event append refused'; \
                 END; $$ LANGUAGE plpgsql"
            ),
            &[],
        )
        .await
        .expect("fn");
        c.execute(
            &format!(
                "CREATE CONSTRAINT TRIGGER tca_event_guard_{tag} \
                   AFTER INSERT ON canonical_events \
                   NOT DEFERRABLE \
                   FOR EACH ROW EXECUTE FUNCTION tca_reject_event_{tag}()"
            ),
            &[],
        )
        .await
        .expect("trigger");
    }

    let result = db.close_topic(&topic_id, "seat:nimbus-001").await;

    {
        let c = pool.get().await.expect("client");
        let _ = c
            .execute(
                &format!("DROP TRIGGER IF EXISTS tca_event_guard_{tag} ON canonical_events"),
                &[],
            )
            .await;
        let _ = c
            .execute(
                &format!("DROP FUNCTION IF EXISTS tca_reject_event_{tag}()"),
                &[],
            )
            .await;
    }

    result.expect_err("the close must fail — the event append is refused");

    let (status, has_closed_at) = topic_state(&pool, &topic_id).await;
    let events = closed_events(&pool, PROJECT).await;

    // THE POINT. The UPDATE ran and the append was refused. If the two were not
    // in one transaction, the topic would now be closed with nothing in the
    // ledger saying so — the exact state the doc claim rules out.
    assert_eq!(
        status, "open",
        "the UPDATE must be rolled back when the event append fails — the row \
         says {status:?}, so the two halves had decoupled"
    );
    assert!(
        !has_closed_at,
        "closed_at must not survive a close whose event append was refused"
    );
    assert_eq!(events, 0, "no event was written");
}
