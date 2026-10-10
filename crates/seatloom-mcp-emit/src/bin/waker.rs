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
//! # The three hardening fixes (dispatch disp-q__RsEv13FNQ78nCbGd8c)
//!
//! A guaranteed deadlock was found in production: a seat whose turn ended
//! WITHOUT `complete_dispatch` (context exhaustion, an error, an interrupt, a
//! restart, a kill) leaves its dispatch `in_flight` forever. `in_flight` is a
//! CLAIM, not a liveness fact — and the waker's "busy → it self-drains" rule
//! assumed the claim implied a living, progressing turn. One stale claim stopped
//! three seats for twelve hours.
//!
//!   1. STALE-CLAIM DETECTION. An `in_flight` row older than
//!      `SEATLOOM_STALE_AFTER_SECS` (default 15 min) whose pane shows no agent
//!      activity is poked anyway. Poking is safe by construction: the seat's
//!      AGENTS.md rules that an "already in_flight" answer from
//!      `get_next_dispatch` IS its current task, so a nudge resumes it rather
//!      than duplicating it.
//!   2. PERIODIC CATCH-UP. The catch-up scan used to run once per (re)connect.
//!      A dispatch that went READY while its seat was busy has no NOTIFY to
//!      replay, so the seat sat idle with live work queued. The scan now also
//!      runs every `SEATLOOM_SWEEP_SECS` (default 90s).
//!   3. SEAT-MAP HOT RELOAD. The map was read once at startup, so a new seat
//!      needed a restart (`NOTIFY for unmapped seat ... — ignoring`). It is now
//!      re-read when the file changes (checked every sweep) and once on demand
//!      when a NOTIFY names an unmapped seat.
//!
//! What this daemon deliberately does NOT do: change a dispatch's status. A
//! stale `in_flight` is not rewritten to `failed` or `ready` — that is ledger
//! semantics (Onyx/Lyra's lane) and it would hide the real cause. The waker only
//! "goes and calls the seat"; the seat decides how to close its own task.
//!
//! Env: DATABASE_URL (default postgresql://seatloom:seatloom@localhost:5432/seatloom),
//!      SEATLOOM_SEAT_MAP (default .seatloom/dispatch-seats.tsv),
//!      SEATLOOM_STALE_AFTER_SECS (default 900), SEATLOOM_SWEEP_SECS (default 90),
//!      SEATLOOM_POKE_COOLDOWN_SECS (default 120),
//!      SEATLOOM_AGENT_PROCS (comma-separated pane-child names that count as an
//!      agent working; default "pi,node,claude,gemini,codex,opencode,cursor,
//!      stepcode,stepgo").

use std::collections::HashMap;
use std::future::poll_fn;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

use tokio_postgres::{AsyncMessage, Client, NoTls, Notification};

/// First reconnect delay; doubles per failed attempt up to [`BACKOFF_CAP`].
const BACKOFF_BASE: Duration = Duration::from_millis(500);
/// Ceiling for the reconnect delay — a daemon keeps retrying, patiently.
const BACKOFF_CAP: Duration = Duration::from_secs(30);
/// Default age after which an `in_flight` claim is considered stale.
const DEFAULT_STALE_AFTER: Duration = Duration::from_secs(15 * 60);
/// Default interval between periodic catch-up sweeps.
const DEFAULT_SWEEP_EVERY: Duration = Duration::from_secs(90);
/// Default minimum gap between two nudges to the same seat.
const DEFAULT_POKE_COOLDOWN: Duration = Duration::from_secs(120);
/// Pane-child command names that mean "an agent is working here".
const DEFAULT_AGENT_PROCS: &str = "pi,node,claude,gemini,codex,opencode,cursor,stepcode,stepgo";

// =============================================================================
// Configuration
// =============================================================================

/// Everything the daemon needs, resolved once from the environment.
struct Config {
    db_url: String,
    map_path: String,
    stale_after: Duration,
    sweep_every: Duration,
    poke_cooldown: Duration,
    agent_procs: Vec<String>,
}

