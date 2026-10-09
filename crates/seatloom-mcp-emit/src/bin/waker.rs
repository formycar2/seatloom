//! seatloom-waker — event-driven, zero-poll dispatch waker.
//!
//! Listens on the Postgres `dispatch_ready` channel (NOTIFY fires from
//! `enqueue_dispatch` on commit, only for READY dispatches). When a seat has a
//! new ready dispatch AND is idle (no in_flight dispatch), it nudges that seat's
//! tmux pane so the seat runs its `get_next_dispatch` drain loop.
//!
//! Design (why this shape — see the dispatch-pickup discussion):
//!   - NON-LLM: waiting costs zero tokens; the seat's LLM only runs on real work.
//!   - Only pokes an IDLE seat (no in_flight row). A BUSY seat is never
//!     interrupted — it absorbs newly-arrived work itself at its next task
//!     boundary (its get_next loop drains the queue). This preserves execution
//!     continuity (the whole point).
//!   - in_flight is the busy signal (one in_flight per seat is a DB fact), so the
//!     check is race-correct and free; no lost wakeups (a dispatch committed
//!     after in_flight clears triggers a NOTIFY that finds the seat not-busy and
//!     pokes; a nudge landing during turn-teardown is buffered and processed).
//!
//! Env: DATABASE_URL (default postgresql://seatloom:seatloom@localhost:5432/seatloom),
//!      SEATLOOM_SEAT_MAP (default .seatloom/dispatch-seats.tsv).

use std::collections::HashMap;
use std::future::poll_fn;
use std::process::Command;

use tokio_postgres::{AsyncMessage, NoTls};

fn load_seat_map(path: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(path) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((seat, sess)) = line.split_once('\t') {
                m.insert(seat.trim().to_string(), sess.trim().to_string());
            }
        }
    }
    m
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://seatloom:seatloom@localhost:5432/seatloom".to_string());
    let map_path = std::env::var("SEATLOOM_SEAT_MAP")
        .unwrap_or_else(|_| ".seatloom/dispatch-seats.tsv".to_string());
    let seat_map = load_seat_map(&map_path);
    eprintln!(
        "seatloom-waker: {} seat(s) mapped from {}; LISTEN dispatch_ready on {}",
        seat_map.len(),
        map_path,
        db_url.split('@').next_back().unwrap_or("db")
    );

    let (client, mut connection) = tokio_postgres::connect(&db_url, NoTls).await?;

    // Drive the connection and forward async notifications to a channel.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<tokio_postgres::Notification>();
    tokio::spawn(async move {
        loop {
            match poll_fn(|cx| connection.poll_message(cx)).await {
                Some(Ok(AsyncMessage::Notification(n))) => {
                    if tx.send(n).is_err() {
                        break;
                    }
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => {
                    eprintln!("seatloom-waker: connection error: {e}");
                    break;
                }
                None => break,
            }
        }
    });

    client.batch_execute("LISTEN dispatch_ready").await?;
    eprintln!("seatloom-waker: armed.");

    while let Some(n) = rx.recv().await {
        let seat_id = n.payload().to_string();
        let Some(session) = seat_map.get(&seat_id) else {
            eprintln!("seatloom-waker: NOTIFY for unmapped seat {seat_id} — ignoring");
            continue;
        };

        // Busy check: a seat with an in_flight dispatch is executing — never
        // interrupt it; it will drain new work itself at its next boundary.
        let busy = client
            .query_opt(
                "SELECT 1 FROM dispatches WHERE target_seat_id = $1 AND status = 'in_flight' LIMIT 1",
                &[&seat_id],
            )
            .await
            .unwrap_or(None)
            .is_some();
        if busy {
            eprintln!("seatloom-waker: {seat_id} busy (in_flight) — not poking; it self-drains");
            continue;
        }

        eprintln!("seatloom-waker: waking idle seat {seat_id} -> {session}");
        nudge(session);
    }

    eprintln!("seatloom-waker: notification stream ended, exiting");
    Ok(())
}

/// Nudge an idle seat's tmux pane to run its dispatch drain loop.
fn nudge(session: &str) {
    let msg = "📥 new governed dispatch available — run your dispatch loop now: \
               call get_next_dispatch repeatedly (do each payload, then complete_dispatch) \
               until it returns \"nothing ready\", then stop.";
    // Type the message, then submit with Enter (sent separately so the pane's
    // line editor registers a real submit).
    let _ = Command::new("tmux")
        .args(["send-keys", "-t", session, msg])
        .status();
    let _ = Command::new("tmux")
        .args(["send-keys", "-t", session, "Enter"])
        .status();
}
