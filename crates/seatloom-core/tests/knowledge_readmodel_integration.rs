//! Knowledge read model integration tests (overnight step 4) for the two pure
//! reads: the topic tree with memory counts, and the memories-by-topic card
//! list with progressive disclosure and the ancestor chain.
//!
//! These require a running PostgreSQL with migrations 001–018 applied, so they
//! are `#[ignore]` — `cargo test -p seatloom-core` passes without a DB. They
//! REFUSE to run against the live `seatloom` database (name guard): point them
//! at a scratch DB, e.g.
//!
//!   DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_km \
//!     cargo test -p seatloom-core --test knowledge_readmodel_integration -- --include-ignored
//!
//! The reads are PURE: the tests assert that the ledger does not move.

#[cfg(test)]
mod knowledge {
    use deadpool_postgres::Pool;
    use seatloom_core::db::connection::{create_pool, DATABASE_URL_ENV};
    use seatloom_core::db::SeatloomDb;
    use seatloom_core::objects::memory::MemoryLayer;

    async fn scratch_pool() -> Option<Pool> {
        let url = std::env::var(DATABASE_URL_ENV).ok()?;
        let dbname = url.rsplit('/').next().unwrap_or("");
        if dbname == "seatloom" || dbname.starts_with("seatloom?") {
            eprintln!("REFUSING live `seatloom` DB for knowledge tests — set DATABASE_URL to a scratch DB");
            return None;
        }
        let pool = create_pool().ok()?;
        let client = pool.get().await.ok()?;
        let row = client
            .query_one(
                "SELECT to_regclass('public.memories') IS NOT NULL \
                    AND EXISTS (SELECT 1 FROM information_schema.columns \
                                 WHERE table_name='memories' AND column_name='kind')",
                &[],
            )
            .await
            .ok()?;
        if !row.get::<_, bool>(0) {
            return None;
        }
        Some(pool)
    }

    const PROJECT: &str = "km-itest";