impl Config {
    fn from_env() -> Self {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://seatloom:seatloom@localhost:5432/seatloom".to_string()
        });
        let map_path = std::env::var("SEATLOOM_SEAT_MAP")
            .unwrap_or_else(|_| ".seatloom/dispatch-seats.tsv".to_string());
        Self {
            db_url,
            map_path,
            stale_after: secs_from_env("SEATLOOM_STALE_AFTER_SECS", DEFAULT_STALE_AFTER),
            sweep_every: secs_from_env("SEATLOOM_SWEEP_SECS", DEFAULT_SWEEP_EVERY),
            poke_cooldown: secs_from_env("SEATLOOM_POKE_COOLDOWN_SECS", DEFAULT_POKE_COOLDOWN),
            agent_procs: std::env::var("SEATLOOM_AGENT_PROCS")
                .unwrap_or_else(|_| DEFAULT_AGENT_PROCS.to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        }
    }
}

/// A duration from an env var in seconds, falling back on absence or nonsense
/// (a daemon must never fail to start over a typo'd knob).
fn secs_from_env(key: &str, default: Duration) -> Duration {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|s| *s > 0)
        .map(Duration::from_secs)
        .unwrap_or(default)
}

// =============================================================================
// Seat map, hot-reloadable
// =============================================================================

/// The seat -> tmux-session map, with the file identity it was loaded from so a
/// change is detectable without re-reading on every sweep.
struct SeatMap {
    path: String,
    seats: HashMap<String, String>,
    /// (mtime, length) of the file the current map came from.
    stamp: Option<(SystemTime, u64)>,
}

impl SeatMap {
    fn load(path: &str) -> Self {
        let mut map = Self {
            path: path.to_string(),
            seats: HashMap::new(),
            stamp: None,
        };
        map.reload(true);
        map
    }

    /// Re-read the file if it changed. Returns true when the map was replaced.
    /// A missing or unreadable file keeps the previous map: a transient read
    /// error must not silently drop the whole fleet.
    fn reload(&mut self, force: bool) -> bool {
        let stamp = Self::stamp_of(&self.path);
        if !force && stamp == self.stamp {
            return false;
        }
        let mut m = HashMap::new();
        if let Ok(text) = std::fs::read_to_string(&self.path) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((seat, sess)) = line.split_once('\t') {
                    m.insert(seat.trim().to_string(), sess.trim().to_string());
                }
            }
        } else if !force {
            // The file vanished or became unreadable — keep what we have.
            return false;
        }
        let before = self.seats.len();
        self.seats = m;
        self.stamp = stamp;
        let after = self.seats.len();
        if before != after || force {
            eprintln!(
                "seatloom-waker: seat map loaded from {} ({after} seat(s))",
                self.path
            );
        }
        true
    }

    fn stamp_of(path: &str) -> Option<(SystemTime, u64)> {
        let meta = std::fs::metadata(path).ok()?;
        Some((meta.modified().ok()?, meta.len()))
    }

    fn get(&self, seat: &str) -> Option<&str> {
        self.seats.get(seat).map(String::as_str)
    }

    fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.seats.iter()
    }
}

// =============================================================================
// Pane liveness
// =============================================================================

/// What the seat's tmux pane is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PaneLiveness {
    /// The pane could not be interrogated (no tmux, no such session). Treated as
    /// NOT working: we cannot prove activity, and a nudge is safe by design.
    Unknown,
    /// No child processes: a bare shell, nobody is working.
    Idle,
    /// A recognised agent is running in the pane (its name is carried so the log
    /// says WHICH agent — the pane may host a different one than expected).
    Working(String),
    /// Children exist but none is a recognised agent (a build, an editor, a
    /// stuck command). Not an agent turn.
    Other(Vec<String>),
}

impl PaneLiveness {
    /// Is an agent turn in progress?
    fn is_working(&self) -> bool {
        matches!(self, PaneLiveness::Working(_))
    }

