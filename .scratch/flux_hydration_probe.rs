// Flux verification probe for the Mira knowledge-panel slice (2026-10-16), reused
// for the Nimbus harness-pi slice (2026-10-17).
//
// Exercises the EXACT core reads the SeatLoom app's hydration calls, against the
// live database, READ-ONLY. hydrateFromBackend runs eight reads in Promise.all;
// the first rejection abandons the rest and lands in backendHydrateError. This
// probe runs each one independently so it can say WHICH reads answer and which
// one fails — the thing the dispatch asked to be dug out and the thing nobody
// could see from the webview console.
//
//   cmd_list_projects      -> SeatloomDb::list_projects
//   cmd_list_seats         -> SeatloomDb::list_seats
//   cmd_list_workitems     -> SeatloomDb::list_workitems
//   cmd_list_handoffs      -> SeatloomDb::list_handoffs
//   cmd_list_artifacts     -> SeatloomDb::list_artifacts
//   cmd_list_sessions      -> SeatloomDb::list_sessions
//   cmd_list_events(400)   -> SeatloomDb::list_events
//   cmd_list_documents     -> SeatloomDb::list_documents
//   cmd_list_role_bindings -> SeatloomDb::list_role_bindings_for_project
//
// No writes of any kind.
//
// Usage: copy to crates/seatloom-core/examples/, then
//   DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/seatloom" \
//     cargo run -p seatloom-core --example flux_hydration_probe
//
// Results:
//   2026-10-16 (before the Pi fix): list_seats FAILS — seats.default_harness='Pi'
//     out of the Harness domain on seat-ql-001 / seat-vela-001; every other read OK.
//   2026-10-17 (after the Pi fix, packet/nimbus/harness-pi @ ace1b2a): ALL eight
//     reads OK, list_seats 8 rows; ql_delivery's binding read still fails on
//     role='worker' (a data defect, caught per-project by the hydration).
use seatloom_core::db::connection::create_pool;
use seatloom_core::db::repositories::SeatloomDb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = SeatloomDb::new(create_pool()?);

    macro_rules! probe {
        ($label:expr, $call:expr) => {
            match $call.await {
                Ok(rows) => println!("OK   {:22} rows={}", $label, rows.len()),
                Err(e) => println!("FAIL {:22} {}", $label, e),
            }
        };
    }

    probe!("list_projects", db.list_projects());
    probe!("list_seats", db.list_seats());
    probe!("list_workitems", db.list_workitems());
    probe!("list_handoffs", db.list_handoffs());
    probe!("list_artifacts", db.list_artifacts(None, None));
    probe!("list_sessions", db.list_sessions());
    probe!("list_events(400)", db.list_events(400));
    probe!("list_documents", db.list_documents("seatloom", None, None));

    for project_id in ["seatloom", "ql_delivery", "ThoughtOnly"] {
        match db.list_role_bindings_for_project(project_id).await {
            Ok(rows) => {
                let mut ids: Vec<String> = rows.iter().map(|b| b.seat_id.clone()).collect();
                ids.sort();
                println!(
                    "OK   bindings[{:12}] count={} ql_bound={} seats={:?}",
                    project_id,
                    ids.len(),
                    ids.iter().any(|s| s == "seat-ql-001"),
                    ids
                );
            }
            Err(e) => println!("FAIL bindings[{:12}] {}", project_id, e),
        }
    }
    Ok(())
}