    /// A project with a root topic, a child under it, and three memories: one
    /// plain, one superseded by a third, and one forgotten.
    async fn seed(pool: &Pool) {
        let c = pool.get().await.expect("client");
        for sql in [
            format!("DELETE FROM memory_relations WHERE from_id IN (SELECT id FROM memories WHERE project_id='{PROJECT}') OR to_id IN (SELECT id FROM memories WHERE project_id='{PROJECT}')"),
            format!("DELETE FROM memories WHERE project_id='{PROJECT}'"),
            format!("DELETE FROM topics WHERE project_id='{PROJECT}'"),
            format!("DELETE FROM documents WHERE project_id='{PROJECT}'"),
            format!("DELETE FROM event_object_refs WHERE event_id IN (SELECT id FROM canonical_events WHERE project_id='{PROJECT}')"),
            format!("DELETE FROM canonical_events WHERE project_id='{PROJECT}'"),
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
        c.execute(
            "INSERT INTO topics (id,project_id,title,opened_by) VALUES ('km-root',$1,'root','seat:seat-lyra-001') ON CONFLICT (id) DO NOTHING",
            &[&PROJECT],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO topics (id,project_id,title,opened_by,parent_topic_id) \
             VALUES ('km-child',$1,'child','seat:seat-lyra-001','km-root') ON CONFLICT (id) DO NOTHING",
            &[&PROJECT],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO documents (id,project_id,title,file_path,parse_status,tags,depends_on) \
             VALUES ('km-doc',$1,'the doc','docs/km.md','ok','{}','{}') ON CONFLICT (id) DO NOTHING",
            &[&PROJECT],
        )
        .await
        .unwrap();
        for (id, title, status) in [
            ("km-mem-current", "the current decision", "active"),
            ("km-mem-old", "the overturned decision", "active"),
            ("km-mem-forgotten", "the forgotten one", "forgotten"),
        ] {
            c.execute(
                "INSERT INTO memories (id,project_id,topic_id,title,l0_summary,l1_overview,l2_ref_kind, \
                        l2_document_id,source_kind,source_ref,kind,confidence,status,supersedes) \
                 VALUES ($1,$2,'km-child',$3,$3,'the overview','document','km-doc','document','km-doc', \
                        'decision','quoted',$4,NULL)",
                &[&id, &PROJECT, &title, &status],
            )
            .await
            .unwrap();
        }
        // A third memory supersedes the old one: both spellings of the edge.
        c.execute(
            "INSERT INTO memories (id,project_id,topic_id,title,l0_summary,l2_ref_kind,l2_file_path, \
                    l2_line_start,l2_line_end,source_kind,source_ref,kind,confidence,supersedes) \
             VALUES ('km-mem-new',$1,'km-child','the newer decision','the newer summary','file','docs/s.md', \
                     41,57,'archived_session','claude:ses-1','decision','inferred','km-mem-old')",
            &[&PROJECT],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO memory_relations (from_id,to_id,rel) VALUES ('km-mem-new','km-mem-old','supersedes')",
            &[],
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 001-018 (scratch DB)"]
    async fn the_tree_counts_only_active_unsuperseded_memories() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        seed(&pool).await;
        let db = SeatloomDb::new(pool.clone());
        let tree = db.knowledge_topic_tree(PROJECT).await.unwrap();
        let root = tree.iter().find(|t| t.topic_id == "km-root").unwrap();
        let child = tree.iter().find(|t| t.topic_id == "km-child").unwrap();
        assert_eq!(root.parent_topic_id, None);
        assert_eq!(child.parent_topic_id.as_deref(), Some("km-root"));
        assert_eq!(child.status, "open");
        // Four memories live under km-child: the forgotten one, the superseded
        // one, the current one and the new one. Only the two that are active AND
        // not superseded count.
        assert_eq!(
            child.memory_count, 2,
            "the count matches what a recall returns"
        );
        assert_eq!(root.memory_count, 0, "a topic with no memories counts zero");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL with 001-018 (scratch DB)"]
    async fn memories_disclose_by_depth_and_exclude_superseded_by_default() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        seed(&pool).await;
        let db = SeatloomDb::new(pool.clone());

        // Default depth: only the summary, no l1, no l2.
        let cards = db
            .topic_memories("km-child", MemoryLayer::L0, false, 50)
            .await
            .unwrap();
        let ids: Vec<&str> = cards.iter().map(|c| c.memory_id.as_str()).collect();
        assert!(!ids.contains(&"km-mem-old"), "the superseded row is hidden");
        // The forgotten row IS returned — the panel shows a card's status
        // (active|forgotten|resolved), so a tombstone must be visible, not
        // silently dropped. Only the superseded row is excluded by default.
        assert!(
            ids.contains(&"km-mem-forgotten"),
            "the forgotten row is returned with its status, not dropped"
        );
        assert!(ids.contains(&"km-mem-current") && ids.contains(&"km-mem-new"));
        for c in &cards {
            assert!(c.l1_overview.is_none(), "l1 is absent at l0");
            assert!(c.l2.is_none(), "l2 is absent at l0");
            assert!(c.superseded_by.is_none());
        }

        // l1: the overview appears (a memory that has none still has none —
        // absence is data, not a default).
        let cards = db
            .topic_memories("km-child", MemoryLayer::L1, false, 50)
            .await
            .unwrap();
        let current = cards
            .iter()
            .find(|c| c.memory_id == "km-mem-current")
            .expect("the current decision");
        assert_eq!(
            current.l1_overview.as_deref(),
            Some("the overview"),
            "l1 at l1"
        );
        let newer = cards
            .iter()
            .find(|c| c.memory_id == "km-mem-new")
            .expect("the newer decision");
        assert_eq!(
            newer.l1_overview, None,
            "a memory with no overview stays without one"
        );
        assert!(cards.iter().all(|c| c.l2.is_none()), "still no l2 at l1");

        // l2: the pointer appears, with the line range for the transcript slice.
        let cards = db
            .topic_memories("km-child", MemoryLayer::L2, false, 50)
            .await
            .unwrap();
        let new = cards
            .iter()
            .find(|c| c.memory_id == "km-mem-new")
            .expect("the newer decision");
        let l2 = new.l2.as_ref().expect("l2 at l2");
        assert_eq!(l2.ref_kind, "file");
        assert_eq!(l2.file_path.as_deref(), Some("docs/s.md"));
        assert_eq!((l2.line_start, l2.line_end), (Some(41), Some(57)));
        assert_eq!(new.kind, "decision");
        assert_eq!(new.confidence, "inferred");

        // Opting in returns the superseded row, annotated.
        let cards = db
            .topic_memories("km-child", MemoryLayer::L0, true, 50)
            .await
            .unwrap();
        let old = cards
            .iter()
            .find(|c| c.memory_id == "km-mem-old")
            .expect("the superseded row, when asked for");
        assert_eq!(old.superseded_by.as_deref(), Some("km-mem-new"));
    }

    /// The reads are PURE: neither of them writes a row or appends an event.
    #[tokio::test]
    #[ignore = "requires PostgreSQL with 001-018 (scratch DB)"]
    async fn the_knowledge_reads_append_no_events() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        seed(&pool).await;
        let db = SeatloomDb::new(pool.clone());
        let before: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one("SELECT count(*) FROM canonical_events", &[])
            .await
            .unwrap()
            .get(0);
        let _ = db.knowledge_topic_tree(PROJECT).await.unwrap();
        let _ = db
            .topic_memories("km-child", MemoryLayer::L2, true, 50)
            .await
            .unwrap();
        let after: i64 = pool
            .get()
            .await
            .unwrap()
            .query_one("SELECT count(*) FROM canonical_events", &[])
            .await
            .unwrap()
            .get(0);
        assert_eq!(before, after, "a panel query must not move the ledger");
    }
}