    fn describe(&self) -> String {
        match self {
            PaneLiveness::Unknown => "pane unknown".to_string(),
            PaneLiveness::Idle => "pane idle".to_string(),
            PaneLiveness::Working(name) => format!("pane running {name}"),
            PaneLiveness::Other(names) => {
                format!("pane has non-agent children: {}", names.join(","))
            }
        }
    }
}

/// The pane's child processes as `(pid, command)`.
///
/// NOT `pgrep -P`: on macOS `pgrep -P` silently skips the calling process's own
/// ancestors, which makes a seat's agent invisible exactly when the checker runs
/// inside that seat's subtree (verified live: `pgrep -P <pane>` returned nothing
/// while `ps` showed the `pi` child). `ps` with an explicit ppid filter has no
/// such quirk and returns the command names in the same pass, which the agent
/// check needs anyway.
fn pane_children(pane_pid: &str) -> Vec<(String, String)> {
    let out = match Command::new("ps")
        .args(["-ax", "-o", "pid=", "-o", "ppid=", "-o", "comm="])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .filter_map(|line| {
            let mut it = line.split_whitespace();
            let pid = it.next()?;
            let ppid = it.next()?;
            let comm = it.collect::<Vec<_>>().join(" ");
            (ppid == pane_pid).then(|| (pid.to_string(), comm))
        })
        .collect()
}

/// Interrogate the pane: is an agent working in it?
///
/// BOTH the pane's own process and its children are considered. A seat's pane is
/// normally a shell whose child is the agent (`pi`), but a pane created with
/// `tmux new-session <cmd>` runs that command AS the pane process with no shell
/// in between — checking only the children would call a working agent idle.
fn pane_liveness(session: &str, agents: &[String]) -> PaneLiveness {
    let Some(pane_pid) = pane_pid(session) else {
        return PaneLiveness::Unknown;
    };
    // The pane's own command (it may BE the agent when the pane was created
    // with `tmux new-session <cmd>`, with no shell in between).
    let own = Command::new("ps")
        .args(["-o", "comm=", "-p", &pane_pid])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .map(str::trim)
                .filter(|c| !c.is_empty())
                .map(str::to_string)
        });
    let children: Vec<String> = pane_children(&pane_pid)
        .into_iter()
        .map(|(_, c)| c)
        .collect();

    // The agent match considers the pane's own process and then its children.
    for comm in own.iter().chain(children.iter()) {
        // Match on the basename: `comm` may be a full path.
        let base = Path::new(comm)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| comm.clone());
        if agents.iter().any(|a| a == &base) {
            return PaneLiveness::Working(base);
        }
    }
    // No agent anywhere: a pane with nothing under it is idle (its own process
    // is the shell, which is not work). Children that are not agents are
    // reported, so a build or a stuck command is visible rather than assumed.
    if children.is_empty() {
        PaneLiveness::Idle
    } else {
        PaneLiveness::Other(children)
    }
}

