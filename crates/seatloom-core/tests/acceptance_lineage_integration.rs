//! Acceptance-lineage integration tests (acceptance-lineage milestone, Lane B)
//! for the pure-read assembler `seatloom_core::db::lineage::get_acceptance_lineage`.
//!
//! These require a running PostgreSQL with migrations 001–014 + 016
//! (acceptance_plan) applied, so they are `#[ignore]` — `cargo test -p
//! seatloom-core` passes without a DB. They REFUSE to run against the live
//! `seatloom` database (name guard): point them at a scratch DB, e.g.
//!
//!   DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_lineage \
//!     cargo test -p seatloom-core --test acceptance_lineage_integration -- --include-ignored
//!
//! The assembler is a READ: these tests seed rows, assemble, and assert — they
//! never write through the assembler.

#[cfg(test)]
mod lineage {
    use deadpool_postgres::Pool;
    use seatloom_core::db::connection::{create_pool, DATABASE_URL_ENV};
    use seatloom_core::db::lineage::{get_acceptance_lineage, LineageError};
    use seatloom_core::db::SeatloomDb;

    /// Connect to a SCRATCH database, or return `None` to skip. Refuses the live
    /// `seatloom` database outright — these tests write fixtures.
    async fn scratch_pool() -> Option<Pool> {
        let url = std::env::var(DATABASE_URL_ENV).ok()?;
        let dbname = url.rsplit('/').next().unwrap_or("");
        if dbname == "seatloom" || dbname.starts_with("seatloom?") {
            eprintln!(
                "REFUSING live `seatloom` DB for lineage tests — set DATABASE_URL to a scratch DB"
            );
            return None;
        }
        let pool = create_pool().ok()?;
        let client = pool.get().await.ok()?;
        // Require 014 (acceptances) + 016 (acceptance_plan); else skip.
        let row = client
            .query_one(
                "SELECT to_regclass('public.acceptances') IS NOT NULL \
                    AND EXISTS (SELECT 1 FROM information_schema.columns \
                                 WHERE table_name='acceptances' AND column_name='acceptance_plan')",
                &[],
            )
            .await
            .ok()?;
        if !row.get::<_, bool>(0) {
            return None;
        }
        Some(pool)
    }

    const PROJECT: &str = "lineage-itest";
    const DISPATCH: &str = "disp-lineage-itest";
    const ACC: &str = "acc-disp-lineage-itest";
    const VERDICT: &str = "ev-lineage-verdict";
    const PRODUCT: &str = "ar-lineage-product";
    const DOC: &str = "doc-lineage-product";
    const REWORK: &str = "disp-lineage-rework";

    /// A shape-v1 plan (the 016 CHECK rejects `{}` for new rows, which is the
    /// point of the migration).
    fn plan() -> serde_json::Value {
        serde_json::json!({
            "plan_version": 1,
            "scenario": "operator verifies the delivered slice",
            "steps": [{ "action": "open the card", "input": "none" }],
            "expected": [{ "output": "lineage renders", "expect": "seven blocks visible" }]
        })
    }

