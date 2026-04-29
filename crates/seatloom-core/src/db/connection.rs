use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

/// Default local development connection string.
pub const DEFAULT_DATABASE_URL: &str = "postgresql://seatloom:seatloom@localhost:5432/seatloom";

pub const DATABASE_URL_ENV: &str = "DATABASE_URL";

pub type DbPool = Pool;

/// Create a connection pool from DATABASE_URL env var or the default local URL.
pub fn create_pool() -> Result<DbPool, Box<dyn std::error::Error>> {
    let url = std::env::var(DATABASE_URL_ENV).unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let config = parse_connection_url(&url);
    Ok(config.create_pool(Some(Runtime::Tokio1), NoTls)?)
}

/// Parse a simple postgresql://user:password@host:port/dbname URL into a deadpool Config.
fn parse_connection_url(url: &str) -> Config {
    let rest = url
        .strip_prefix("postgresql://")
        .or_else(|| url.strip_prefix("postgres://"))
        .unwrap_or(url);

    let mut cfg = Config::new();

    if let Some((credentials, host_db)) = rest.split_once('@') {
        if let Some((user, pass)) = credentials.split_once(':') {
            cfg.user = Some(user.to_string());
            cfg.password = Some(pass.to_string());
        } else {
            cfg.user = Some(credentials.to_string());
        }

        if let Some((host_port, dbname)) = host_db.split_once('/') {
            cfg.dbname = Some(dbname.to_string());
            if let Some((host, port)) = host_port.split_once(':') {
                cfg.host = Some(host.to_string());
                if let Ok(p) = port.parse::<u16>() {
                    cfg.port = Some(p);
                }
            } else {
                cfg.host = Some(host_port.to_string());
            }
        } else {
            cfg.host = Some(host_db.to_string());
        }
    }

    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg
}
