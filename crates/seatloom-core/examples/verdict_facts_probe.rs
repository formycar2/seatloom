//! Print how a Flux verdict reference reads for a given dispatch — the two
//! questions the acceptance gate now asks (is it green, is it about THIS
//! dispatch) that `event_or_artifact_exists` was being asked instead.
//!
//! Driven by `proof/acceptance_verdict_gate_proof.sh` so the criterion is
//! measured against the LIVE database rather than asserted in prose.
//!
//!   DATABASE_URL=... VERDICT_REF=ev-... DISPATCH_ID=disp-... \
//!     cargo run -p seatloom-core --example verdict_facts_probe

#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let ref_id = std::env::var("VERDICT_REF").expect("VERDICT_REF");
    let dispatch_id = std::env::var("DISPATCH_ID").expect("DISPATCH_ID");
    let mut cfg = deadpool_postgres::Config::new();
    cfg.url = Some(url);
    let pool = cfg
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .unwrap();
    let db = seatloom_core::db::SeatloomDb::new(pool);
    let f = db
        .flux_verdict_facts(&ref_id, &dispatch_id)
        .await
        .expect("the read");
    println!(
        "exists={} event_type={:?} verdict={:?} names_dispatch={} green={}",
        f.exists,
        f.event_type,
        f.verdict,
        f.names_dispatch,
        f.is_merge_criterion()
    );
}
