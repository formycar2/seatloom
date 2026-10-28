//! Operator-decision (020) integration tests for `SeatloomDb`'s
//! `request_operator_decision` / `resolve_operator_decision`.
//!
//! These require a running PostgreSQL with migrations 001–020 applied, so they
//! are `#[ignore]` — `cargo test -p seatloom-core` passes without a DB. They
//! REFUSE to run against the live `seatloom` database (name guard): point them
//! at a scratch DB, e.g.
//!
//!   DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/scratch_020 \
//!     cargo test -p seatloom-core --test operator_decision_integration -- --include-ignored
//!
//! Each test namespaces its rows under its own project id and cleans up first,
//! so they are independent and re-runnable.
//!
//! WHAT THESE PROVE that the unit tests cannot: the row and the event land in
//! ONE transaction, the pending list is scoped by decider, and a resolved
//! decision is genuinely immutable at the DATABASE level — not merely refused by
//! the Rust path. The last one matters most: a second implementation (a command,
//! a script, a psql session) must hit the same wall.

#[cfg(test)]
mod operator_decision {
    use deadpool_postgres::Pool;
    use seatloom_core::db::connection::{create_pool, DATABASE_URL_ENV};
    use seatloom_core::db::repositories::{DbError, NewOperatorDecision, SeatloomDb};

    /// Connect to a SCRATCH database, or return `None` to skip. Refuses the live
    /// `seatloom` database outright — these tests write rows and must never
    /// touch live.
    async fn scratch_pool() -> Option<Pool> {
        let url = std::env::var(DATABASE_URL_ENV).ok()?;
        let dbname = url.rsplit('/').next().unwrap_or("");
        if dbname == "seatloom" || dbname.starts_with("seatloom?") {
            eprintln!(
                "REFUSING live `seatloom` DB for 020 integration tests — set DATABASE_URL to a scratch DB"
            );
            return None;
        }
        let pool = create_pool().ok()?;
        let client = pool.get().await.ok()?;
        // Confirm 020 is applied; otherwise skip rather than fail spuriously.
        client
            .query_one(
                "SELECT to_regclass('public.operator_decisions') IS NOT NULL",
                &[],
            )
            .await
            .ok()
            .filter(|r| r.get::<_, bool>(0))?;
        Some(pool)
    }

    /// Fresh fixture: the project, and no leftover decisions under it.
    async fn seed(pool: &Pool, project: &str) {
        let c = pool.get().await.expect("client");
        c.execute(
            "DELETE FROM operator_decisions WHERE project_id = $1",
            &[&project],
        )
        .await
        .expect("clean decisions");
        c.execute(
            "DELETE FROM event_object_refs WHERE event_id IN \
             (SELECT id FROM canonical_events WHERE project_id = $1)",
            &[&project],
        )
        .await
        .expect("clean refs");
        c.execute(
            "DELETE FROM canonical_events WHERE project_id = $1",
            &[&project],
        )
        .await
        .expect("clean events");
        c.execute(
            "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
             VALUES ($1,$1,NOW(),1024,1024) ON CONFLICT (id) DO NOTHING",
            &[&project],
        )
        .await
        .expect("project");
    }

    /// How many events of one type a project carries — the proof that a write
    /// path appended exactly one, not zero and not two.
    async fn event_count(pool: &Pool, project: &str, event_type: &str) -> i64 {
        let c = pool.get().await.expect("client");
        c.query_one(
            "SELECT count(*) FROM canonical_events WHERE project_id = $1 AND event_type = $2",
            &[&project, &event_type],
        )
        .await
        .expect("count")
        .get::<_, i64>(0)
    }

    #[tokio::test]
    async fn request_lands_row_and_exactly_one_event() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let project = "scratch-020-request";
        seed(&pool, project).await;

        let row = db
            .request_operator_decision(NewOperatorDecision {
                decision_id: "dec-scratch-request",
                project_id: project,
                target: "删不删那个文件",
                choices: &["删".to_string(), "留".to_string()],
                decider: "human:supervisor",
                consequence: "删了主线就干净了",
                evidence: &["disp-DFeH6Ut".to_string()],
                created_by: "seat:seat-aegis-001",
            })
            .await
            .expect("a well-formed obligation must land");

        assert_eq!(row.status, "pending");
        assert_eq!(row.choices, vec!["删", "留"]);
        assert_eq!(row.decider, "human:supervisor");
        assert!(
            row.chosen.is_none(),
            "a pending obligation carries no ruling"
        );

