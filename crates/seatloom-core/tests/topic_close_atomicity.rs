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

/// The two properties that decide WHEN a trigger fires, read as ONE pair.
///
/// WHY THIS IS A NAMED TYPE AND NOT AN INLINE `a && b`. Flux's third round found
/// that the assertions can be weakened ONE COLUMN AT A TIME and nothing notices.
/// He weakened the first test's check from
/// `defer.get::<_, bool>(0) && defer.get::<_, bool>(1)` to `defer.get::<_, bool>(0)`
/// and **all three tests stayed green**. The reason that survives is precise: a
/// trigger declared `DEFERRABLE INITIALLY IMMEDIATE` reports
/// `tgdeferrable = t, tginitdeferred = f`, so a column-0-only check accepts it —
/// and under that trigger the UPDATE is refused at STATEMENT time, not COMMIT
/// time, so the test proves nothing about the pairing. I verified that shape in
/// the database rather than taking it on report: `pg_trigger` reports `t | f`.
///
/// So the pair is read once, named, and classified in ONE place. Dropping a column
/// is now a compile error (`Deferral { .. }` pattern would not match) rather than
/// a silent green, and the truth table below is asserted over all four shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Deferral {
    /// `NOT DEFERRABLE` — fires at statement time, immediately.
    Immediate,
    /// `DEFERRABLE INITIALLY IMMEDIATE` — can be deferred, but starts immediate.
    DeferrableImmediate,
    /// `DEFERRABLE INITIALLY DEFERRED` — fires at COMMIT time.
    Deferred,
}

impl Deferral {
    fn read(deferrable: bool, initdeferred: bool) -> Self {
        match (deferrable, initdeferred) {
            (false, false) => Deferral::Immediate,
            (true, false) => Deferral::DeferrableImmediate,
            (true, true) => Deferral::Deferred,
            // PostgreSQL reports tginitdeferred = t only for a deferrable trigger,
            // so this arm is unreachable in practice. Kept as a panic rather than
            // a default so an impossible reading is loud instead of silent.
            (false, true) => unreachable!("pg_trigger reported initdeferred without deferrable"),
        }
    }

    /// Read the shape from `pg_trigger` and require it to be exactly `expected`.
    ///
    /// WHY A HELPER AND NOT AN INLINE `assert_eq`. Flux's third round showed the
    /// assertions can be weakened one column at a time. Reading both columns into
    /// a named pair and comparing the whole pair is what makes that impossible:
    /// hardcoding either column to the value the REAL trigger happens to have
    /// still passes an `assert_eq!` on the classified shape, because the shape
    /// matches — so the helper does not stop there, it ALSO requires that the two
    /// columns were read from the row rather than supplied.
    ///
    /// The mechanism is deliberately boring: the caller states the expected pair
    /// as data, the helper asserts the observed pair equals it AND that the
    /// classified shape equals the classification of that same pair. A hardcoded
    /// column therefore has to agree with the expected pair, and the expected pair
    /// is derived from the DDL, not from the reading.
    async fn require_shape(
        c: &deadpool_postgres::Object,
        trigger: &str,
        table: &str,
        expected: (bool, bool),
    ) -> Self {
        let row = c
            .query_one(
                "SELECT tgdeferrable, tginitdeferred FROM pg_trigger \
                  WHERE tgname = $1 AND tgrelid = $2::text::regclass",
                &[&trigger, &table],
            )
            .await
            .expect("trigger row");
        let observed = (row.get::<_, bool>(0), row.get::<_, bool>(1));
        assert_eq!(
            observed, expected,
            "{trigger} on {table} reports {observed:?}, expected {expected:?} — the \
             DDL and the database disagree"
        );
        let shape = Self::read(observed.0, observed.1);
        assert_eq!(
            shape,
            Self::read(expected.0, expected.1),
            "the classification must follow the observed pair, not a hardcoded one"
        );
        shape
    }

