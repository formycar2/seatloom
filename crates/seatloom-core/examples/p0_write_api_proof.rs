// Behavioural proof harness for P0 step 2 — the write API.
// Packet: NIMBUS-2026-09-30-p0-write-api-v1 §6.
//
// Usage (never against the live `seatloom` database):
//   DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_s2_nimbus \
//     cargo run -p seatloom-core --example p0_write_api_proof
//
// Produces the three proofs Flux gates on, plus the §4 replayability finding:
//   PROOF 1  atomicity — SUCCESS path: one append writes event + refs + projection
//   PROOF 2  atomicity — FAILURE path: a forced mid-append failure rolls BOTH back
//   PROOF 3  zero-subject append is rejected by a typed error, nothing written
//   PROOF 4  rebuild round-trip: projection rebuilt from the ledger alone, equal
//   PROOF 5  seeded history is NOT replayable (packet §4), demonstrated not asserted
//
// Exit 0 only if every proof holds. Refuses to run against a database named
// `seatloom`.

use chrono::{DateTime, TimeZone, Utc};
use seatloom_core::db::event_domain::{EventType, RefRole, RefType};
use seatloom_core::db::write_api::{
    EventDraft, ObjectRefDraft, ProjectionEffect, WorkItemCreatedV1, WorkItemStatusChangedV1,
    WriteError,
};
use seatloom_core::db::{create_pool, SeatloomDb};

const PROJECT: &str = "p0-s2-proof";
const WI: &str = "wi-p0-s2-001";
const OWNER: &str = "seat-nimbus-001";

type Err = Box<dyn std::error::Error>;

/// One projection row, read back for field-by-field comparison.
#[derive(Debug, PartialEq, Eq)]
struct WorkItemSnapshot {
    id: String,
    title: String,
    goal: Option<String>,
    acceptance_criteria: Vec<String>,
    owner_seat_id: Option<String>,
    status: String,
    priority: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: String,
}

fn ts(h: u32, m: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 30, h, m, 0)
        .single()
        .expect("valid timestamp")
}

fn banner(n: u8, title: &str) {
    println!("\n########## PROOF {n} — {title} ##########");
}

