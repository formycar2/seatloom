use clap::{Parser, Subcommand};

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
    /// Run reconciliation against the current project.
    Reconcile,
}

#[tokio::main]
async fn main() {
    let _cli = Cli::parse();
    // TODO: dispatch subcommands to seatloom-core
}