/// The first pane's pid of a tmux session.
fn pane_pid(session: &str) -> Option<String> {
    let out = Command::new("tmux")
        .args(["list-panes", "-t", session, "-F", "#{pane_pid}"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

// =============================================================================
// Nudge bookkeeping
// =============================================================================

/// Per-seat last-nudge times, so a sweep never spams a seat.
struct Poker {
    last: HashMap<String, Instant>,
    cooldown: Duration,
}

impl Poker {
    fn new(cooldown: Duration) -> Self {
        Self {
            last: HashMap::new(),
            cooldown,
        }
    }

    /// May this seat be nudged now? Records the nudge when allowed.
    fn allow(&mut self, seat: &str) -> bool {
        let now = Instant::now();
        if let Some(at) = self.last.get(seat) {
            if now.duration_since(*at) < self.cooldown {
                return false;
            }
        }
        self.last.insert(seat.to_string(), now);
        true
    }
}

// =============================================================================
// Reconnect backoff
// =============================================================================

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

// =============================================================================
// main
// =============================================================================

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cfg = Config::from_env();
    let seat_map = SeatMap::load(&cfg.map_path);
    eprintln!(
        "seatloom-waker: {} seat(s) mapped from {}; LISTEN dispatch_ready on {}; \
         stale-after {:?}, sweep every {:?}, poke cooldown {:?}",
        seat_map.seats.len(),
        cfg.map_path,
        cfg.db_url.split('@').next_back().unwrap_or("db"),
        cfg.stale_after,
        cfg.sweep_every,
        cfg.poke_cooldown,
    );

    // Resident daemon: every failure path leads back here. The seat map is
    // reloaded by the sweep (and on an unmapped NOTIFY); the connection is
    // rebuilt as often as it takes.
    let mut backoff = Backoff::new();
    loop {
        let reason = serve_once(&cfg, &mut backoff).await;
        let delay = backoff.next_delay();
        eprintln!("seatloom-waker: {reason} — reconnecting in {delay:?}");
        tokio::time::sleep(delay).await;
    }
}

/// One service lifetime: connect (retrying with backoff — the DB may still be
/// booting), LISTEN, run the catch-up scan, then consume notifications AND
/// periodic sweeps until the connection or a query on it fails. Returns the
/// reason this lifetime ended; the caller backs off and calls again. All
/// connection handles are local, so returning drops them (closing the old
/// connection and ending the old driver task) before the next attempt.
async fn serve_once(cfg: &Config, backoff: &mut Backoff) -> String {
    let (client, mut connection) = loop {
        match tokio_postgres::connect(&cfg.db_url, NoTls).await {
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

    // The map and the poker live for this lifetime; the map is reloaded in place
    // so an unmapped NOTIFY and the sweep see the same state.
    let mut map = SeatMap::load(&cfg.map_path);
    let mut poker = Poker::new(cfg.poke_cooldown);
    let mut ticker = tokio::time::interval(cfg.sweep_every);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Catch-up scan: NOTIFY is fire-and-forget, so anything that went ready
    // while the previous connection was down has no pending notification to
    // receive. Re-derive the missed wakeups from the DB before trusting the
    // stream again.
    if let Err(e) = catch_up(&client, &mut map, &mut poker).await {
        return format!("catch-up scan failed: {}", describe(e));
    }

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(ConnMsg::Notify(n)) => {
                        let seat_id = n.payload().to_string();
                        if let Err(e) = handle_ready(&client, &mut map, &mut poker, &seat_id).await {
                            // The busy-check query failed — the connection is suspect.
                            // Fail safe: never nudge a seat we could not prove idle.
                            // Reconnect; the catch-up scan re-derives this seat's state.
                            return format!("query for {seat_id} failed: {}", describe(e));
                        }
                    }
                    Some(ConnMsg::Lost(r)) => return r,
                    None => return "notification stream closed".to_string(),
                }
            }
            _ = ticker.tick() => {
                // Periodic catch-up + stale-claim detection (fixes 2 and 1).
                if let Err(e) = sweep(&client, &mut map, cfg, &mut poker).await {
                    return format!("sweep failed: {}", describe(e));
                }
            }
        }
    }
}

/// Handle one `dispatch_ready` NOTIFY: poke the seat iff it is idle. A DB error
/// is propagated (the caller reconnects) rather than swallowed — poking after a
/// failed busy check could interrupt a busy seat.
///
/// An unmapped seat triggers ONE map re-read before giving up: a seat added
/// while the daemon runs is picked up without a restart.
async fn handle_ready(
    client: &Client,
    map: &mut SeatMap,
    poker: &mut Poker,
    seat_id: &str,
) -> Result<(), tokio_postgres::Error> {
    if map.get(seat_id).is_none() {
        // Unmapped: re-read the file ONCE — a seat added while the daemon runs
        // must not need a restart — and only then give up.
        if map.reload(false) && map.get(seat_id).is_some() {
            return handle_ready_mapped(client, map, poker, seat_id).await;
        }
        eprintln!("seatloom-waker: NOTIFY for unmapped seat {seat_id} — ignoring");
        return Ok(());
    }
    handle_ready_mapped(client, map, poker, seat_id).await
}

