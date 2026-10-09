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
//! Resident robustness — this binary is a daemon, so it must never exit on a
//! database hiccup:
//!   - Every failure (connect, LISTEN, a busy-check query, the catch-up scan)
//!     leads back to a rebuilt connection: the connect/LISTEN/consume cycle runs
//!     inside an endless loop with capped exponential backoff between attempts.
//!     A dropped LISTEN connection used to end autonomous dispatch entirely;
//!     now it is one reconnect.
//!   - NOTIFY is fire-and-forget — Postgres does not replay it for a listener
//!     that was disconnected. So after every successful (re)connect, a catch-up
//!     scan re-derives what the stream would have said: every mapped seat that
//!     holds a ready dispatch and no in_flight one is nudged. Wakeups missed
//!     during an outage are recovered, not lost.
//!
//! Env: DATABASE_URL (default postgresql://seatloom:seatloom@localhost:5432/seatloom),
//!      SEATLOOM_SEAT_MAP (default .seatloom/dispatch-seats.tsv).

use std::collections::HashMap;
use std::future::poll_fn;
use std::process::Command;
use std::time::Duration;

use tokio_postgres::{AsyncMessage, Client, NoTls, Notification};

/// First reconnect delay; doubles per failed attempt up to [`BACKOFF_CAP`].
const BACKOFF_BASE: Duration = Duration::from_millis(500);
/// Ceiling for the reconnect delay — a daemon keeps retrying, patiently.
const BACKOFF_CAP: Duration = Duration::from_secs(30);

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

/// Capped exponential backoff for the reconnect loop.
struct Backoff {
    current: Duration,
}

impl Backoff {
    fn new() -> Self {
        Self {
            current: BACKOFF_BASE,
        }
    }

    /// The delay to wait before the next attempt; each call advances the
    /// backoff (doubling, capped at [`BACKOFF_CAP`]).
    fn next_delay(&mut self) -> Duration {
        let delay = self.current;
        self.current = self.current.saturating_mul(2).min(BACKOFF_CAP);
        delay
    }

    /// A successful connection means the outage is over — start over next time.
    fn reset(&mut self) {
        self.current = BACKOFF_BASE;
    }
}

/// What the connection driver forwards to the event loop: a NOTIFY payload, or
/// the news that the connection is gone (and why).
enum ConnMsg {
    Notify(Notification),
    Lost(String),
}

/// Render a `tokio_postgres::Error` — whose `Display` is the near-useless "db
/// error" — with its full source chain, so a reconnect is diagnosable.
fn describe<E: std::error::Error>(e: E) -> String {
    let mut parts = vec![e.to_string()];
    let mut source = e.source();
    while let Some(s) = source {
        parts.push(s.to_string());
        source = s.source();
    }
    parts.join(" <- ")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
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

    // Resident daemon: every failure path leads back here. The seat map is
    // loaded once (it tracks the fleet's tmux layout, not runtime state); the
    // connection is rebuilt as often as it takes.
    let mut backoff = Backoff::new();
    loop {
        let reason = serve_once(&db_url, &seat_map, &mut backoff).await;
        let delay = backoff.next_delay();
        eprintln!("seatloom-waker: {reason} — reconnecting in {delay:?}");
        tokio::time::sleep(delay).await;
    }
}

