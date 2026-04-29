//! Integration tests for the PostgreSQL real collaboration truth baseline.
//!
//! These tests require a running PostgreSQL instance seeded with the real collaboration data.
//! They are marked `#[ignore]` so that `cargo test -p seatloom-core` passes without a DB.
//!
//! To run: ensure docker compose is up (cd infra/postgres && docker compose up -d),
//! then run: cargo test -p seatloom-core -- --include-ignored
//!
//! Or use: scripts/verify-postgres-baseline.sh

#[cfg(test)]
mod db_baseline {
    use seatloom_core::db::connection::{create_pool, DEFAULT_DATABASE_URL};
    use seatloom_core::db::repositories::SeatloomDb;

    async fn try_connect() -> Option<SeatloomDb> {
        let pool = create_pool().ok()?;
        let db = SeatloomDb::new(pool);
        // Quick ping to see if the DB is available
        if db.ping().await.is_ok() {
            Some(db)
        } else {
            None
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run: cd infra/postgres && docker compose up -d"]
    async fn db_ping_succeeds() {
        let db = match try_connect().await {
            Some(d) => d,
            None => panic!("Cannot connect to {DEFAULT_DATABASE_URL}"),
        };
        db.ping().await.expect("postgres ping should succeed");
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline"]
    async fn seats_non_empty_and_contain_all_five() {
        let db = try_connect().await.expect("DB must be reachable");
        let seats = db.list_seats().await.expect("seat list must succeed");
        assert!(
            seats.len() >= 5,
            "expected ≥5 seeded seats, got {}",
            seats.len()
        );
        let names: Vec<&str> = seats.iter().map(|s| s.name.as_str()).collect();
        for expected in &["aegis", "lyra", "mira", "nimbus", "flux"] {
            assert!(
                names.contains(expected),
                "missing expected seat: {expected}"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline"]
    async fn flux_mira_delegation_persisted() {
        let db = try_connect().await.expect("DB must be reachable");
        let del = db
            .get_delegation("del-flux-acting-mira-001")
            .await
            .expect("delegation query must succeed");
        let del = del.expect("delegation del-flux-acting-mira-001 must exist");
        assert_eq!(del.from_seat_id, "seat-mira-001");
        assert_eq!(del.to_seat_id, "seat-flux-001");
        assert_eq!(del.status, "closed");
        assert_eq!(del.workitem_id.as_deref(), Some("wi-009"));
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline"]
    async fn all_six_workitems_seeded_as_done() {
        let db = try_connect().await.expect("DB must be reachable");
        let items = db
            .list_workitems()
            .await
            .expect("workitem list must succeed");
        assert!(
            items.len() >= 6,
            "expected ≥6 seeded workitems, got {}",
            items.len()
        );
        let done_ids: Vec<&str> = items
            .iter()
            .filter(|w| w.status == "done")
            .map(|w| w.id.as_str())
            .collect();
        for wid in &["wi-001", "wi-scaffold", "wi-storage", "wi-hardening"] {
            assert!(done_ids.contains(wid), "workitem {wid} should be done");
        }
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline"]
    async fn artifacts_cover_t1_through_t7() {
        let db = try_connect().await.expect("DB must be reachable");
        let all = db
            .list_artifacts(None, None)
            .await
            .expect("artifact list must succeed");
        assert!(!all.is_empty(), "artifact list must not be empty");
        let templates: Vec<Option<&str>> = all.iter().map(|a| a.template.as_deref()).collect();
        for expected in &[
            "T1AuthorityDoc",
            "T2RoleProfile",
            "T3TaskPacket",
            "T4Review",
            "T5Acceptance",
            "T6DailyMemory",
            "T7GovernanceDoc",
        ] {
            assert!(
                templates.contains(&Some(expected)),
                "missing artifact family: {expected}"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline"]
    async fn active_contract_set_present() {
        let db = try_connect().await.expect("DB must be reachable");
        for ar_id in &[
            "ar-prd-v05",
            "ar-interaction-v11",
            "ar-ux-v11",
            "ar-acceptance-v11",
            "ar-arch-decisions",
            "ar-arch-design",
            "ar-product-truth",
        ] {
            let row = db.get_artifact(ar_id).await.expect("query must succeed");
            assert!(row.is_some(), "artifact {ar_id} must be seeded");
        }
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline"]
    async fn timeline_events_descending_order() {
        let db = try_connect().await.expect("DB must be reachable");
        let events = db.list_events(50).await.expect("event list must succeed");
        assert!(!events.is_empty(), "event list must not be empty");
        for pair in events.windows(2) {
            assert!(
                pair[0].occurred_at >= pair[1].occurred_at,
                "events not in descending order"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline"]
    async fn handoffs_reference_valid_workitems() {
        let db = try_connect().await.expect("DB must be reachable");
        let handoffs = db.list_handoffs().await.expect("handoff list must succeed");
        assert!(
            handoffs.len() >= 3,
            "expected ≥3 seeded handoffs, got {}",
            handoffs.len()
        );
        for handoff in &handoffs {
            let wi = db
                .get_workitem(&handoff.workitem_id)
                .await
                .expect("workitem query must succeed");
            assert!(
                wi.is_some(),
                "handoff {} references missing workitem {}",
                handoff.id,
                handoff.workitem_id
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline"]
    async fn role_bindings_cover_all_seats() {
        let db = try_connect().await.expect("DB must be reachable");
        let bindings = db
            .list_role_bindings_for_project("seatloom")
            .await
            .expect("role binding list must succeed");
        assert!(
            bindings.len() >= 5,
            "expected ≥5 role bindings, got {}",
            bindings.len()
        );
    }

    // =========================================================================
    // Document authority layer (schema 002)
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 002_document_seed.sql applied"]
    async fn documents_seeded_for_project() {
        let db = try_connect().await.expect("DB must be reachable");
        let docs = db
            .list_documents("seatloom", None, None)
            .await
            .expect("document list must succeed");
        assert!(
            docs.len() >= 11,
            "expected ≥11 seeded documents, got {}",
            docs.len()
        );
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 002_document_seed.sql applied"]
    async fn documents_cover_t1_through_t7() {
        let db = try_connect().await.expect("DB must be reachable");
        let all = db
            .list_documents("seatloom", None, None)
            .await
            .expect("document list must succeed");
        let templates: Vec<Option<&str>> = all.iter().map(|d| d.template.as_deref()).collect();
        for expected in &[
            "T1AuthorityDoc",
            "T2RoleProfile",
            "T3TaskPacket",
            "T4Review",
            "T5Acceptance",
            "T6DailyMemory",
            "T7GovernanceDoc",
        ] {
            assert!(
                templates.contains(&Some(expected)),
                "missing document family: {expected}"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 002_document_seed.sql applied"]
    async fn active_contract_set_documents_present() {
        let db = try_connect().await.expect("DB must be reachable");
        for doc_id in &[
            "doc-prd-v05",
            "doc-interaction-v11",
            "doc-ux-v11",
            "doc-acceptance-v11",
            "doc-arch-decisions",
            "doc-arch-design",
            "doc-product-truth",
        ] {
            let doc = db.get_document(doc_id).await.expect("query must succeed");
            assert!(doc.is_some(), "document {doc_id} must be seeded");
        }
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 002_document_seed.sql applied"]
    async fn document_sections_seeded_and_ordered() {
        let db = try_connect().await.expect("DB must be reachable");
        let sections = db
            .list_document_sections("doc-prd-v05")
            .await
            .expect("sections must succeed");
        assert!(
            sections.len() >= 2,
            "expected ≥2 sections for PRD doc, got {}",
            sections.len()
        );
        // Sections must be in ascending ordinal order
        for pair in sections.windows(2) {
            assert!(
                pair[0].ordinal < pair[1].ordinal,
                "sections not in ordinal order"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 002_document_seed.sql applied"]
    async fn document_associations_link_to_project_and_workitems() {
        let db = try_connect().await.expect("DB must be reachable");
        // PRD should be associated with the project
        let assocs = db
            .list_document_associations("doc-prd-v05")
            .await
            .expect("associations must succeed");
        assert!(!assocs.is_empty(), "PRD must have at least one association");
        let project_assoc = assocs.iter().find(|a| a.assoc_type == "project");
        assert!(
            project_assoc.is_some(),
            "PRD must have project-scope association"
        );

        // Task packet should be associated with its workitem
        let task_assocs = db
            .list_document_associations("doc-task-hardening")
            .await
            .expect("task associations must succeed");
        let wi_assoc = task_assocs.iter().find(|a| a.assoc_type == "workitem");
        assert!(
            wi_assoc.is_some(),
            "task packet must have workitem association"
        );
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 002_document_seed.sql applied"]
    async fn document_template_filter_works() {
        let db = try_connect().await.expect("DB must be reachable");
        let t1_docs = db
            .list_documents("seatloom", Some("T1AuthorityDoc"), None)
            .await
            .expect("T1 filter must succeed");
        assert!(
            t1_docs.len() >= 7,
            "expected ≥7 T1 docs (active contract set), got {}",
            t1_docs.len()
        );
        for doc in &t1_docs {
            assert_eq!(
                doc.template.as_deref(),
                Some("T1AuthorityDoc"),
                "filter must return only T1 docs"
            );
        }
    }

    // =========================================================================
    // Reconcile bookkeeping (schema 003)
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 003 schema + a completed reconcile run"]
    async fn reconcile_run_recorded_after_run() {
        let db = try_connect().await.expect("DB must be reachable");
        let runs = db
            .list_reconcile_runs(10)
            .await
            .expect("reconcile run list must succeed");
        // After at least one reconcile run, this should be non-empty
        assert!(
            !runs.is_empty(),
            "expected ≥1 reconcile run after seatloom reconcile was executed"
        );
        let run = &runs[0];
        assert!(
            matches!(run.status.as_str(), "completed" | "failed"),
            "run status must be terminal"
        );
        assert!(run.scanned > 0, "reconcile must scan at least one file");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 003 schema + a completed reconcile run"]
    async fn reconcile_items_reference_docs_folder() {
        let db = try_connect().await.expect("DB must be reachable");
        let runs = db
            .list_reconcile_runs(1)
            .await
            .expect("reconcile run list must succeed");
        if runs.is_empty() {
            return; // No runs yet — skip
        }
        let items = db
            .list_reconcile_items(&runs[0].id)
            .await
            .expect("reconcile items must succeed");
        for item in &items {
            assert!(
                item.file_path.starts_with("docs/"),
                "reconcile items must only reference docs/ files, got: {}",
                item.file_path
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 003 schema; run reconcile twice to verify"]
    async fn second_reconcile_run_unchanged_for_unmodified_docs() {
        let db = try_connect().await.expect("DB must be reachable");
        let runs = db
            .list_reconcile_runs(10)
            .await
            .expect("reconcile run list must succeed");
        if runs.len() < 2 {
            return; // Need at least 2 runs
        }
        // Latest run (index 0) should show mostly unchanged items if docs didn't change
        let latest_run = &runs[0];
        assert_eq!(
            latest_run.inserted, 0,
            "second run should insert nothing if docs haven't changed"
        );
        assert!(
            latest_run.unchanged > 0,
            "second run should show unchanged items"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 003 schema + reconcile having run"]
    async fn document_versions_created_on_first_reconcile() {
        let db = try_connect().await.expect("DB must be reachable");
        // Any T1 document that was inserted by reconcile should have revision ≥ 1
        let docs = db
            .list_documents("seatloom", Some("T1AuthorityDoc"), None)
            .await
            .expect("document list must succeed");
        for doc in &docs {
            assert!(doc.revision >= 1, "every document must have revision ≥ 1");
            let versions = db
                .list_document_versions(&doc.id)
                .await
                .expect("version list must succeed");
            assert!(
                !versions.is_empty(),
                "document {} must have at least one version snapshot",
                doc.id
            );
        }
    }
}
