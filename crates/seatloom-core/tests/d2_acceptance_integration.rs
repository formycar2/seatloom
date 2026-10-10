//! D2 acceptance-gate integration tests (schema 014) for `SeatloomDb`'s
//! `decide_acceptance` and the acceptance read model.
//!
//! These require a running PostgreSQL with migrations 001–014 applied, so they
//! are `#[ignore]` — `cargo test -p seatloom-core` passes without a DB. They
//! REFUSE to run against the live `seatloom` database (name guard): point them
//! at a scratch DB, e.g.
//!
//!   DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_d2_nimbus \
//!     cargo test -p seatloom-core --test d2_acceptance_integration -- --include-ignored
//!
//! Each test namespaces its rows under its own project id and cleans up first,
//! so they are independent and re-runnable.

#[cfg(test)]
mod d2 {
    use deadpool_postgres::Pool;
    use seatloom_core::db::connection::{create_pool, DATABASE_URL_ENV};
    use seatloom_core::db::repositories::{DbError, SeatloomDb};

    /// Connect to a SCRATCH database, or return `None` to skip. Refuses the live
    /// `seatloom` database outright — these tests write dispatches/acceptances
    /// and must never touch live.
    async fn scratch_pool() -> Option<Pool> {
        let url = std::env::var(DATABASE_URL_ENV).ok()?;
        let dbname = url.rsplit('/').next().unwrap_or("");
        if dbname == "seatloom" || dbname.starts_with("seatloom?") {
            eprintln!("REFUSING live `seatloom` DB for D2 integration tests — set DATABASE_URL to a scratch DB");
            return None;
        }
        let pool = create_pool().ok()?;
        let client = pool.get().await.ok()?;
        // Confirm 014 is applied; otherwise skip rather than fail spuriously.
        client
            .query_one("SELECT to_regclass('public.acceptances') IS NOT NULL", &[])
            .await
            .ok()
            .filter(|r| r.get::<_, bool>(0))?;
        Some(pool)
    }

    /// Fresh fixture for one test's project: clean prior rows, ensure the
    /// project + a done dispatch + a Flux verdict event + a pending acceptance.
    async fn seed(pool: &Pool, project: &str, dispatch: &str, acceptance: &str, verdict: &str) {
        let c = pool.get().await.expect("client");
        // Clean (child rows first).
        for sql in [
            format!("DELETE FROM acceptances WHERE project_id = '{project}'"),
            format!("DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id = '{project}')"),
            format!("DELETE FROM dispatches WHERE project_id = '{project}'"),
            format!("DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id = '{project}')"),
            format!("DELETE FROM canonical_events WHERE project_id = '{project}'"),
        ] {
            c.execute(&sql, &[]).await.expect("clean");
        }
        c.execute(
            "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
             VALUES ($1,$1,NOW(),1024,1024) ON CONFLICT (id) DO NOTHING",
            &[&project],
        ).await.expect("project");
        c.execute(
            "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at,completed_at) \
             VALUES ($1,'seat-nimbus-001','human:supervisor',$2,'finished task','{\"packet\":\"p/x.md\"}','done',5,NOW(),NOW())",
            &[&dispatch, &project],
        ).await.expect("dispatch");
        c.execute(
            "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
             VALUES ($1,'ReviewVerdictIssued',NOW(),'seat:seat-flux-001','{\"verdict\":\"pass\"}',$2,1)",
            &[&verdict, &project],
        ).await.expect("verdict");
        c.execute(
            "INSERT INTO acceptances (id,dispatch_id,project_id,branch,base,how_to_verify,flux_verdict_ref,status) \
             VALUES ($1,$2,$3,'packet/nimbus/x','track','do A see B',$4,'pending')",
            &[&acceptance, &dispatch, &project, &verdict],
        ).await.expect("acceptance");
    }

    async fn event_types(pool: &Pool, acceptance: &str) -> Vec<String> {
        let c = pool.get().await.expect("client");
        c.query(
            "SELECT e.event_type FROM canonical_events e \
               JOIN event_object_refs r ON r.event_id = e.id AND r.role='subject' \
              WHERE r.ref_type='acceptance' AND r.ref_id=$1 ORDER BY e.occurred_at",
            &[&acceptance],
        )
        .await
        .expect("events")
        .iter()
        .map(|row| row.get::<_, String>(0))
        .collect()
    }

    // ------------------------------------------------------------------ accept

    #[tokio::test]
    #[ignore = "requires PostgreSQL with migrations 001-014 (scratch DB)"]
    async fn decide_accept_lands_status_commit_decider_and_event() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let (project, dispatch, acc, verdict) = (
            "d2-itest-accept",
            "disp-itest-accept",
            "acc-itest-accept",
            "ev-itest-accept-verdict",
        );
        seed(&pool, project, dispatch, acc, verdict).await;
        let db = SeatloomDb::new(pool.clone());

        let decided = db
            .decide_acceptance(
                acc,
                "accepted",
                None,
                Some("deadbeefsha"),
                None,
                "human:operator",
            )
            .await
            .expect("accept must land");
        assert_eq!(decided.status, "accepted");
        assert_eq!(decided.merge_commit.as_deref(), Some("deadbeefsha"));
        assert_eq!(decided.decided_by.as_deref(), Some("human:operator"));
        assert!(decided.decided_at.is_some(), "decided_at must be set");
        assert_eq!(decided.rework_dispatch_id, None);

        assert_eq!(event_types(&pool, acc).await, vec!["AcceptanceGranted"]);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with migrations 001-014 (scratch DB)"]
    async fn decide_reject_lands_feedback_rework_and_event() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let (project, dispatch, acc, verdict) = (
            "d2-itest-reject",
            "disp-itest-reject",
            "acc-itest-reject",
            "ev-itest-reject-verdict",
        );
        seed(&pool, project, dispatch, acc, verdict).await;
        let db = SeatloomDb::new(pool.clone());