        // The event half of the same transaction: exactly one, with the
        // decision as its subject.
        assert_eq!(
            event_count(&pool, project, "OperatorDecisionRequested").await,
            1,
            "one request must append exactly one event"
        );
        let subject = pool
            .get()
            .await
            .expect("client")
            .query_one(
                "SELECT count(*) FROM event_object_refs \
                  WHERE event_id IN (SELECT id FROM canonical_events \
                                      WHERE project_id = $1 AND event_type = 'OperatorDecisionRequested') \
                    AND ref_type = 'decision' AND ref_id = $2 AND role = 'subject'",
                &[&project, &"dec-scratch-request"],
            )
            .await
            .expect("subject ref")
            .get::<_, i64>(0);
        assert_eq!(
            subject, 1,
            "the event must name the decision as its subject"
        );
    }

    #[tokio::test]
    async fn resolve_lands_ruling_and_exactly_one_event() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let project = "scratch-020-resolve";
        seed(&pool, project).await;

        db.request_operator_decision(NewOperatorDecision {
            decision_id: "dec-scratch-resolve",
            project_id: project,
            target: "删不删那个文件",
            choices: &["删".to_string(), "留".to_string()],
            decider: "human:supervisor",
            consequence: "删了主线就干净了",
            evidence: &[],
            created_by: "seat:seat-aegis-001",
        })
        .await
        .expect("request");

        let resolved = db
            .resolve_operator_decision("dec-scratch-resolve", "删", "human:supervisor")
            .await
            .expect("a ruling by the decider must land");

        assert_eq!(resolved.status, "resolved");
        assert_eq!(resolved.chosen.as_deref(), Some("删"));
        assert_eq!(resolved.resolved_by.as_deref(), Some("human:supervisor"));
        assert!(
            resolved.resolved_at.is_some(),
            "the ruling time is recorded"
        );
        assert_eq!(
            event_count(&pool, project, "OperatorDecisionResolved").await,
            1,
            "one ruling must append exactly one event"
        );
    }

    /// THE IMMUTABILITY PROOF. The Rust path refuses a second ruling; this test
    /// goes AROUND it with raw SQL and proves the database refuses too. A second
    /// implementation (a command, a script, a psql session) must hit the same
    /// wall, or "immutable" is only true of one code path.
    #[tokio::test]
    async fn a_resolved_decision_is_immutable_at_the_database() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let project = "scratch-020-immutable";
        seed(&pool, project).await;

        db.request_operator_decision(NewOperatorDecision {
            decision_id: "dec-scratch-immutable",
            project_id: project,
            target: "删不删那个文件",
            choices: &["删".to_string(), "留".to_string()],
            decider: "human:supervisor",
            consequence: "删了主线就干净了",
            evidence: &[],
            created_by: "seat:seat-aegis-001",
        })
        .await
        .expect("request");
        db.resolve_operator_decision("dec-scratch-immutable", "删", "human:supervisor")
            .await
            .expect("first ruling");

        // Raw SQL, bypassing the Rust path entirely: clear the ruling while the
        // row stays 'resolved'. The pairing CHECK must refuse.
        let c = pool.get().await.expect("client");
        let err = c
            .execute(
                "UPDATE operator_decisions SET chosen = NULL, resolved_by = NULL, \
                   resolved_at = NULL WHERE id = $1",
                &[&"dec-scratch-immutable"],
            )
            .await
            .expect_err("a resolved row must not be able to lose its ruling");
        // `tokio_postgres::Error`'s Display for a server error is the literal
        // "db error" — the constraint name is in the `DbError` it wraps, which
        // `as_db_error()` exposes. Asserting on the Display string would have
        // passed against "db error" and proven nothing, which is the shape of a
        // check that cannot fail.
        let pg = err
            .as_db_error()
            .expect("a CHECK violation must carry a db error");
        assert!(
            pg.constraint()
                .is_some_and(|c| c == "operator_decisions_resolved_pairing"),
            "the refusal must name the rule: {pg}"
        );

        // And the reverse direction: a pending row may not carry a ruling.
        // Re-open it with a ruling still attached.
        let err = c
            .execute(
                "UPDATE operator_decisions SET status = 'pending' WHERE id = $1",
                &[&"dec-scratch-immutable"],
            )
            .await
            .expect_err("re-opening must not leave a ruling on a pending row");
        let pg = err
            .as_db_error()
            .expect("a CHECK violation must carry a db error");
        assert!(
            pg.constraint()
                .is_some_and(|c| c == "operator_decisions_resolved_pairing"),
            "the pairing rule holds in both directions: {pg}"
        );

        // The recorded ruling is untouched by both refusals.
        let after = db
            .get_operator_decision("dec-scratch-immutable")
            .await
            .expect("read")
            .expect("row");
        assert_eq!(after.status, "resolved");
        assert_eq!(after.chosen.as_deref(), Some("删"));
    }

    /// The decider scoping the packet demands: a row Aegis created for ITSELF
    /// must not appear in the operator's pending count. If it did, the count
    /// would be wrong — which is the defect this whole object exists to fix.
    #[tokio::test]
    async fn the_pending_list_is_scoped_by_decider() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let project = "scratch-020-decider";
        seed(&pool, project).await;

        db.request_operator_decision(NewOperatorDecision {
            decision_id: "dec-scratch-for-operator",
            project_id: project,
            target: "要不要给防护加豁免",
            choices: &["加".to_string(), "不加".to_string()],
            decider: "human:supervisor",
            consequence: "加了才能合掉那个死代码删除",
            evidence: &[],
            created_by: "seat:seat-aegis-001",
        })
        .await
        .expect("operator obligation");

        db.request_operator_decision(NewOperatorDecision {
            decision_id: "dec-scratch-for-aegis",
            project_id: project,
            target: "活派给谁",
            choices: &["nimbus".to_string(), "lyra".to_string()],
            decider: "seat:seat-aegis-001",
            consequence: "决定了就发派单",
            evidence: &[],
            created_by: "seat:seat-aegis-001",
        })
        .await
        .expect("aegis's own obligation");

        let for_operator = db
            .list_pending_operator_decisions(project, Some("human:supervisor"), 100)
            .await
            .expect("list");
        let for_aegis = db
            .list_pending_operator_decisions(project, Some("seat:seat-aegis-001"), 100)
            .await
            .expect("list");
        let all = db
            .list_pending_operator_decisions(project, None, 100)
            .await
            .expect("list");

        assert_eq!(
            for_operator.len(),
            1,
            "the operator sees only his own obligation"
        );
        assert_eq!(for_operator[0].id, "dec-scratch-for-operator");
        assert_eq!(for_aegis.len(), 1, "Aegis sees only its own");
        assert_eq!(for_aegis[0].id, "dec-scratch-for-aegis");
        assert_eq!(all.len(), 2, "the unscoped list sees both");

        // A resolved row leaves the pending list but stays readable.
        db.resolve_operator_decision("dec-scratch-for-operator", "加", "human:supervisor")
            .await
            .expect("rule");
        let after = db
            .list_pending_operator_decisions(project, Some("human:supervisor"), 100)
            .await
            .expect("list");
        assert!(
            after.is_empty(),
            "a resolved obligation is no longer pending"
        );
        let resolved = db
            .list_resolved_operator_decisions(project, Some("human:supervisor"), 100)
            .await
            .expect("audit list");
        assert_eq!(resolved.len(), 1, "and it is still on the audit side");
    }

    /// `chosen` must be one of the row's OWN choices, and the refusal must name
    /// the legal options. A ruling outside the offered set is not a ruling — it
    /// is a new question wearing a ruling's clothes.
    #[tokio::test]
    async fn a_ruling_outside_the_offered_choices_is_refused_by_name() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let project = "scratch-020-choice";
        seed(&pool, project).await;

        db.request_operator_decision(NewOperatorDecision {
            decision_id: "dec-scratch-choice",
            project_id: project,
            target: "删不删那个文件",
            choices: &["删".to_string(), "留".to_string()],
            decider: "human:supervisor",
            consequence: "删了主线就干净了",
            evidence: &[],
            created_by: "seat:seat-aegis-001",
        })
        .await
        .expect("request");

        let err = db
            .resolve_operator_decision("dec-scratch-choice", "改成大写", "human:supervisor")
            .await
            .expect_err("a ruling outside the offered choices must be refused");
        match err {
            DbError::OperatorDecisionChoiceNotOffered {
                id,
                chosen,
                choices,
            } => {
                assert_eq!(id, "dec-scratch-choice");
                assert_eq!(chosen, "改成大写");
                assert_eq!(choices, vec!["删", "留"], "the legal options are named");
            }
            other => panic!("expected ChoiceNotOffered, got {other:?}"),
        }
        // Nothing was written: the row is still pending with no ruling.
        let after = db
            .get_operator_decision("dec-scratch-choice")
            .await
            .expect("read")
            .expect("row");
        assert_eq!(after.status, "pending");
        assert!(after.chosen.is_none());
        assert_eq!(
            event_count(&pool, project, "OperatorDecisionResolved").await,
            0,
            "a refused ruling appends no event"
        );
    }

    /// A second ruling on a resolved obligation is refused, and the refusal is
    /// the DATABASE's, not only the Rust path's — the row is loaded FOR UPDATE
    /// and its status re-checked inside the transaction.
    #[tokio::test]
    async fn a_second_ruling_is_refused() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let project = "scratch-020-second";
        seed(&pool, project).await;

        db.request_operator_decision(NewOperatorDecision {
            decision_id: "dec-scratch-second",
            project_id: project,
            target: "删不删那个文件",
            choices: &["删".to_string(), "留".to_string()],
            decider: "human:supervisor",
            consequence: "删了主线就干净了",
            evidence: &[],
            created_by: "seat:seat-aegis-001",
        })
        .await
        .expect("request");
        db.resolve_operator_decision("dec-scratch-second", "删", "human:supervisor")
            .await
            .expect("first ruling");

        let err = db
            .resolve_operator_decision("dec-scratch-second", "留", "human:supervisor")
            .await
            .expect_err("a second ruling must be refused");
        assert!(
            matches!(err, DbError::OperatorDecisionAlreadyResolved { .. }),
            "expected AlreadyResolved, got {err:?}"
        );
        assert_eq!(
            event_count(&pool, project, "OperatorDecisionResolved").await,
            1,
            "still exactly one ruling event"
        );
        let after = db
            .get_operator_decision("dec-scratch-second")
            .await
            .expect("read")
            .expect("row");
        assert_eq!(
            after.chosen.as_deref(),
            Some("删"),
            "the first ruling stands, unrewritten"
        );
    }
}