async fn handle_ready_mapped(
    client: &Client,
    map: &SeatMap,
    poker: &mut Poker,
    seat_id: &str,
) -> Result<(), tokio_postgres::Error> {
    let Some(session) = map.get(seat_id) else {
        return Ok(());
    };
    // Busy check: a seat with an in_flight dispatch is executing — never
    // interrupt it; it will drain new work itself at its next boundary.
    if is_busy(client, seat_id).await? {
        eprintln!("seatloom-waker: {seat_id} busy (in_flight) — not poking; it self-drains");
        return Ok(());
    }
    if !poker.allow(seat_id) {
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

// =============================================================================
// The seat state query (one round trip per seat)
// =============================================================================

/// One seat's dispatch state: is it holding an in_flight claim (and for how
/// long), and does it have ready work waiting?
struct SeatState {
    ready_n: i64,
    in_flight_id: Option<String>,
    /// The claim's age IN SECONDS, computed by Postgres from the stored
    /// timestamp — never by this process.
    ///
    /// The database and the daemon can run on different hosts with different
    /// clocks (verified in production: the local Postgres container was ~2h17m
    /// behind the host). Computing `now - stored_at` here would turn that skew
    /// into a bogus age — a fresh claim reading as hours old, or a genuinely
    /// stuck one reading as fresh. Postgres computes the difference against its
    /// own `NOW()`, so both operands share one clock.
    claim_age_secs: Option<f64>,
}

impl SeatState {
    /// Age of the in_flight claim, or `None` when there is no claim or its
    /// timestamp is unknown (an unknown age is never treated as stale).
    fn claim_age(&self) -> Option<Duration> {
        // No claim -> no age, even if a timestamp somehow rode along.
        self.in_flight_id.as_ref()?;
        let secs = self.claim_age_secs?;
        // A negative age means the clocks disagree in the other direction; a
        // claim cannot be from the future, so it is not stale.
        (secs > 0.0).then(|| Duration::from_secs_f64(secs))
    }
}

async fn seat_state(client: &Client, seat_id: &str) -> Result<SeatState, tokio_postgres::Error> {
    let row = client
        .query_one(
            // The claim's timestamp is `dispatches.started_at`, falling back to
            // the `SeatDispatchDelivered` event — the ledger's own record of the
            // handover, for a row written before that column was populated. The
            // AGE is computed here, in the database, against the database's own
            // NOW() (see SeatState::claim_age_secs).
            "SELECT (SELECT count(*) FROM dispatches WHERE target_seat_id = $1 AND status = 'ready'), \
                    (SELECT d.id FROM dispatches d WHERE d.target_seat_id = $1 AND d.status = 'in_flight' LIMIT 1), \
                    (SELECT EXTRACT(EPOCH FROM (NOW() - claimed_at))::float8 \
                       FROM (SELECT COALESCE(d.started_at, \
                                      (SELECT e.occurred_at FROM canonical_events e \
                                         JOIN event_object_refs r ON r.event_id = e.id \
                                        WHERE r.ref_type = 'dispatch' AND r.ref_id = d.id \
                                          AND e.event_type = 'SeatDispatchDelivered' \
                                        ORDER BY e.occurred_at DESC LIMIT 1)) AS claimed_at \
                               FROM dispatches d \
                              WHERE d.target_seat_id = $1 AND d.status = 'in_flight' LIMIT 1) c)",
            &[&seat_id],
        )
        .await?;
    Ok(SeatState {
        ready_n: row.get(0),
        in_flight_id: row.get(1),
        claim_age_secs: row.get(2),
    })
}

// =============================================================================
// Sweep: periodic catch-up + stale-claim detection
// =============================================================================

/// The periodic pass over every mapped seat:
///   - idle + ready work waiting            -> poke (the lost-NOTIFY case);
///   - stale in_flight + pane not working   -> poke (the stuck-claim case);
///   - anything else                        -> leave alone, and say why.
async fn sweep(
    client: &Client,
    map: &mut SeatMap,
    cfg: &Config,
    poker: &mut Poker,
) -> Result<(), tokio_postgres::Error> {
    map.reload(false);
    let seats: Vec<(String, String)> = map
        .iter()
        .map(|(s, sess)| (s.clone(), sess.clone()))
        .collect();
    for (seat_id, session) in seats {
        let state = seat_state(client, &seat_id).await?;
        match (&state.in_flight_id, state.ready_n) {
            // Idle with work waiting: the classic wakeup, now periodic so a
            // NOTIFY that fired while the seat was busy is not lost forever.
            (None, n) if n > 0 => {
                if poker.allow(&seat_id) {
                    eprintln!(
                        "seatloom-waker: sweep: idle seat {seat_id} has {n} ready dispatch(es) -> {session}"
                    );
                    nudge(&session);
                }
            }
            // Busy: is the claim stale, and is the pane actually working?
            (Some(id), _) => match state.claim_age() {
                Some(age) if age >= cfg.stale_after => {
                    let live = pane_liveness(&session, &cfg.agent_procs);
                    if live.is_working() {
                        eprintln!(
                            "seatloom-waker: {seat_id} in_flight {id} for {age:?} ({}) — an agent is working; not poking",
                            live.describe()
                        );
                    } else if poker.allow(&seat_id) {
                        eprintln!(
                            "seatloom-waker: {seat_id} STALE claim: in_flight {id} for {age:?} with {} — poking anyway; \
                             the seat resumes its own task (get_next_dispatch answers 'already in_flight')",
                            live.describe()
                        );
                        nudge(&session);
                    }
                }
                Some(age) => {
                    // Fresh claim: the normal, healthy case.
                    let _ = age;
                }
                None => {
                    eprintln!(
                        "seatloom-waker: {seat_id} in_flight {id} with no claim timestamp — not poking (unknown age)"
                    );
                }
            },
            // Idle with nothing ready: nothing to do.
            (None, _) => {}
        }
    }
    Ok(())
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
    map: &mut SeatMap,
    poker: &mut Poker,
) -> Result<(), tokio_postgres::Error> {
    map.reload(true);
    let mut woken = 0usize;
    let seats: Vec<(String, String)> = map
        .iter()
        .map(|(s, sess)| (s.clone(), sess.clone()))
        .collect();
    for (seat_id, session) in seats {
        let state = seat_state(client, &seat_id).await?;
        if state.in_flight_id.is_none() && state.ready_n > 0 && poker.allow(&seat_id) {
            eprintln!("seatloom-waker: catch-up: waking idle seat {seat_id} -> {session}");
            nudge(&session);
            woken += 1;
        }
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The map parses the header/comment/blank lines away and reloads when the
    /// file's identity changes — the hot-reload primitive, without a DB.
    #[test]
    fn seat_map_parses_and_detects_change() {
        let dir = std::env::temp_dir().join(format!("waker-map-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dispatch-seats.tsv");
        let write = |text: &str| std::fs::write(&path, text).unwrap();

        write("# seat_id\ttmux_session\nseat-a\tSessA\n\n");
        let mut map = SeatMap::load(path.to_str().unwrap());
        assert_eq!(map.get("seat-a"), Some("SessA"));
        assert!(map.get("seat-b").is_none());

        // Unchanged file: no reload.
        assert!(!map.reload(false));

        // A new seat appears: the next sweep picks it up without a restart.
        write("# seat_id\ttmux_session\nseat-a\tSessA\nseat-b\tSessB\n");
        assert!(map.reload(false), "a changed file must be re-read");
        assert_eq!(map.get("seat-b"), Some("SessB"));

        // The file disappears: the previous map is kept, not emptied.
        std::fs::remove_file(&path).unwrap();
        assert!(!map.reload(false));
        assert_eq!(map.get("seat-a"), Some("SessA"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Pane liveness: no children is idle; a recognised agent is working (and the
    /// log names WHICH agent, because the pane may host a different one); other
    /// children are neither. The unknown case (no pane at all) is treated as not
    /// working — we cannot prove activity, and a nudge is safe by design.
    #[test]
    fn pane_liveness_classifies_the_pane() {
        assert!(!PaneLiveness::Idle.is_working());
        assert!(!PaneLiveness::Unknown.is_working());
        assert!(!PaneLiveness::Other(vec!["cargo".into()]).is_working());
        assert!(
            PaneLiveness::Working("gemini".into()).is_working(),
            "a DIFFERENT agent in the pane still counts as working"
        );
        assert!(PaneLiveness::Working("pi".into()).describe().contains("pi"));
        assert!(PaneLiveness::Unknown.describe().contains("unknown"));
    }

    /// A child command is matched on its basename, so a full path still counts.
    #[test]
    fn agent_matching_uses_the_basename() {
        let agents = ["pi".to_string(), "node".to_string()];
        for comm in ["pi", "/opt/homebrew/bin/pi", "node"] {
            let base = Path::new(comm)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| comm.to_string());
            assert!(
                agents.iter().any(|a| a == &base),
                "{comm} should match an agent"
            );
        }
        let base = Path::new("/usr/bin/cargo")
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap();
        assert!(!agents.iter().any(|a| a == &base), "cargo is not an agent");
    }

    /// The nudge cooldown: one nudge per seat per window, and a nudge is
    /// remembered so a sweep never spams.
    #[test]
    fn poker_enforces_the_cooldown() {
        let mut poker = Poker::new(Duration::from_millis(60));
        assert!(poker.allow("seat-a"), "the first nudge is allowed");
        assert!(!poker.allow("seat-a"), "an immediate second one is not");
        assert!(poker.allow("seat-b"), "another seat is unaffected");
        std::thread::sleep(Duration::from_millis(80));
        assert!(
            poker.allow("seat-a"),
            "after the cooldown it is allowed again"
        );
    }

    /// Config defaults, and a nonsense knob never stops the daemon from starting.
    #[test]
    fn config_falls_back_on_nonsense() {
        assert_eq!(
            secs_from_env("SEATLOOM_DEFINITELY_NOT_SET", Duration::from_secs(7)),
            Duration::from_secs(7)
        );
        std::env::set_var("SEATLOOM_TEST_SECS", "not-a-number");
        assert_eq!(
            secs_from_env("SEATLOOM_TEST_SECS", Duration::from_secs(9)),
            Duration::from_secs(9)
        );
        std::env::set_var("SEATLOOM_TEST_SECS", "0");
        assert_eq!(
            secs_from_env("SEATLOOM_TEST_SECS", Duration::from_secs(9)),
            Duration::from_secs(9)
        );
        std::env::remove_var("SEATLOOM_TEST_SECS");
    }

    /// A claim with no timestamp is never treated as stale — an unknown age is
    /// not evidence of a stuck seat.
    #[test]
    fn claim_age_is_none_without_a_timestamp() {
        let state = SeatState {
            ready_n: 0,
            in_flight_id: Some("disp-1".into()),
            claim_age_secs: None,
        };
        assert!(state.claim_age().is_none());
        let fresh = SeatState {
            ready_n: 1,
            in_flight_id: None,
            claim_age_secs: Some(0.0),
        };
        assert!(fresh.claim_age().is_none(), "no claim, no age");
        // A negative age (clock disagreement) is not evidence of a stuck seat.
        let future = SeatState {
            ready_n: 0,
            in_flight_id: Some("disp-1".into()),
            claim_age_secs: Some(-5.0),
        };
        assert!(future.claim_age().is_none(), "a future claim is not stale");
        let old = SeatState {
            ready_n: 0,
            in_flight_id: Some("disp-1".into()),
            claim_age_secs: Some(2400.0),
        };
        assert_eq!(old.claim_age(), Some(Duration::from_secs(2400)));
    }
}