        // A real rework dispatch row the decision links to (lineage).
        let c = pool.get().await.expect("client");
        c.execute(
            "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at) \
             VALUES ('disp-itest-rework','seat-nimbus-001','human:operator',$1,'打回重做:finished task','{\"rework_of\":\"disp-itest-reject\"}','ready',15,NOW())",
            &[&project],
        ).await.expect("rework dispatch");

        let decided = db
            .decide_acceptance(
                acc,
                "rejected",
                Some("fix the edge case"),
                None,
                Some("disp-itest-rework"),
                "human:operator",
            )
            .await
            .expect("reject must land");
        assert_eq!(decided.status, "rejected");
        assert_eq!(decided.feedback.as_deref(), Some("fix the edge case"));
        assert_eq!(
            decided.rework_dispatch_id.as_deref(),
            Some("disp-itest-rework")
        );
        assert_eq!(decided.merge_commit, None);
        assert!(decided.decided_at.is_some());

        assert_eq!(event_types(&pool, acc).await, vec!["AcceptanceRejected"]);
    }

    // ------------------------------------------------------------------ guards

    #[tokio::test]
    #[ignore = "requires PostgreSQL with migrations 001-014 (scratch DB)"]
    async fn terminal_acceptance_is_immutable_second_decide_refused() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let (project, dispatch, acc, verdict) = (
            "d2-itest-terminal",
            "disp-itest-terminal",
            "acc-itest-terminal",
            "ev-itest-terminal-verdict",
        );
        seed(&pool, project, dispatch, acc, verdict).await;
        let db = SeatloomDb::new(pool.clone());

        db.decide_acceptance(acc, "accepted", None, Some("sha1"), None, "human:operator")
            .await
            .expect("first decision lands");
        // A second decision of the now-terminal acceptance is refused; no rewrite.
        let err = db
            .decide_acceptance(
                acc,
                "rejected",
                Some("too late"),
                None,
                None,
                "human:operator",
            )
            .await
            .expect_err("a decided acceptance is terminal");
        assert!(
            matches!(err, DbError::AcceptanceNotPending { .. }),
            "expected AcceptanceNotPending, got {err:?}"
        );
        // Still accepted, still one event.
        let row = db.get_acceptance(acc).await.unwrap().unwrap();
        assert_eq!(row.acceptance.status, "accepted");
        assert_eq!(event_types(&pool, acc).await, vec!["AcceptanceGranted"]);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with migrations 001-014 (scratch DB)"]
    async fn second_pending_acceptance_is_refused_by_the_partial_unique_index() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let (project, dispatch, acc, verdict) = (
            "d2-itest-uniq",
            "disp-itest-uniq",
            "acc-itest-uniq",
            "ev-itest-uniq-verdict",
        );
        seed(&pool, project, dispatch, acc, verdict).await;
        let c = pool.get().await.expect("client");
        let dup = c
            .execute(
                "INSERT INTO acceptances (id,dispatch_id,project_id,branch,how_to_verify,flux_verdict_ref,status) \
                 VALUES ('acc-itest-uniq-2',$1,$2,'b','h',$3,'pending')",
                &[&dispatch, &project, &verdict],
            )
            .await;
        assert!(dup.is_err(), "a second pending acceptance must be rejected");
        // tokio_postgres's top-level Display is "db error"; the constraint name
        // lives in the source chain — walk it.
        let err = dup.unwrap_err();
        let mut chain = err.to_string();
        let mut src = std::error::Error::source(&err);
        while let Some(s) = src {
            chain.push_str(" | ");
            chain.push_str(&s.to_string());
            src = s.source();
        }
        assert!(
            chain.contains("uq_acceptances_one_pending_per_dispatch")
                || chain.contains("duplicate")
                || chain.contains("unique"),
            "must be the partial-unique-index violation: {chain}"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with migrations 001-014 (scratch DB)"]
    async fn flux_verdict_existence_check_reads_events_and_artifacts() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let (project, dispatch, acc, verdict) = (
            "d2-itest-verdict",
            "disp-itest-verdict",
            "acc-itest-verdict",
            "ev-itest-verdict-verdict",
        );
        seed(&pool, project, dispatch, acc, verdict).await;
        let db = SeatloomDb::new(pool.clone());
        assert!(
            db.event_or_artifact_exists(verdict).await.unwrap(),
            "the seeded verdict event must be found"
        );
        assert!(
            !db.event_or_artifact_exists("ev-nope-does-not-exist")
                .await
                .unwrap(),
            "a missing referent reads as absent (未验绿不可合)"
        );
    }

    // ------------------------------------------------------------- read model

    #[tokio::test]
    #[ignore = "requires PostgreSQL with migrations 001-014 (scratch DB)"]
    async fn read_model_joins_dispatch_and_verdict_pending_first() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let (project, dispatch, acc, verdict) = (
            "d2-itest-read",
            "disp-itest-read",
            "acc-itest-read",
            "ev-itest-read-verdict",
        );
        seed(&pool, project, dispatch, acc, verdict).await;
        let db = SeatloomDb::new(pool.clone());
        let cards = db.list_acceptances_for_project(project, 50).await.unwrap();
        let card = cards
            .iter()
            .find(|c| c.acceptance.id == acc)
            .expect("the card");
        assert_eq!(card.acceptance.status, "pending");
        assert_eq!(card.dispatch_title, "finished task");
        assert_eq!(card.dispatch_seat_id, "seat-nimbus-001");
        assert_eq!(card.flux_verdict.as_deref(), Some("pass"));
        assert!(card.flux_verdict_occurred_at.is_some());
    }
}