    /// Clean, then seed one full lineage: a done dispatch with the original task
    /// payload, its lifecycle events, a Flux verdict, an artifact + a document
    /// product, one dependency edge, and a pending acceptance carrying a plan.
    async fn seed(pool: &Pool) {
        let c = pool.get().await.expect("client");
        for sql in [
            format!("DELETE FROM acceptances WHERE project_id = '{PROJECT}'"),
            format!("DELETE FROM dispatch_dependencies WHERE dispatch_id IN (SELECT id FROM dispatches WHERE project_id = '{PROJECT}')"),
            format!("DELETE FROM dispatches WHERE project_id = '{PROJECT}'"),
            format!("DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id = '{PROJECT}')"),
            format!("DELETE FROM canonical_events WHERE project_id = '{PROJECT}'"),
            format!("DELETE FROM artifacts WHERE id = '{PRODUCT}'"),
            format!("DELETE FROM documents WHERE id = '{DOC}'"),
        ] {
            c.execute(&sql, &[]).await.expect("clean");
        }
        c.execute(
            "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
             VALUES ($1,$1,NOW(),1024,1024) ON CONFLICT (id) DO NOTHING",
            &[&PROJECT],
        )
        .await
        .expect("project");

        // The dispatch — the ORIGINAL task statement the card never showed.
        c.execute(
            "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at,completed_at) \
             VALUES ($1,'seat-nimbus-001','seat:seat-lyra-001',$2,'lineage task',\
                     '{\"packet\":\"docs/coordination/tasks/nimbus/x.md\",\"note\":\"the original words\"}',\
                     'done',5,NOW(),NOW())",
            &[&DISPATCH, &PROJECT],
        )
        .await
        .expect("dispatch");

        // A rework dispatch (for the reject-lineage case, exercised below).
        c.execute(
            "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at) \
             VALUES ($1,'seat-nimbus-001','human:operator',$2,'打回重做:lineage task',('{\"rework_of\":\"' || $3 || '\"}')::jsonb,'ready',15,NOW())",
            &[&REWORK, &PROJECT, &DISPATCH],
        )
        .await
        .expect("rework");

        // One dependency edge whose referent EXISTS (the dispatch itself) and
        // the existence answer is assembled per edge.
        c.execute(
            "INSERT INTO dispatch_dependencies (dispatch_id, dep_kind, dep_value) \
             VALUES ($1,'dispatch',$2)",
            &[&DISPATCH, &REWORK],
        )
        .await
        .expect("dep");

        // The Flux verdict event, referenced by the card.
        c.execute(
            "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
             VALUES ($1,'ReviewVerdictIssued',NOW(),'seat:seat-flux-001',\
                     '{\"verdict\":\"pass\",\"evidence\":\"gates green\",\"evidence_detail\":\"tsc+vitest\",\
                       \"target_branch\":\"track/infra-foundation\",\"lane\":\"nimbus\"}',$2,1)",
            &[&VERDICT, &PROJECT],
        )
        .await
        .expect("verdict");

        // A product artifact + document, referenced by an ArtifactCreated event.
        c.execute(
            "INSERT INTO artifacts (id,template,subtype,subtype_valid,title,storage_path,created_at) \
             VALUES ($1,'T3','implementation-delivery',true,'the delivery doc','docs/x.md',NOW())",
            &[&PRODUCT],
        )
        .await
        .expect("artifact");
        c.execute(
            "INSERT INTO documents (id,project_id,title,file_path,parse_status,tags,depends_on) \
             VALUES ($1,$2,'the document','docs/y.md','ok','{}','{}')",
            &[&DOC, &PROJECT],
        )
        .await
        .expect("document");
        // The products, each produced by its own ArtifactCreated that names the
        // DISPATCH as context (the emission convention: subject = the product,
        // context = the work that produced it). One subject per event (010's
        // partial unique index), so the document gets its own event.
        for (ev, rt, rid) in [
            ("ev-lineage-product-artifact", "artifact", PRODUCT),
            ("ev-lineage-product-document", "document", DOC),
        ] {
            c.execute(
                "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
                 VALUES ($1,'ArtifactCreated',NOW(),'seat:seat-nimbus-001','{\"title\":\"p\"}',$2,1)",
                &[&ev, &PROJECT],
            )
            .await
            .expect("product event");
            c.execute(
                "INSERT INTO event_object_refs (event_id,ref_type,ref_id) VALUES ($1,$2,$3)",
                &[&ev, &rt, &rid],
            )
            .await
            .expect("product subject ref");
            c.execute(
                "INSERT INTO event_object_refs (event_id,ref_type,ref_id) VALUES ($1,'dispatch',$2)",
                &[&ev, &DISPATCH],
            )
            .await
            .expect("product context ref");
        }

        // A later event that names ONLY the artifact (no dispatch ref). It is
        // reachable through hop 2 of the assembler — the artifact was discovered
        // in hop 1, so the events naming it join the chain.
        c.execute(
            "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
             VALUES ('ev-lineage-artifact-note','ReviewVerdictIssued',NOW(),'seat:seat-flux-001',\
                     '{\"verdict\":\"pass\",\"evidence\":\"artifact reviewed\"}',$1,1)",
            &[&PROJECT],
        )
        .await
        .expect("artifact note event");
        c.execute(
            "INSERT INTO event_object_refs (event_id,ref_type,ref_id) \
             VALUES ('ev-lineage-artifact-note','artifact',$1)",
            &[&PRODUCT],
        )
        .await
        .expect("artifact note ref");

        // The dispatch's lifecycle chain + the acceptance events, each naming
        // its object as a ref (that is how the assembler walks the chain).
        for (id, ty) in [
            ("ev-lineage-issued", "SeatDispatchIssued"),
            ("ev-lineage-delivered", "SeatDispatchDelivered"),
            ("ev-lineage-completed", "SeatDispatchCompleted"),
        ] {
            c.execute(
                "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
                 VALUES ($1,$2,NOW(),'seat:seat-nimbus-001','{}',$3,1)",
                &[&id, &ty, &PROJECT],
            )
            .await
            .expect("lifecycle event");
            c.execute(
                "INSERT INTO event_object_refs (event_id,ref_type,ref_id) VALUES ($1,'dispatch',$2)",
                &[&id, &DISPATCH],
            )
            .await
            .expect("lifecycle ref");
        }
        // jsonb_build_object rather than a literal with $n inside: a parameter
        // cannot be interpolated into a JSON literal in SQL.
        c.execute(
            "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
             VALUES ('ev-lineage-requested','AcceptanceRequested',NOW(),'seat:seat-nimbus-001', \
                     jsonb_build_object('dispatch_id',$1::text,'branch','packet/nimbus/x', \
                       'base','track/infra-foundation','how_to_verify','do A see B', \
                       'acceptance_plan',$2::jsonb), \
                     $3,1)",
            &[&DISPATCH, &plan(), &PROJECT],
        )
        .await
        .expect("requested event");
        c.execute(
            "INSERT INTO event_object_refs (event_id,ref_type,ref_id) \
             VALUES ('ev-lineage-requested','acceptance',$1)",
            &[&ACC],
        )
        .await
        .expect("requested ref");

        // The card itself, carrying the plan.
        c.execute(
            "INSERT INTO acceptances (id,dispatch_id,project_id,branch,base,how_to_verify,flux_verdict_ref,status,acceptance_plan) \
             VALUES ($1,$2,$3,'packet/nimbus/x','track/infra-foundation','do A see B',$4,'pending',$5)",
            &[&ACC, &DISPATCH, &PROJECT, &VERDICT, &plan()],
        )
        .await
        .expect("acceptance");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 001-014 + 016 (scratch DB)"]
    async fn assembles_all_seven_blocks_from_the_ledger() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        seed(&pool).await;
        let db = SeatloomDb::new(pool.clone());

        let l = get_acceptance_lineage(&db, ACC)
            .await
            .expect("the seeded card assembles");

        // 1. the card, with its plan.
        assert_eq!(l.acceptance.id, ACC);
        assert_eq!(l.acceptance.status, "pending");
        assert_eq!(l.acceptance.base, "track/infra-foundation");
        let plan = l
            .acceptance
            .acceptance_plan
            .expect("the plan rides the card");
        assert_eq!(plan["scenario"], "operator verifies the delivered slice");

        // 2. the dispatch — the ORIGINAL task statement, verbatim.
        assert_eq!(l.dispatch.id, DISPATCH);
        assert_eq!(l.dispatch.issued_by, "seat:seat-lyra-001");
        assert_eq!(l.dispatch.payload["note"], "the original words");

        // 3. the event chain, in occurred_at order, including the verdict the
        //    card names (it is pulled in via flux_verdict_ref).
        let types: Vec<&str> = l.events.iter().map(|e| e.event_type.as_str()).collect();
        for expected in [
            "SeatDispatchIssued",
            "SeatDispatchDelivered",
            "SeatDispatchCompleted",
            "ArtifactCreated",
            "AcceptanceRequested",
            "ReviewVerdictIssued",
        ] {
            assert!(types.contains(&expected), "missing {expected} in {types:?}");
        }
        // Two ArtifactCreated events (one per product) and TWO verdict events:
        // the card's own, plus the artifact note that hop 2 pulled in. The
        // seeded chain is 3 lifecycle + 2 product + 1 requested + 2 verdict = 8.
        assert_eq!(
            types.iter().filter(|t| **t == "ArtifactCreated").count(),
            2,
            "one per product: {types:?}"
        );
        assert_eq!(
            types
                .iter()
                .filter(|t| **t == "ReviewVerdictIssued")
                .count(),
            2,
            "the card's verdict + the hop-2 artifact note: {types:?}"
        );
        assert_eq!(l.events.len(), 8, "the full chain: {types:?}");
        assert!(
            l.events.iter().any(|e| e.id == "ev-lineage-artifact-note"),
            "the artifact-only note joins the chain via hop 2: {types:?}"
        );

        // 4. the Flux verdict, rendered from its payload.
        let v = l.flux_verdict.expect("the verdict block");
        assert_eq!(v.ref_id, VERDICT);
        assert_eq!(v.verdict.as_deref(), Some("pass"));
        assert_eq!(v.evidence.as_deref(), Some("gates green"));
        assert_eq!(v.evidence_detail.as_deref(), Some("tsc+vitest"));
        assert_eq!(v.target_branch.as_deref(), Some("track/infra-foundation"));
        assert_eq!(v.lane.as_deref(), Some("nimbus"));

        // 5. the products: both the artifact and the document, resolved.
        assert_eq!(l.products.len(), 2, "{:?}", l.products);
        assert!(l.products.iter().all(|p| p.resolved));
        assert!(l.products.iter().any(|p| p.ref_type == "artifact"
            && p.title.as_deref() == Some("the delivery doc")
            && p.path.as_deref() == Some("docs/x.md")));
        assert!(l
            .products
            .iter()
            .any(|p| p.ref_type == "document" && p.path.as_deref() == Some("docs/y.md")));

        // 6. the dependency edges, with the referent's live existence.
        assert_eq!(l.dependencies.len(), 1);
        assert_eq!(l.dependencies[0].dep_kind, "dispatch");
        assert_eq!(l.dependencies[0].dep_value, REWORK);
        assert!(l.dependencies[0].exists);

        // 7. no rework on a pending card.
        assert!(l.rework.is_none());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 001-014 + 016 (scratch DB)"]
    async fn a_rejected_carries_its_rework_and_a_void_carries_its_reason() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        seed(&pool).await;
        let db = SeatloomDb::new(pool.clone());
        let c = pool.get().await.expect("client");

        // Reject the card with the rework lineage.
        c.execute(
            "UPDATE acceptances SET status='rejected', feedback='fix the edge case', \
                    rework_dispatch_id=$2, decided_by='human:operator', decided_at=NOW() \
              WHERE id=$1",
            &[&ACC, &REWORK],
        )
        .await
        .expect("reject");
        let l = get_acceptance_lineage(&db, ACC).await.expect("assemble");
        let rework = l.rework.expect("the rework block");
        assert_eq!(rework.id, REWORK);
        assert_eq!(l.acceptance.feedback.as_deref(), Some("fix the edge case"));

        // A void (017) records its reason and still assembles.
        c.execute(
            "UPDATE acceptances SET status='cancelled', feedback='already landed via the bootstrap install' \
              WHERE id=$1",
            &[&ACC],
        )
        .await
        .expect("cancel");
        let l = get_acceptance_lineage(&db, ACC).await.expect("assemble");
        assert_eq!(l.acceptance.status, "cancelled");
        assert_eq!(
            l.acceptance.feedback.as_deref(),
            Some("already landed via the bootstrap install")
        );
    }

    /// An unknown id is a typed error, not an empty card — a mistyped id must
    /// say so.
    #[tokio::test]
    #[ignore = "requires PostgreSQL with 001-014 + 016 (scratch DB)"]
    async fn an_unknown_acceptance_is_a_typed_refusal() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let err = get_acceptance_lineage(&db, "acc-does-not-exist")
            .await
            .expect_err("unknown id");
        assert!(
            matches!(err, LineageError::NotFound { .. }),
            "expected NotFound, got {err:?}"
        );
    }
}
