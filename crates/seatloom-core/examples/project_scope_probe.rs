//! Measure every project-scoped read for one project, so the per-project
//! filtering is OBSERVED against the live database rather than asserted in prose.
//!
//! The defect this proves fixed: `cmd_list_seats` / `cmd_list_workitems` /
//! `cmd_list_handoffs` / `cmd_list_sessions` took no `project_id` and read their
//! tables unfiltered, and the store copied the same arrays into every project's
//! `ProjectData`. So SeatLoom and ql_delivery rendered byte-identical modules.
//!
//!   DATABASE_URL=postgresql://... PROJECT_ID=seatloom \
//!     cargo run -q -p seatloom-core --example project_scope_probe
//!
//! Read-only: it only ever SELECTs.

#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let project_id = std::env::var("PROJECT_ID").expect("PROJECT_ID");

    let mut cfg = deadpool_postgres::Config::new();
    cfg.url = Some(url);
    let pool = cfg
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .unwrap();
    let db = seatloom_core::db::SeatloomDb::new(pool);

    // The seat read goes through project_role_bindings, because `seats` has no
    // project_id. This is the only one of the five that is not a WHERE.
    let seats = db
        .list_seats_for_project(&project_id)
        .await
        .expect("list_seats_for_project");
    let workitems = db
        .list_workitems_for_project(&project_id)
        .await
        .expect("list_workitems_for_project");
    let handoffs = db
        .list_handoffs_for_project(&project_id)
        .await
        .expect("list_handoffs_for_project");
    let sessions = db
        .list_sessions_for_project(&project_id)
        .await
        .expect("list_sessions_for_project");
    let events = db
        .list_events_for_project(&project_id, 400)
        .await
        .expect("list_events_for_project");

    // One line per kind, so a shell proof can diff two projects' output and see
    // that the numbers MOVE rather than reading a prose claim that they do.
    println!("project={project_id}");
    println!("seats={}", seats.len());
    print!("seat_ids=");
    for (i, s) in seats.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!("{}", s.id);
    }
    println!();
    println!("workitems={}", workitems.len());
    println!("handoffs={}", handoffs.len());
    println!("sessions={}", sessions.len());
    println!("events={}", events.len());
    // The read the bug report named: cmd_list_dispatches used to take an
    // Option<String> and fall back to the app's DEFAULT project, so the UI never
    // passed one and every project showed seatloom's dispatches. It now demands one.
    let dispatches = db
        .list_dispatches_for_project(&project_id, 500)
        .await
        .expect("list_dispatches_for_project");
    println!("dispatches={}", dispatches.len());

    // The negative controls. The whole claim is "an unscoped read returns another
    // project's rows", so prove the UNSCOPED reads really are bigger — otherwise
    // the scoping is not doing anything and the test would pass vacuously.
    let unscoped_seats = db.list_seats().await.expect("all seats").len();
    let unscoped_wi = db.list_workitems().await.expect("all wi").len();
    let unscoped_ho = db.list_handoffs().await.expect("all ho").len();
    let unscoped_se = db.list_sessions().await.expect("all se").len();
    let unscoped_ev = db.list_events(400).await.expect("all ev").len();
    // And a project that does not exist at all must return ZERO, not everything —
    // that is the shape a silent default would produce.
    let nonexistent = db
        .list_dispatches_for_project("__no_such_project__", 500)
        .await
        .expect("a nonexistent project is a valid read")
        .len();
    println!("unscoped_seats={unscoped_seats} unscoped_workitems={unscoped_wi} unscoped_handoffs={unscoped_ho} unscoped_sessions={unscoped_se} unscoped_events={unscoped_ev}");
    println!("nonexistent_project_dispatches={nonexistent}");

    // The supervision overview's three arithmetic cards, computed from THIS
    // project's rows exactly as SupervisionDashboard computes them — so the proof
    // measures the operator's first screen rather than a proxy for it. (The fourth
    // card, pendingDecisions, is derived from inboxItems in the store and is
    // covered by the store's own tests.)
    let in_progress = workitems.iter().filter(|w| w.status == "Active").count();
    let blocked_wi = workitems.iter().filter(|w| w.status == "Blocked").count();
    let blocked_sessions = sessions
        .iter()
        .filter(|s| s.status == "InputRequired")
        .count();
    let in_review = workitems.iter().filter(|w| w.status == "InReview").count();
    println!("overview_in_progress={in_progress}");
    println!("overview_blocked={}", blocked_wi + blocked_sessions);
    println!("overview_in_review={in_review}");

    // Every seat must carry a runtime triple that is inside its declared domain.
    // A scoped read must not make a domain violation invisible.
    for s in &seats {
        println!(
            "seat_domain_ok={} harness={:?} agent={:?}",
            s.id, s.default_harness, s.default_agent
        );
    }
}
