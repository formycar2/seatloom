//! Supervisor IM window-order regression tests: the window must be the NEWEST
//! `limit` events, returned in ASCENDING order (chat reading order).
#[cfg(test)]
mod supim {
    use deadpool_postgres::Pool;
    use seatloom_core::db::connection::{create_pool, DATABASE_URL_ENV};
    use seatloom_core::db::SeatloomDb;

    async fn scratch_pool() -> Option<Pool> {
        let url = std::env::var(DATABASE_URL_ENV).ok()?;
        let dbname = url.rsplit('/').next().unwrap_or("");
        if dbname == "seatloom" || dbname.starts_with("seatloom?") {
            eprintln!("REFUSING live `seatloom` DB for supervisor IM tests");
            return None;
        }
        let pool = create_pool().ok()?;
        let client = pool.get().await.ok()?;
        client.query_one("SELECT 1", &[]).await.ok()?;
        Some(pool)
    }

    /// 120 events with distinct timestamps; a limit of 400 must return ALL of
    /// them in ascending order (the window is bigger than the data).
    #[tokio::test]
    #[ignore = "requires PostgreSQL (scratch DB)"]
    async fn a_window_larger_than_the_data_returns_everything_ascending() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let c = pool.get().await.unwrap();
        c.execute(
            "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
             VALUES ('supim-itest','supim itest',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING",
            &[],
        )
        .await
        .unwrap();
        c.execute("DELETE FROM event_object_refs", &[])
            .await
            .unwrap();
        c.execute("DELETE FROM canonical_events", &[])
            .await
            .unwrap();
        for i in 0..120i32 {
            c.execute(
                "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
                 VALUES ($1,'SupervisorMessage',NOW() - ($2::int * interval '1 minute'),'human:user','{}','supim-itest',1)",
                &[&format!("ev-supim-{i:03}"), &(120 - i)],
            ).await.unwrap();
        }
        let rows = db.list_supervisor_messages(None, 400).await.unwrap();
        assert_eq!(rows.len(), 120);
        for w in rows.windows(2) {
            assert!(w[0].occurred_at <= w[1].occurred_at, "must be ascending");
        }
        // The oldest and newest are both present.
        assert_eq!(rows[0].id, "ev-supim-000");
        assert_eq!(rows[119].id, "ev-supim-119");
    }

    /// THE regression: with more events than the window, the window must hold
    /// the NEWEST ones (the old shape returned the oldest and hid the recent
    /// 115 — including the operator's own messages).
    #[tokio::test]
    #[ignore = "requires PostgreSQL (scratch DB)"]
    async fn a_window_smaller_than_the_data_returns_the_newest_ascending() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let c = pool.get().await.unwrap();
        c.execute(
            "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
             VALUES ('supim-itest','supim itest',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING",
            &[],
        )
        .await
        .unwrap();
        c.execute(
            "DELETE FROM canonical_events WHERE project_id='supim-itest'",
            &[],
        )
        .await
        .unwrap();
        for i in 0..120i32 {
            c.execute(
                "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
                 VALUES ($1,'SupervisorMessage',NOW() - ($2::int * interval '1 minute'),'human:user','{}','supim-itest',1)",
                &[&format!("ev-supim-{i:03}"), &(120 - i)],
            ).await.unwrap();
        }
        // The operator's own message, the newest of all.
        c.execute(
            "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
             VALUES ('ev-supim-operator','SupervisorMessage',NOW(),'human:user','{}','supim-itest',1)",
            &[],
        ).await.unwrap();

        let rows = db.list_supervisor_messages(None, 50).await.unwrap();
        assert_eq!(rows.len(), 50, "exactly the window size");
        // Newest last (chat order).
        assert_eq!(
            rows[49].id, "ev-supim-operator",
            "the operator's message must be visible"
        );
        // And it is the NEWEST 50: ids 71..119 plus the operator's.
        assert_eq!(
            rows[0].id, "ev-supim-071",
            "the window starts at the 50th-newest"
        );
        for w in rows.windows(2) {
            assert!(w[0].occurred_at <= w[1].occurred_at, "must be ascending");
        }
    }

    /// Equal timestamps must not be reshuffled by the reversal: the id
    /// tiebreaker keeps the window stable and the order deterministic.
    #[tokio::test]
    #[ignore = "requires PostgreSQL (scratch DB)"]
    async fn equal_timestamps_keep_a_stable_deterministic_order() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let c = pool.get().await.unwrap();
        c.execute(
            "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
             VALUES ('supim-itest','supim itest',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING",
            &[],
        )
        .await
        .unwrap();
        c.execute("DELETE FROM event_object_refs", &[])
            .await
            .unwrap();
        c.execute("DELETE FROM canonical_events", &[])
            .await
            .unwrap();
        c.execute(
            // Zero-padded: `id` is TEXT, so the tiebreak is lexical and the
            // expectation below is written for lexical order (tie-02 < tie-10).
            "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
             SELECT 'ev-supim-tie-'||lpad(g::text,2,'0'), 'SupervisorMessage', '2026-10-11 12:00:00+00', 'human:user', '{}', 'supim-itest', 1 \
               FROM generate_series(1,10) g",
            &[],
        ).await.unwrap();
        let first = db.list_supervisor_messages(None, 5).await.unwrap();
        let second = db.list_supervisor_messages(None, 5).await.unwrap();
        let ids: Vec<&str> = first.iter().map(|r| r.id.as_str()).collect();
        let ids2: Vec<&str> = second.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ids2, "the window is stable across calls");
        assert_eq!(
            ids,
            vec![
                "ev-supim-tie-06",
                "ev-supim-tie-07",
                "ev-supim-tie-08",
                "ev-supim-tie-09",
                "ev-supim-tie-10"
            ],
            "the newest five by id tiebreak, ascending"
        );
    }

    /// The seat-filtered branch behaves the same way (newest window, ascending).
    #[tokio::test]
    #[ignore = "requires PostgreSQL (scratch DB)"]
    async fn the_seat_filtered_branch_returns_the_newest_window_ascending() {
        let Some(pool) = scratch_pool().await else {
            return;
        };
        let db = SeatloomDb::new(pool.clone());
        let c = pool.get().await.unwrap();
        c.execute(
            "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
             VALUES ('supim-itest','supim itest',NOW(),1024,1024) ON CONFLICT (id) DO NOTHING",
            &[],
        )
        .await
        .unwrap();
        c.execute(
            "DELETE FROM canonical_events WHERE project_id='supim-itest'",
            &[],
        )
        .await
        .unwrap();
        for i in 0..30i32 {
            c.execute(
                "INSERT INTO canonical_events (id,event_type,occurred_at,actor_ref,payload,project_id,schema_version) \
                 VALUES ($1,'SeatResponse',NOW() - ($2::int * interval '1 minute'),'seat:seat-itest-001','{}','supim-itest',1)",
                &[&format!("ev-supim-seat-{i:03}"), &(30 - i)],
            ).await.unwrap();
        }
        let rows = db
            .list_supervisor_messages(Some("seat-itest-001"), 10)
            .await
            .unwrap();
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].id, "ev-supim-seat-020", "the newest ten, ascending");
        assert_eq!(rows[9].id, "ev-supim-seat-029");
    }
}
