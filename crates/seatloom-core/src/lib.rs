pub mod adapter;
/// The operator's explicitly-configured identity — one file, read at decision
/// time, never invented.
pub mod config;
pub mod data_engine;
pub mod db;
pub mod git;
pub mod history;
pub mod intent;
pub mod ledger;
pub mod objects;
pub mod pipeline;
pub mod pty;
pub mod reconcile;
pub mod storage;
/// Turn boundaries over the EXISTING tmux mirror byte stream — a heuristic, not a
/// claim about what the model did. Read the module header before trusting one.
pub mod turn_stream;
pub mod views;
