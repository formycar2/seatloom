use clap::{Parser, Subcommand};
use seatloom_core::db::connection::create_pool;
use seatloom_core::db::reconcile::{run_reconcile, ReconcileTrigger};

#[derive(Parser)]
#[command(name = "seatloom", about = "SeatLoom CLI — project continuity layer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new SeatLoom project in the given path.
    Init { path: Option<String> },
    /// Attach SeatLoom to a running agent process by PID.
    Attach { pid: u32 },
    /// List sessions for the current project.
    Sessions,
    /// List WorkItems for the current project.
    Workitems,
    /// Reconcile repository markdown documents into PostgreSQL authority.
    /// Scans docs/ and docs/coordination/ for typed markdown, ingests into DB.
    /// Trigger: manual (startup and pre-pipeline are invoked by the app runtime).
    Reconcile {
        /// Project ID to associate ingested documents with.
        #[arg(long, default_value = "seatloom")]
        project: String,
        /// Repo root path (default: current directory).
        #[arg(long)]
        root: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { path } => {
            eprintln!("init: {:?} — not yet implemented", path);
        }
        Commands::Attach { pid } => {
            eprintln!("attach: pid={pid} — not yet implemented");
        }
        Commands::Sessions => {
            eprintln!("sessions — not yet implemented");
        }
        Commands::Workitems => {
            eprintln!("workitems — not yet implemented");
        }
        Commands::Reconcile { project, root } => {
            let repo_root = root
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().expect("cwd must be accessible"));

            eprintln!(
                "seatloom reconcile: project={project} root={}",
                repo_root.display()
            );

            let pool = match create_pool() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: cannot create DB connection pool: {e}");
                    std::process::exit(1);
                }
            };

            match run_reconcile(&pool, ReconcileTrigger::Manual, &repo_root, &project).await {
                Ok(result) => {
                    println!("reconcile completed: run_id={}", result.run_id);
                    println!(
                        "  scanned={} inserted={} updated={} unchanged={} failed={} conflicted={}",
                        result.scanned,
                        result.inserted,
                        result.updated,
                        result.unchanged,
                        result.failed,
                        result.conflicted
                    );
                    if result.failed > 0 || result.conflicted > 0 {
                        eprintln!(
                            "warning: {} failed, {} conflicted — check reconcile_items table",
                            result.failed, result.conflicted
                        );
                    }
                }
                Err(e) => {
                    eprintln!("reconcile error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
