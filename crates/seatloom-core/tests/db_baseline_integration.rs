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

    // =========================================================================
    // Operational review + continuity layer (schema 004)
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 003_operational_review_and_continuity_seed.sql applied"]
    async fn checkpoints_seeded_and_session_backlinks_updated() {
        let db = try_connect().await.expect("DB must be reachable");
        let checkpoints = db
            .list_checkpoints(10)
            .await
            .expect("checkpoint list must succeed");
        assert!(
            checkpoints.len() >= 3,
            "expected ≥3 seeded checkpoints, got {}",
            checkpoints.len()
        );

        let nimbus = db
            .get_checkpoint("cp-nimbus-infra-001")
            .await
            .expect("checkpoint query must succeed")
            .expect("seeded Nimbus checkpoint must exist");
        assert_eq!(nimbus.session_id, "ses-nimbus-infra-001");
        assert_eq!(nimbus.trigger, "session_ended");
        assert_eq!(nimbus.summary_quality, "full");

        let sessions = db
            .list_sessions_for_seat("seat-nimbus-001")
            .await
            .expect("session list must succeed");
        let active = sessions
            .iter()
            .find(|session| session.id == "ses-nimbus-infra-001")
            .expect("seeded Nimbus session must exist");
        assert_eq!(
            active.last_checkpoint_id.as_deref(),
            Some("cp-nimbus-infra-001")
        );
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 003_operational_review_and_continuity_seed.sql applied"]
    async fn handoff_receipts_seeded_for_real_handoffs() {
        let db = try_connect().await.expect("DB must be reachable");
        let receipts = db
            .list_handoff_receipts_for_handoff("ho-nimbus-lyra-storage")
            .await
            .expect("receipt list must succeed");
        assert_eq!(
            receipts.len(),
            1,
            "expected one receipt for storage handoff"
        );
        let receipt = &receipts[0];
        assert_eq!(receipt.id, "hr-ho-nimbus-lyra-storage-001");
        assert_eq!(receipt.acknowledged_by, "seat-lyra-001");
        assert_eq!(receipt.source_channel, "desktop");
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 003_operational_review_and_continuity_seed.sql applied"]
    async fn pipeline_run_seeded_for_foundation_gate() {
        let db = try_connect().await.expect("DB must be reachable");
        let run = db
            .get_pipeline_run("plrun-rust-foundation-gate-001")
            .await
            .expect("pipeline run query must succeed")
            .expect("seeded pipeline run must exist");
        assert_eq!(run.pipeline_id, "pl-rust-foundation-gate");
        assert_eq!(run.workitem_id.as_deref(), Some("wi-hardening"));
        assert_eq!(run.status, "completed");
        assert_eq!(run.trigger, "gate");
        assert!(
            run.evidence_refs
                .iter()
                .any(|e| e.contains("foundation-hardening-verification")),
            "pipeline run must retain verification evidence refs"
        );
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 003_operational_review_and_continuity_seed.sql applied"]
    async fn review_thread_seeded_with_l3_change_tier_record() {
        let db = try_connect().await.expect("DB must be reachable");
        let threads = db
            .list_review_threads_for_workitem("wi-004")
            .await
            .expect("review thread list must succeed");
        assert_eq!(
            threads.len(),
            1,
            "expected one seeded review thread for wi-004"
        );
        let thread = &threads[0];
        assert_eq!(thread.id, "rt-wi004-sg01-gap-001");
        assert_eq!(thread.status, "resolved");
        assert_eq!(thread.review_tier.as_deref(), Some("L3"));
        let record = thread
            .change_tier_record
            .as_ref()
            .expect("L3 change tier record must be present");
        assert_eq!(record["ack_mode"], "full_gate");
        assert!(record["changed_clauses"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires seeded PostgreSQL baseline with 003_operational_review_and_continuity_seed.sql applied"]
    async fn review_comments_seeded_and_ordered() {
        let db = try_connect().await.expect("DB must be reachable");
        let comments = db
            .list_review_comments("rt-wi004-sg01-gap-001")
            .await
            .expect("review comment list must succeed");
        assert_eq!(comments.len(), 2, "expected two seeded review comments");
        assert_eq!(comments[0].id, "rc-wi004-sg01-gap-001");
        assert_eq!(
            comments[1].parent_comment_id.as_deref(),
            Some("rc-wi004-sg01-gap-001")
        );
        assert!(
            comments[0].created_at <= comments[1].created_at,
            "comments must be returned in chronological order"
        );
    }

    // =========================================================================
    // Schema 005: Prompt + channel action authority (zero-row seed baseline)
    // These tests prove the new families exist and the write/read path is real.
    // They use in-test fixtures only; the seed is intentionally empty.
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 005_prompt_and_channel_action_authority.sql applied"]
    async fn prompt_instance_tables_exist_and_empty_at_baseline() {
        let db = try_connect().await.expect("DB must be reachable");
        // Baseline seed is zero-row; structural proof only.
        let instances = db
            .list_active_prompt_instances("seatloom")
            .await
            .expect("prompt instance list must succeed");
        assert_eq!(
            instances.len(),
            0,
            "no fabricated prompt rows in baseline seed"
        );
        let receipts = db
            .list_channel_action_receipts("seatloom", None, None)
            .await
            .expect("channel action receipt list must succeed");
        assert_eq!(
            receipts.len(),
            0,
            "no fabricated channel action receipt rows in baseline seed"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 005_prompt_and_channel_action_authority.sql applied"]
    async fn prompt_instance_write_read_round_trip() {
        use chrono::Utc;
        use seatloom_core::db::models::PromptInstanceRow;

        let db = try_connect().await.expect("DB must be reachable");
        let now = Utc::now();
        let fixture = PromptInstanceRow {
            id: "pi-test-001".to_string(),
            project_id: "seatloom".to_string(),
            session_id: "ses-nimbus-infra-001".to_string(),
            status: "active".to_string(),
            prompt_kind: "freeform".to_string(),
            prompt_policy: "needs_approval".to_string(),
            evidence_ref: Some("ref-bounded-window-001".to_string()),
            evidence_preview: Some("$ git commit -m \"".to_string()),
            available_actions: vec!["approve".to_string(), "human_takeover".to_string()],
            assist_max_steps: Some(10),
            assist_max_tokens: Some(2000),
            assist_steps_used: 0,
            assist_tokens_used: 0,
            expected_next_pattern: Some("Enter commit message".to_string()),
            detected_at: now,
            resolved_at: None,
            resolved_by: None,
            result_event_id: None,
            created_at: now,
            updated_at: now,
        };

        db.create_prompt_instance(&fixture)
            .await
            .expect("prompt instance insert must succeed");

        let fetched = db
            .get_prompt_instance("pi-test-001")
            .await
            .expect("get must succeed")
            .expect("row must be present");

        assert_eq!(fetched.id, "pi-test-001");
        assert_eq!(fetched.prompt_kind, "freeform");
        assert_eq!(fetched.prompt_policy, "needs_approval");
        assert_eq!(fetched.available_actions.len(), 2);
        assert!(fetched.available_actions.contains(&"approve".to_string()));

        let by_session = db
            .list_prompt_instances_for_session("ses-nimbus-infra-001")
            .await
            .expect("session list must succeed");
        assert!(
            by_session.iter().any(|p| p.id == "pi-test-001"),
            "instance must appear in session list"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 005_prompt_and_channel_action_authority.sql applied; run after prompt_instance_write_read_round_trip"]
    async fn prompt_action_append_and_list() {
        use chrono::Utc;
        use seatloom_core::db::models::PromptActionRow;

        let db = try_connect().await.expect("DB must be reachable");
        let now = Utc::now();
        let action = PromptActionRow {
            id: "pa-test-001".to_string(),
            prompt_id: "pi-test-001".to_string(),
            action_kind: "approve".to_string(),
            actor_ref: "lyra".to_string(),
            source_channel: "desktop".to_string(),
            note: Some("Approved via desktop review".to_string()),
            steps_budget_used: None,
            tokens_budget_used: None,
            result_status: "applied".to_string(),
            result_event_id: None,
            created_at: now,
        };

        db.append_prompt_action(&action)
            .await
            .expect("prompt action insert must succeed");

        let actions = db
            .list_prompt_actions_for_prompt("pi-test-001")
            .await
            .expect("action list must succeed");

        assert!(
            actions.iter().any(|a| a.id == "pa-test-001"),
            "appended action must appear in list"
        );
        let fetched = actions.iter().find(|a| a.id == "pa-test-001").unwrap();
        assert_eq!(fetched.action_kind, "approve");
        assert_eq!(fetched.source_channel, "desktop");
        assert_eq!(fetched.result_status, "applied");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 005_prompt_and_channel_action_authority.sql applied"]
    async fn channel_action_receipt_write_read_and_idempotency() {
        use chrono::Utc;
        use seatloom_core::db::models::ChannelActionReceiptRow;

        let db = try_connect().await.expect("DB must be reachable");
        let now = Utc::now();
        let receipt = ChannelActionReceiptRow {
            id: "car-test-001".to_string(),
            project_id: "seatloom".to_string(),
            target_kind: "workitem".to_string(),
            target_id: "wi-hardening".to_string(),
            action_kind: "approve".to_string(),
            actor_ref: "lyra".to_string(),
            source_channel: "mobile".to_string(),
            note: Some("Approved from mobile during standup".to_string()),
            evidence_refs: vec!["docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md".to_string()],
            policy_summary: Some("P0 acceptance gate passed".to_string()),
            idempotency_key: "approve-wi-hardening-lyra-20260430T120000Z".to_string(),
            expected_revision: Some(1),
            applied_revision: Some(2),
            receipt_status: "applied".to_string(),
            result_event_id: None,
            created_at: now,
        };

        db.create_channel_action_receipt(&receipt)
            .await
            .expect("receipt insert must succeed");

        let fetched = db
            .get_channel_action_receipt("car-test-001")
            .await
            .expect("get must succeed")
            .expect("row must be present");

        assert_eq!(fetched.id, "car-test-001");
        assert_eq!(fetched.source_channel, "mobile");
        assert_eq!(fetched.receipt_status, "applied");
        assert_eq!(
            fetched.idempotency_key,
            "approve-wi-hardening-lyra-20260430T120000Z"
        );

        let by_target = db
            .list_channel_action_receipts_for_target("workitem", "wi-hardening")
            .await
            .expect("target list must succeed");
        assert!(
            by_target.iter().any(|r| r.id == "car-test-001"),
            "receipt must appear in target list"
        );

        let mobile_receipts = db
            .list_channel_action_receipts("seatloom", Some("mobile"), None)
            .await
            .expect("channel filter list must succeed");
        assert!(
            mobile_receipts.iter().any(|r| r.id == "car-test-001"),
            "receipt must appear in mobile-filtered list"
        );

        // Idempotency: duplicate insert must fail (unique constraint on idempotency_key)
        let dup = ChannelActionReceiptRow {
            id: "car-test-002".to_string(),
            idempotency_key: "approve-wi-hardening-lyra-20260430T120000Z".to_string(),
            ..receipt.clone()
        };
        let result = db.create_channel_action_receipt(&dup).await;
        assert!(
            result.is_err(),
            "duplicate idempotency_key must be rejected"
        );
    }
}