/// One service lifetime: connect (retrying with backoff — the DB may still be
/// booting), LISTEN, run the catch-up scan, then consume notifications until
/// the connection or a query on it fails. Returns the reason this lifetime
/// ended; the caller backs off and calls again. All connection handles are
/// local, so returning drops them (closing the old connection and ending the
/// old driver task) before the next attempt.
async fn serve_once(
    db_url: &str,
    seat_map: &HashMap<String, String>,
    backoff: &mut Backoff,
) -> String {
    let (client, mut connection) = loop {
        match tokio_postgres::connect(db_url, NoTls).await {
            Ok(pair) => break pair,
            Err(e) => {
                let delay = backoff.next_delay();
                eprintln!(
                    "seatloom-waker: connect failed: {} — retrying in {delay:?}",
                    describe(e)
                );
                tokio::time::sleep(delay).await;
            }
        }
    };
    backoff.reset();

    // Drive the connection and forward async notifications — plus the
    // connection's eventual demise — to the event loop below.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ConnMsg>();
    tokio::spawn(async move {
        loop {
            match poll_fn(|cx| connection.poll_message(cx)).await {
                Some(Ok(AsyncMessage::Notification(n))) => {
                    if tx.send(ConnMsg::Notify(n)).is_err() {
                        break; // the event loop is gone; nothing to report to
                    }
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => {
                    let _ = tx.send(ConnMsg::Lost(describe(e)));
                    break;
                }
                None => {
                    let _ = tx.send(ConnMsg::Lost("connection closed".to_string()));
                    break;
                }
            }
        }
    });

    if let Err(e) = client.batch_execute("LISTEN dispatch_ready").await {
        return format!("LISTEN failed: {}", describe(e));
    }
    eprintln!("seatloom-waker: armed.");

    // Catch-up scan: NOTIFY is fire-and-forget, so anything that went ready
    // while the previous connection was down has no pending notification to
    // receive. Re-derive the missed wakeups from the DB before trusting the
    // stream again.
    if let Err(e) = catch_up(&client, seat_map).await {
        return format!("catch-up scan failed: {}", describe(e));
    }

    let mut reason = "notification stream ended".to_string();
    while let Some(msg) = rx.recv().await {
        match msg {
            ConnMsg::Notify(n) => {
                let seat_id = n.payload().to_string();
                if let Err(e) = handle_ready(&client, seat_map, &seat_id).await {
                    // The busy-check query failed — the connection is suspect.
                    // Fail safe: never nudge a seat we could not prove idle.
                    // Reconnect; the catch-up scan re-derives this seat's state.
                    reason = format!("query for {seat_id} failed: {}", describe(e));
                    break;
                }
            }
            ConnMsg::Lost(r) => {
                reason = r;
                break;
            }
        }
    }
    reason
}

/// Handle one `dispatch_ready` NOTIFY: poke the seat iff it is idle. A DB error
/// is propagated (the caller reconnects) rather than swallowed — poking after a
/// failed busy check could interrupt a busy seat.
async fn handle_ready(
    client: &Client,
    seat_map: &HashMap<String, String>,
    seat_id: &str,
) -> Result<(), tokio_postgres::Error> {
    let Some(session) = seat_map.get(seat_id) else {
        eprintln!("seatloom-waker: NOTIFY for unmapped seat {seat_id} — ignoring");
        return Ok(());
    };
    // Busy check: a seat with an in_flight dispatch is executing — never
    // interrupt it; it will drain new work itself at its next boundary.
    if is_busy(client, seat_id).await? {
        eprintln!("seatloom-waker: {seat_id} busy (in_flight) — not poking; it self-drains");
        return Ok(());
    }
    eprintln!("seatloom-waker: waking idle seat {seat_id} -> {session}");
    nudge(session);
    Ok(())
}

/// A seat with an in_flight dispatch is executing. One in_flight per seat is a
/// DB fact (the partial unique index), so the check is race-correct and free.
async fn is_busy(client: &Client, seat_id: &str) -> Result<bool, tokio_postgres::Error> {
    let row = client
        .query_opt(
            "SELECT 1 FROM dispatches WHERE target_seat_id = $1 AND status = 'in_flight' LIMIT 1",
            &[&seat_id],
        )
        .await?;
    Ok(row.is_some())
}

/// Catch-up scan after a (re)connect: nudge every mapped seat that holds a ready
/// dispatch and no in_flight one — exactly the seats whose NOTIFYs fired while
/// we were away. The condition mirrors the NOTIFY condition precisely (the
/// enqueue path only notifies for a just-committed READY row), so this closes
/// the lost-notification window and nothing more; a row stored `blocked` whose
/// deps became satisfied has no NOTIFY to miss and stays the scheduler's
/// concern.
async fn catch_up(
    client: &Client,
    seat_map: &HashMap<String, String>,
) -> Result<(), tokio_postgres::Error> {
    let rows = client
        .query(
            "SELECT DISTINCT target_seat_id FROM dispatches WHERE status = 'ready'",
            &[],
        )
        .await?;
    let mut woken = 0usize;
    for row in &rows {
        let seat_id: String = row.get(0);
        let Some(session) = seat_map.get(&seat_id) else {
            continue;
        };
        if is_busy(client, &seat_id).await? {
            continue;
        }
        eprintln!("seatloom-waker: catch-up: waking idle seat {seat_id} -> {session}");
        nudge(session);
        woken += 1;
    }
    eprintln!("seatloom-waker: catch-up scan done ({woken} seat(s) nudged)");
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