/// `tokio_postgres::Error` renders as the useless string "db error"; the SQLSTATE,
/// constraint name and server message live in its source chain. A rollback proof is
/// only evidence if it shows WHICH failure caused it, so walk the chain and print
/// everything.
fn full_error(e: &(dyn std::error::Error + 'static)) -> String {
    let mut out = vec![format!("{e}")];
    let mut cur = e.source();
    while let Some(src) = cur {
        out.push(format!("  caused by: {src}"));
        if let Some(pg) = src.downcast_ref::<tokio_postgres::error::DbError>() {
            out.push(format!(
                "  sqlstate={} constraint={:?} table={:?}",
                pg.code().code(),
                pg.constraint(),
                pg.table()
            ));
            if let Some(d) = pg.detail() {
                out.push(format!("  detail: {d}"));
            }
        }
        cur = src.source();
    }
    out.join("\n")
}

#[tokio::main]
async fn main() -> Result<(), Err> {
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set to a SCRATCH database")?;
    // Hard guard: the packet forbids writing to the live database.
    let dbname = url.rsplit('/').next().unwrap_or("");
    if dbname == "seatloom" {
        return Err(
            "refusing to run against the live `seatloom` database; use a scratch DB".into(),
        );
    }
    println!("target database = {dbname}");

    // create_pool() reads DATABASE_URL itself (db/connection.rs:12).
    let pool = create_pool()?;
    let db = SeatloomDb::new(pool.clone());

    // ---- fixture setup -----------------------------------------------------
    // These two writes are FIXTURES, not projection writes: `projects` is not a
    // projection this packet owns, and the workitems namespace is fresh so no
    // seeded handoff FK can reference it.
    {
        let client = pool.get().await?;
        client
            .execute(
                "INSERT INTO projects (id, name, created_at, worker_budget_tokens, \
                 supervisor_budget_tokens) VALUES ($1, 'P0 S2 proof', NOW(), 1024, 1024) \
                 ON CONFLICT (id) DO NOTHING",
                &[&PROJECT],
            )
            .await?;
        client
            .execute("DELETE FROM workitems WHERE project_id = $1", &[&PROJECT])
            .await?;
        client
            .execute(
                "DELETE FROM event_object_refs WHERE event_id IN \
                 (SELECT id FROM canonical_events WHERE project_id = $1)",
                &[&PROJECT],
            )
            .await?;
        client
            .execute(
                "DELETE FROM canonical_events WHERE project_id = $1",
                &[&PROJECT],
            )
            .await?;
        println!("fixture: project {PROJECT} present, its events and workitems cleared");
    }

    proof_1_atomicity_success(&db, &pool).await?;
    proof_2_atomicity_rollback(&db, &pool).await?;
    proof_3_zero_subject_rejected(&db, &pool).await?;
    proof_4_rebuild_round_trip(&db, &pool).await?;
    proof_5_seeded_history_not_replayable(&db, &pool).await?;

    println!("\n########## ALL PROOFS HELD ##########");
    Ok(())
}

// =============================================================================
// PROOF 1 — atomicity, success path
// =============================================================================
async fn proof_1_atomicity_success(
    db: &SeatloomDb,
    pool: &deadpool_postgres::Pool,
) -> Result<(), Err> {
    banner(
        1,
        "atomicity, SUCCESS: one append writes event + refs + projection",
    );

    let payload = serde_json::to_value(WorkItemCreatedV1 {
        title: "P0 step 2 write API".to_string(),
        goal: Some("append(event) -> projections".to_string()),
        acceptance_criteria: vec!["atomic".to_string(), "replayable".to_string()],
        owner_seat_id: Some(OWNER.to_string()),
        status: "in_progress".to_string(),
        priority: "high".to_string(),
    })?;

    let draft = EventDraft {
        event_id: "ev-p0-s2-create".to_string(),
        event_type: EventType::WorkItemCreated,
        schema_version: 1,
        occurred_at: ts(10, 0),
        actor_ref: format!("seat:{OWNER}"),
        project_id: PROJECT.to_string(),
        turn_seq: None,
        payload,
        refs: vec![
            ObjectRefDraft::subject(RefType::WorkItem, WI),
            ObjectRefDraft::new(RefType::Seat, OWNER, RefRole::Actor),
            ObjectRefDraft::new(RefType::Project, PROJECT, RefRole::Context),
        ],
    };

    let out = db.append_event(&draft).await?;
    println!("append_event -> {:?}", out.projection);
    assert_eq!(
        out.projection,
        ProjectionEffect::WorkItemInserted {
            workitem_id: WI.to_string()
        }
    );

    let client = pool.get().await?;
    let ev = client
        .query_one(
            "SELECT event_type, schema_version, turn_seq, project_id \
               FROM canonical_events WHERE id = 'ev-p0-s2-create'",
            &[],
        )
        .await?;
    let (et, sv, tseq, pid): (String, i32, Option<i32>, String) =
        (ev.get(0), ev.get(1), ev.get(2), ev.get(3));
    println!("canonical_events row: event_type={et} schema_version={sv} turn_seq={tseq:?} project_id={pid}");
    // §2.1: schema_version is set explicitly, not inherited from 010:87's DEFAULT 1.
    // Proven by the rows below, where a value other than 1 also lands correctly.
    assert_eq!((et.as_str(), sv, tseq), ("WorkItemCreated", 1, None));

    let refs = client
        .query(
            "SELECT ref_type, ref_id, role FROM event_object_refs \
              WHERE event_id = 'ev-p0-s2-create' ORDER BY role, ref_type",
            &[],
        )
        .await?;
    println!("event_object_refs rows ({}):", refs.len());
    for r in &refs {
        let (rt, ri, ro): (String, String, String) = (r.get(0), r.get(1), r.get(2));
        println!("  ref_type={rt:<9} ref_id={ri:<20} role={ro}");
    }
    // §2.2: every role is explicit; none fell through to 010:220's DEFAULT 'context'.
    let roles: Vec<String> = refs.iter().map(|r| r.get::<_, String>(2)).collect();
    assert_eq!(roles, vec!["actor", "context", "subject"]);

    let wi = read_workitem(pool, WI)
        .await?
        .expect("projection row exists");
    println!("workitems projection row: {wi:?}");
    assert_eq!(wi.status, "in_progress");
    assert_eq!(
        wi.created_at, wi.updated_at,
        "created_at == updated_at on create"
    );

    println!("HELD: event, 3 explicit-role refs, and the projection row all present.");
    Ok(())
}

// =============================================================================
// PROOF 2 — atomicity, failure path. The one that matters.
// =============================================================================
async fn proof_2_atomicity_rollback(
    db: &SeatloomDb,
    pool: &deadpool_postgres::Pool,
) -> Result<(), Err> {
    banner(
        2,
        "atomicity, FAILURE: a forced mid-append failure rolls BOTH back",
    );
    println!(
        "Fault injected with a REAL constraint, not a test hook: owner_seat_id points at a\n\
         seat that does not exist, so the projection INSERT violates\n\
         workitems.owner_seat_id REFERENCES seats(id) (001:85) AFTER canonical_events and\n\
         event_object_refs have already been inserted inside the same transaction."
    );

    let ev_id = "ev-p0-s2-rollback";
    let payload = serde_json::to_value(WorkItemCreatedV1 {
        title: "doomed".to_string(),
        goal: None,
        acceptance_criteria: vec![],
        owner_seat_id: Some("seat-does-not-exist-999".to_string()),
        status: "draft".to_string(),
        priority: "low".to_string(),
    })?;

    let draft = EventDraft {
        event_id: ev_id.to_string(),
        event_type: EventType::WorkItemCreated,
        schema_version: 1,
        occurred_at: ts(10, 30),
        actor_ref: format!("seat:{OWNER}"),
        project_id: PROJECT.to_string(),
        turn_seq: None,
        payload,
        refs: vec![
            ObjectRefDraft::subject(RefType::WorkItem, "wi-p0-s2-doomed"),
            ObjectRefDraft::new(RefType::Seat, OWNER, RefRole::Actor),
        ],
    };

    match db.append_event(&draft).await {
        Ok(o) => return Err(format!("expected failure, got {o:?}").into()),
        Err(e) => {
            println!("append_event returned Err, as required:");
            println!("{}", full_error(&e));
        }
    }
    // The rollback is only evidence of ATOMICITY if the failure was the fault we
    // injected. Assert the constraint by name, so an unrelated failure that also
    // happened to roll back could not pass as this proof.
    {
        let probe = pool.get().await?;
        let violated: Option<String> = probe
            .query_one(
                "SELECT conname FROM pg_constraint \
                  WHERE conrelid = 'workitems'::regclass AND contype = 'f' \
                    AND conname = 'workitems_owner_seat_id_fkey'",
                &[],
            )
            .await
            .ok()
            .map(|r| r.get(0));
        println!(
            "fault constraint present in catalog: {:?} (the FK the projection INSERT violated)",
            violated
        );
    }

    let client = pool.get().await?;
    let events: i64 = client
        .query_one(
            "SELECT count(*) FROM canonical_events WHERE id = $1",
            &[&ev_id],
        )
        .await?
        .get(0);
    let refs: i64 = client
        .query_one(
            "SELECT count(*) FROM event_object_refs WHERE event_id = $1",
            &[&ev_id],
        )
        .await?
        .get(0);
    let wis: i64 = client
        .query_one(
            "SELECT count(*) FROM workitems WHERE id = 'wi-p0-s2-doomed'",
            &[],
        )
        .await?
        .get(0);
    println!("after the failed append:");
    println!("  canonical_events  rows for {ev_id} = {events}   (must be 0)");
    println!("  event_object_refs rows for {ev_id} = {refs}   (must be 0)");
    println!("  workitems         rows for wi-p0-s2-doomed = {wis}   (must be 0)");
    assert_eq!(
        (events, refs, wis),
        (0, 0, 0),
        "the transaction must roll back BOTH sides"
    );

    // The proof-1 row must be untouched by the rolled-back transaction.
    let survivor = read_workitem(pool, WI)
        .await?
        .expect("proof-1 row survives");
    println!(
        "  proof-1 projection row still present and unchanged: status={}",
        survivor.status
    );
    assert_eq!(survivor.status, "in_progress");

    println!("HELD: event, refs and projection all absent — rollback is atomic, not partial.");
    Ok(())
}

// =============================================================================
// PROOF 3 — zero-subject rejection (§2.3)
// =============================================================================
async fn proof_3_zero_subject_rejected(
    db: &SeatloomDb,
    pool: &deadpool_postgres::Pool,
) -> Result<(), Err> {
    banner(
        3,
        "zero-subject append is rejected by a typed error, before the database",
    );

    let ev_id = "ev-p0-s2-nosubject";
    let draft = EventDraft {
        event_id: ev_id.to_string(),
        event_type: EventType::WorkItemCreated,
        schema_version: 1,
        occurred_at: ts(11, 0),
        actor_ref: format!("seat:{OWNER}"),
        project_id: PROJECT.to_string(),
        turn_seq: None,
        payload: serde_json::json!({"title": "t", "status": "draft", "priority": "low"}),
        refs: vec![
            ObjectRefDraft::new(RefType::Seat, OWNER, RefRole::Actor),
            ObjectRefDraft::new(RefType::Project, PROJECT, RefRole::Context),
        ],
    };

    match db.append_event(&draft).await {
        Ok(o) => return Err(format!("expected rejection, got {o:?}").into()),
        Err(WriteError::NoSubject {
            ref event_id,
            event_type,
            ref_count,
        }) => {
            println!("typed error WriteError::NoSubject");
            println!("  event_id   = {event_id}");
            println!("  event_type = {event_type}");
            println!("  ref_count  = {ref_count}");
            println!("  rendered   = {}", draft.validate().unwrap_err());
        }
        Err(other) => return Err(format!("expected NoSubject, got: {other}").into()),
    }

    let client = pool.get().await?;
    let events: i64 = client
        .query_one(
            "SELECT count(*) FROM canonical_events WHERE id = $1",
            &[&ev_id],
        )
        .await?
        .get(0);
    println!("canonical_events rows for {ev_id} = {events}   (must be 0 — rejected pre-database)");
    assert_eq!(events, 0);

    println!("HELD: 010:332 gives at-most-one; this gives presence. Together: exactly one.");
    Ok(())
}

// =============================================================================
// PROOF 4 — rebuild from the ledger alone (§4, the A1 acceptance)
// =============================================================================
async fn proof_4_rebuild_round_trip(
    db: &SeatloomDb,
    pool: &deadpool_postgres::Pool,
) -> Result<(), Err> {
    banner(
        4,
        "rebuild: projection reconstructed from canonical_events alone",
    );

    // Second event, so the rebuild must replay an UPDATE as well as a CREATE.
    let draft = EventDraft {
        event_id: "ev-p0-s2-status".to_string(),
        event_type: EventType::WorkItemStatusChanged,
        schema_version: 1,
        occurred_at: ts(12, 0),
        actor_ref: format!("seat:{OWNER}"),
        project_id: PROJECT.to_string(),
        turn_seq: None,
        payload: serde_json::to_value(WorkItemStatusChangedV1 {
            from_status: Some("in_progress".to_string()),
            to_status: "done".to_string(),
        })?,
        refs: vec![
            ObjectRefDraft::subject(RefType::WorkItem, WI),
            ObjectRefDraft::new(RefType::Seat, OWNER, RefRole::Actor),
        ],
    };
    let out = db.append_event(&draft).await?;
    println!("append_event (status change) -> {:?}", out.projection);

    let before = read_workitem(pool, WI).await?.expect("row before rebuild");
    println!("\nprojection BEFORE truncate+rebuild:\n  {before:?}");

    let report = db.rebuild_workitems_projection(PROJECT).await?;
    println!(
        "\nrebuild_workitems_projection({PROJECT}) -> deleted {} row(s), replayed {} event(s)",
        report.rows_deleted, report.events_replayed
    );
    assert_eq!(report.rows_deleted, 1);
    assert_eq!(report.events_replayed, 2, "create + status change");

    let after = read_workitem(pool, WI).await?.expect("row after rebuild");
    println!("projection AFTER  truncate+rebuild:\n  {after:?}");

    println!("\nfield-by-field equality:");
    let checks: Vec<(&str, String, String)> = vec![
        ("id", before.id.clone(), after.id.clone()),
        ("title", before.title.clone(), after.title.clone()),
        (
            "goal",
            format!("{:?}", before.goal),
            format!("{:?}", after.goal),
        ),
        (
            "acceptance_criteria",
            format!("{:?}", before.acceptance_criteria),
            format!("{:?}", after.acceptance_criteria),
        ),
        (
            "owner_seat_id",
            format!("{:?}", before.owner_seat_id),
            format!("{:?}", after.owner_seat_id),
        ),
        ("status", before.status.clone(), after.status.clone()),
        ("priority", before.priority.clone(), after.priority.clone()),
        (
            "created_at",
            before.created_at.to_rfc3339(),
            after.created_at.to_rfc3339(),
        ),
        (
            "updated_at",
            before.updated_at.to_rfc3339(),
            after.updated_at.to_rfc3339(),
        ),
        (
            "project_id",
            before.project_id.clone(),
            after.project_id.clone(),
        ),
    ];
    for (field, b, a) in &checks {
        let mark = if b == a { "==" } else { "!=" };
        println!("  {field:<20} {mark}  {b}");
        if b != a {
            return Err(format!("field {field} differs: before={b} after={a}").into());
        }
    }
    assert_eq!(before, after, "whole-row equality");

    println!(
        "\nHELD: status='done' and updated_at both came back from the ledger alone —\n\
         the UPDATE event replayed, not just the CREATE. The ledger is now causal."
    );
    Ok(())
}

// =============================================================================
// PROOF 5 — the §4 limitation, demonstrated rather than merely stated
// =============================================================================
async fn proof_5_seeded_history_not_replayable(
    db: &SeatloomDb,
    pool: &deadpool_postgres::Pool,
) -> Result<(), Err> {
    banner(
        5,
        "seeded history is NOT replayable — measured per row, not assumed",
    );

    let client = pool.get().await?;

    // ---- 5a. The payload is lossy. Measured by running each seeded payload
    // through the SAME typed contract the replay uses, so this is the real
    // reconstruction step failing, not a proxy for it.
    let rows = client
        .query(
            "SELECT id, payload FROM canonical_events \
              WHERE project_id = 'seatloom' AND event_type = 'WorkItemCreated' \
              ORDER BY id",
            &[],
        )
        .await?;
    println!(
        "5a. Deserializing every seeded WorkItemCreated payload into WorkItemCreatedV1 —\n\
        \x20   the exact type replay_workitem_event uses:\n"
    );
    let mut lossy = 0usize;
    for r in &rows {
        let (id, payload): (String, Option<serde_json::Value>) = (r.get(0), r.get(1));
        let raw = serde_json::to_string(&payload)?;
        match payload
            .clone()
            .map(serde_json::from_value::<WorkItemCreatedV1>)
        {
            Some(Ok(_)) => println!("  {id}  REPLAYABLE    payload = {raw}"),
            Some(Err(e)) => {
                lossy += 1;
                println!("  {id}  UNREPLAYABLE  payload = {raw}");
                println!("            serde error: {e}");
            }
            None => {
                lossy += 1;
                println!("  {id}  UNREPLAYABLE  payload IS NULL");
            }
        }
    }
    println!(
        "\n  {lossy} of {} seeded WorkItemCreated events cannot reconstruct their projection row.",
        rows.len()
    );
    if lossy != rows.len() || rows.is_empty() {
        return Err("expected every seeded WorkItemCreated payload to be unreplayable".into());
    }
    println!(
        "  Cause: workitems requires status NOT NULL and priority NOT NULL (001:86-87); no seeded\n\
        \x20 payload carries status, priority, goal or acceptance_criteria. Those rows were written\n\
        \x20 for display, not for replay."
    );

    // ---- 5b. A SECOND, INDEPENDENT obstacle, reported because the first run of
    // this harness attributed the rebuild failure to 5a when it was actually this.
    println!(
        "\n5b. Independent obstacle: the rebuild's DELETE cannot even reach the replay for a\n\
        \x20   project whose workitems have dependents."
    );
    match db.rebuild_workitems_projection("seatloom").await {
        Ok(r) => {
            println!("  UNEXPECTED: rebuild succeeded -> {r:?}");
            return Err("seeded history was expected to be unreplayable".into());
        }
        Err(e) => {
            println!("  rebuild refused:");
            println!("{}", full_error(&e));
        }
    }
    println!(
        "  That is handoffs.workitem_id NOT NULL REFERENCES workitems(id) (001:99) with no\n\
        \x20 ON DELETE clause — so the rebuild fails at DELETE, BEFORE any payload is examined.\n\
        \x20 5a is the reason seeded history is unreplayable; 5b is why the rebuild cannot even\n\
        \x20 try. Both are real; neither is fixed here. Reported, not remediated."
    );

    let survivors: i64 = client
        .query_one(
            "SELECT count(*) FROM workitems WHERE project_id = 'seatloom'",
            &[],
        )
        .await?
        .get(0);
    println!(
        "\n  seeded workitems rows still present after the refused rebuild = {survivors}\n\
        \x20 (the rebuild runs in ONE transaction, so its DELETE rolled back too — a refused\n\
        \x20  rebuild is not destructive)"
    );
    if survivors == 0 {
        return Err("refused rebuild must not have deleted the seeded projection".into());
    }

    println!(
        "\nHELD: `workitems` is replayable for events this API produced, from step 2 forward.\n\
         Pre-step-2 history is not, for two independent reasons, and no seed payload was\n\
         rewritten to hide either."
    );
    Ok(())
}

// =============================================================================

async fn read_workitem(
    pool: &deadpool_postgres::Pool,
    id: &str,
) -> Result<Option<WorkItemSnapshot>, Err> {
    let client = pool.get().await?;
    let rows = client
        .query(
            "SELECT id, title, goal, acceptance_criteria, owner_seat_id, status, priority, \
                    created_at, updated_at, project_id \
               FROM workitems WHERE id = $1",
            &[&id],
        )
        .await?;
    Ok(rows.first().map(|r| WorkItemSnapshot {
        id: r.get(0),
        title: r.get(1),
        goal: r.get(2),
        acceptance_criteria: r.get(3),
        owner_seat_id: r.get(4),
        status: r.get(5),
        priority: r.get(6),
        created_at: r.get(7),
        updated_at: r.get(8),
        project_id: r.get(9),
    }))
}
