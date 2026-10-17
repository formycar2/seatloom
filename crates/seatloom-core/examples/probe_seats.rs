#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").unwrap();
    let mut cfg = deadpool_postgres::Config::new();
    cfg.url = Some(url.clone());
    let pool = cfg
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .unwrap();
    let db = seatloom_core::db::SeatloomDb::new(pool);
    match db.list_seats().await {
        Ok(rows) => println!("OK: {} seats", rows.len()),
        Err(e) => println!("ERR: {e}"),
    }
    let (client, connection) = tokio_postgres::connect(&url, tokio_postgres::NoTls)
        .await
        .unwrap();
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let rows = client
        .query(
            "SELECT id, default_agent, default_harness FROM seats ORDER BY id",
            &[],
        )
        .await
        .unwrap();
    for r in &rows {
        let id: String = r.get(0);
        let agent: Option<String> = r.get(1);
        let harness: Option<String> = r.get(1 + 1);
        let ok_agent =
            seatloom_core::objects::session::Agent::try_from(agent.as_deref().unwrap_or(""))
                .is_ok();
        let ok_harness =
            seatloom_core::objects::session::Harness::try_from(harness.as_deref().unwrap_or(""))
                .is_ok();
        println!("  {id}: agent={agent:?}(in-domain={ok_agent}) harness={harness:?}(in-domain={ok_harness})");
    }
}