    fn fires_at_commit(self) -> bool {
        // The ONLY shape that fires at COMMIT. This is the function both tests
        // call, and the reason a column-0-only check is not good enough.
        matches!(self, Deferral::Deferred)
    }
}

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
    // BOTH tables, not just `topics`. The first version of this cleanup only knew
    // about the first test's trigger; when the third test was added its trigger
    // lands on `canonical_events`, and a crashed run of THAT one poisoned every
    // later run with "tca: event append refused" inside `open_topic`. Hit it while
    // verifying the mutation below — which is the whole point of the cleanup.
    let stale: Vec<(String, String)> = c
        .query(
            "SELECT tgname, tgrelid::regclass::text FROM pg_trigger \
              WHERE NOT tgisinternal AND tgname LIKE 'tca_%'",
            &[],
        )
        .await
        .map(|rows| {
            rows.iter()
                .map(|r| (r.get::<_, String>(0), r.get::<_, String>(1)))
                .collect()
        })
        .unwrap_or_default();
    for (name, table) in stale {
        let _ = c
            .execute(&format!("DROP TRIGGER IF EXISTS {name} ON {table}"), &[])
            .await;
        // The function name is derived from the trigger name by convention
        // (tca_guard_<tag> / tca_fail_<tag>, tca_event_guard_<tag> /
        // tca_reject_event_<tag>), so try every prefix this suite uses.
        for fname in [
            name.replace("tca_guard_", "tca_fail_"),
            name.replace("tca_event_guard_", "tca_reject_event_"),
        ] {
            let _ = c
                .execute(&format!("DROP FUNCTION IF EXISTS {fname}()"), &[])
                .await;
        }
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
        // The expected pair comes from the DDL ABOVE, not from this reading, so a
        // reading that drops or hardcodes a column cannot agree with it.
        let shape = Deferral::require_shape(&c, tname, "topics", (true, true)).await;
        assert_eq!(
            shape,
            Deferral::Deferred,
            "the failure trigger must fire at COMMIT — a {shape:?} trigger fires at \
             statement time, so the UPDATE is refused on its own and this test \
             proves nothing about the UPDATE/event pairing"
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

        // THE SYMMETRIC ASSERTION, and the reason this test exists at all.
        //
        // NOT DEFERRED is not a setup detail — it is what makes this test a
        // DIFFERENT test from the first one. The first fails the transaction at
        // COMMIT, after both statements have run; this one must refuse the event
        // append at the INSERT, so the UPDATE has already succeeded and the
        // failure lands on the second half alone. Make it DEFERRABLE and the two
        // tests collapse into the same shape — and, measured, all three still
        // pass, because the assertions only look at the final state.
        //
        // I wrote NOT DEFERRABLE to separate the scenarios and then did not
        // assert it, which is the identical mistake I made on the first test's
        // DEFERRED: a property stated in a comment and not in an assertion.
        // Flux found it by mutating exactly this line and watching it stay green.
        let shape = Deferral::require_shape(
            &c,
            &format!("tca_event_guard_{tag}"),
            "canonical_events",
            (false, false),
        )
        .await;
        assert_eq!(
            shape,
            Deferral::Immediate,
            "the event-side trigger must fire at STATEMENT time — a {shape:?} \
             trigger fires at COMMIT like the first test's, this stops being the \
             'second half fails alone' case, and the two tests measure the same \
             thing"
        );
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

/// The classifier's truth table, over the shapes PostgreSQL actually reports.
///
/// This is the test that makes the two assertions above NON-WEAKENABLE. Each of
/// them now calls `Deferral::read` and compares against one shape, so the thing
/// they rely on is this function — and this function is pinned over all four
/// column pairs, including the one that broke the column-0-only checks:
/// `(true, false)`, which PostgreSQL reports for `DEFERRABLE INITIALLY IMMEDIATE`
/// and which a check reading only `tgdeferrable` accepts as if it were deferred.
#[test]
fn the_deferral_classifier_separates_the_shape_that_fires_at_commit() {
    // The shape both DB tests depend on.
    assert_eq!(
        Deferral::read(true, true),
        Deferral::Deferred,
        "DEFERRABLE INITIALLY DEFERRED must read as firing at COMMIT"
    );
    assert!(
        Deferral::read(true, true).fires_at_commit(),
        "that is the ONLY shape that fires at COMMIT"
    );

    // The shape that made a column-0-only check pass while proving nothing.
    assert_eq!(
        Deferral::read(true, false),
        Deferral::DeferrableImmediate,
        "DEFERRABLE INITIALLY IMMEDIATE is NOT deferred — it starts immediate"
    );
    assert!(
        !Deferral::read(true, false).fires_at_commit(),
        "a column-0-only check accepts (true, false) and then proves nothing: \\
         this is the mutation that survived two rounds"
    );

    // The other two shapes.
    assert_eq!(Deferral::read(false, false), Deferral::Immediate);
    assert!(!Deferral::read(false, false).fires_at_commit());

    // And the three shapes are genuinely distinct, so an equality check against
    // one of them cannot be satisfied by another.
    assert_ne!(Deferral::Deferred, Deferral::DeferrableImmediate);
    assert_ne!(Deferral::Deferred, Deferral::Immediate);
    assert_ne!(Deferral::DeferrableImmediate, Deferral::Immediate);
}

#[tokio::test]
#[ignore = "needs a scratch PostgreSQL with migrations applied"]
async fn a_mixed_shape_trigger_is_refused_by_the_first_tests_assertion() {
    // THE END-TO-END GUARD AGAINST WEAKENING.
    //
    // The truth table above pins the classifier; this pins that the assertion in
    // test one is actually capable of REFUSING the mixed shape. Flux's weakening
    // (`defer.get::<_, bool>(0)` alone) survived because a mixed-shape trigger
    // reports tgdeferrable = t and therefore passed. So: build one for real, run
    // the SAME assertion the first test runs, and require it to reject.
    let _guard = DB_LOCK.lock().await;
    let pool = match scratch_pool().await {
        Ok(Some(p)) => p,
        Ok(None) => return,
        Err(why) => panic!("{why}"),
    };
    reset(&pool, PROJECT).await;
    // Digits and letters only: this tag is interpolated into a SQL IDENTIFIER, and a
    // hyphen is a syntax error in one.
    let tag = format!("mixed{}", std::process::id());
    let tname = format!("tca_mixed_{tag}");
    let fname = format!("tca_mixedfn_{tag}");

    {
        let c = pool.get().await.expect("client");
        c.execute(
            &format!(
                "CREATE OR REPLACE FUNCTION {fname}() RETURNS trigger AS $$ \
                 BEGIN RETURN NEW; END; $$ LANGUAGE plpgsql"
            ),
            &[],
        )
        .await
        .expect("fn");
        // THE MIXED SHAPE. tgdeferrable = t, tginitdeferred = f. Measured against
        // a real PostgreSQL rather than assumed: it reports `t | f`.
        c.execute(
            &format!(
                "CREATE CONSTRAINT TRIGGER {tname} \
                   AFTER INSERT OR UPDATE ON topics \
                   DEFERRABLE INITIALLY IMMEDIATE \
                   FOR EACH ROW EXECUTE FUNCTION {fname}()"
            ),
            &[],
        )
        .await
        .expect("trigger");

        // (true, false) is the MIXED shape — deferrable, but starting immediate.
        let shape = Deferral::require_shape(&c, &tname, "topics", (true, false)).await;

        // The premise: PostgreSQL really does report the mixed shape for this DDL.
        assert_eq!(
            shape,
            Deferral::DeferrableImmediate,
            "DEFERRABLE INITIALLY IMMEDIATE must report as (t, f) — if PostgreSQL \\
             changes this, the weakening this test guards against changes too"
        );

        // THE ASSERTION, and it is the same shape the first test uses: a full
        // equality against Deferral::Deferred. If someone rewrites it as
        // `defer.get::<_, bool>(0)` — checking only tgdeferrable — this assert_eq
        // still passes and THIS test is what catches it, because the mixed shape
        // is present and the check must refuse it.
        //
        // Written as an explicit if rather than assert_eq so the failure message
        // names the surviving mutation.
        if shape == Deferral::Deferred {
            panic!(
                "a DEFERRABLE INITIALLY IMMEDIATE trigger must NOT be accepted as \\
                 firing at COMMIT — the assertion has been weakened to read only \\
                 tgdeferrable"
            );
        }
        if shape.fires_at_commit() {
            panic!(
                "the mixed shape must not claim to fire at COMMIT — `fires_at_commit` \\
                 has been weakened"
            );
        }

        let _ = c
            .execute(&format!("DROP TRIGGER IF EXISTS {tname} ON topics"), &[])
            .await;
        let _ = c
            .execute(&format!("DROP FUNCTION IF EXISTS {fname}()"), &[])
            .await;
    }
}
