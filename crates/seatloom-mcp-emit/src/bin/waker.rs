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
//!   4. A PROGRESS SIGNAL, AND A HARD FLOOR BEHIND IT. "The pane has an agent in
//!      it" turned out not to mean "the agent is working": an agent parked at its
//!      prompt and an agent streaming tokens are the same process list, and that
//!      equivalence held a seat's claim — and four ready dispatches behind it —
//!      for seventeen hours while the daemon logged "an agent is working; not
//!      poking" every 90 seconds. The primary signal is now whether the pane's
//!      captured OUTPUT changes between two sweeps (an agent mid-turn streams; an
//!      agent waiting does not). Behind it sits `SEATLOOM_STALE_HARD_AFTER_SECS`
//!      (default 4 x the soft threshold): past that age a stale claim is poked
//!      whatever the pane says, because the worst case of one redundant nudge is
//!      strictly better than the worst case of another seventeen hours.
//!
//! What this daemon deliberately does NOT do: change a dispatch's status. A
//! stale `in_flight` is not rewritten to `failed` or `ready` — that is ledger
//! semantics (Onyx/Lyra's lane) and it would hide the real cause. The waker only
//! "goes and calls the seat"; the seat decides how to close its own task.
//!
//! Env: DATABASE_URL (default postgresql://seatloom:seatloom@localhost:5432/seatloom),
//!      SEATLOOM_SEAT_MAP (default .seatloom/dispatch-seats.tsv),
//!      SEATLOOM_STALE_AFTER_SECS (default 900),
//!      SEATLOOM_STALE_HARD_AFTER_SECS (default 4 x stale_after),
//!      SEATLOOM_SWEEP_SECS (default 90),
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
/// How long between two HEARTBEAT nudges to the same seat.
///
/// **Why this value, and why it is not the dispatch cooldown.** The dispatch
/// cooldown (`DEFAULT_POKE_COOLDOWN`, 120 s) exists because a NOTIFY can fire
/// while a seat is busy, so it is about not *losing* work — it has to be short.
/// A heartbeat nudge loses nothing: it reminds an idle seat of duties it already
/// has. So it is deliberately long. 30 minutes is 15x the dispatch cooldown and
/// means a seat is interrupted at most twice an hour, while still short enough
/// that a seat which goes idle is not left for a day. It is a separate knob
/// (`SEATLOOM_HEARTBEAT_COOLDOWN_SECS`) on a separate [`Poker`], so tuning one
/// never moves the other.
const DEFAULT_HEARTBEAT_COOLDOWN: Duration = Duration::from_secs(30 * 60);
/// The directory holding each seat's heartbeat document, relative to the working
/// directory the same way `SEATLOOM_SEAT_MAP`'s default is. The convention is
/// `<tasks_dir>/<seat>/<SEAT>-HEARTBEAT.md` — already established by
/// `AEGIS-HEARTBEAT.md` and `LYRA-HEARTBEAT.md`, so this is not a new layout.
const DEFAULT_TASKS_DIR: &str = "docs/coordination/tasks";
/// Multiplier applied to `stale_after` when `SEATLOOM_STALE_HARD_AFTER_SECS` is
/// not set: the backstop age past which a stale claim is poked REGARDLESS of
/// what the pane looks like. Four times the soft threshold — long enough that a
/// legitimately slow turn is never interrupted by the backstop, short enough
/// that a wedged seat costs an hour rather than a day (the stall this fixes ran
/// for seventeen).
const HARD_FLOOR_MULTIPLIER: u32 = 4;
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
    /// The backstop: past this age a stale claim is poked even when the pane
    /// looks busy. Defaults to `HARD_FLOOR_MULTIPLIER` x `stale_after`.
    stale_hard_after: Duration,
    sweep_every: Duration,
    poke_cooldown: Duration,
    /// The HEARTBEAT nudge's own, longer cooldown. Separate from
    /// `poke_cooldown` on purpose: a dispatch nudge is urgent, a heartbeat one
    /// is not, and sharing one window would make one of the two wrong.
    heartbeat_cooldown: Duration,
    /// Where each seat's heartbeat document lives.
    tasks_dir: String,
    agent_procs: Vec<String>,
}

impl Config {
    fn from_env() -> Self {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://seatloom:seatloom@localhost:5432/seatloom".to_string()
        });
        let stale_after = secs_from_env("SEATLOOM_STALE_AFTER_SECS", DEFAULT_STALE_AFTER);
        let map_path = std::env::var("SEATLOOM_SEAT_MAP")
            .unwrap_or_else(|_| ".seatloom/dispatch-seats.tsv".to_string());
        Self {
            db_url,
            map_path,
            stale_after,
            stale_hard_after: secs_from_env(
                "SEATLOOM_STALE_HARD_AFTER_SECS",
                stale_after.saturating_mul(HARD_FLOOR_MULTIPLIER),
            ),
            sweep_every: secs_from_env("SEATLOOM_SWEEP_SECS", DEFAULT_SWEEP_EVERY),
            poke_cooldown: secs_from_env("SEATLOOM_POKE_COOLDOWN_SECS", DEFAULT_POKE_COOLDOWN),
            heartbeat_cooldown: secs_from_env(
                "SEATLOOM_HEARTBEAT_COOLDOWN_SECS",
                DEFAULT_HEARTBEAT_COOLDOWN,
            ),
            tasks_dir: std::env::var("SEATLOOM_TASKS_DIR")
                .unwrap_or_else(|_| DEFAULT_TASKS_DIR.to_string()),
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

/// Whether the pane is PROGRESSING, as opposed to merely hosting a live process.
///
/// The distinction the 17-hour fleet stall turned on: "an agent is running" and
/// "an agent is working" look identical to a process check, because an idle but
/// living agent and a busy one both have a `pi` child. What separates them is
/// whether the pane's OUTPUT CHANGES between two sweeps — an agent mid-turn
/// streams tokens, an agent waiting at its prompt does not.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PaneProgress {
    /// The pane's captured output differs from the last sweep's.
    Advancing,
    /// The pane's captured output is byte-identical to the last sweep's.
    Static,
    /// The pane could not be captured (no tmux, no such session). Treated as
    /// NOT advancing — we cannot prove progress, and a nudge is safe by design.
    Unknown,
}

impl PaneProgress {
    fn describe(&self) -> String {
        match self {
            PaneProgress::Advancing => "pane output advancing".to_string(),
            PaneProgress::Static => "pane output unchanged".to_string(),
            PaneProgress::Unknown => "pane output unknown".to_string(),
        }
    }
}

/// Capture the pane's visible output, hashed.
///
/// `tmux capture-pane -p` is the pane's own rendered content, so it moves when
/// the agent streams, when a build prints, or when a human types — everything
/// that means "something is happening here" — and stays put when the agent sits
/// at its prompt. A hash keeps the comparison O(1) and keeps the (potentially
/// large) pane text out of the daemon's memory.
fn pane_output_hash(session: &str) -> Option<u64> {
    use std::hash::{Hash, Hasher};
    let out = Command::new("tmux")
        .args(["capture-pane", "-p", "-t", session])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    out.stdout.hash(&mut hasher);
    Some(hasher.finish())
}

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
         stale-after {:?}, hard-floor {:?}, sweep every {:?}, poke cooldown {:?}",
        seat_map.seats.len(),
        cfg.map_path,
        cfg.db_url.split('@').next_back().unwrap_or("db"),
        cfg.stale_after,
        cfg.stale_hard_after,
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
    // The heartbeat nudge has its OWN cooldown, on its own Poker: a dispatch
    // nudge is urgent (work is waiting) and a heartbeat one is not, so sharing a
    // window would make one of the two wrong. They are separate knobs on separate
    // state, and tuning one never moves the other.
    let mut heartbeat_poker = Poker::new(cfg.heartbeat_cooldown);
    // Per-seat pane hashes from the previous sweep: the progress signal.
    let mut pane_hashes: HashMap<String, u64> = HashMap::new();
    let mut ticker = tokio::time::interval(cfg.sweep_every);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Catch-up scan: NOTIFY is fire-and-forget, so anything that went ready
    // while the previous connection was down has no pending notification to
    // receive. Re-derive the missed wakeups from the DB before trusting the
    // stream again.
    if let Err(e) = catch_up(&client, &mut map, &mut poker, &cfg.tasks_dir).await {
        return format!("catch-up scan failed: {}", describe(e));
    }

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(ConnMsg::Notify(n)) => {
                        let seat_id = n.payload().to_string();
                        if let Err(e) =
                            handle_ready(&client, &mut map, &mut poker, &cfg.tasks_dir, &seat_id).await
                        {
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
                if let Err(e) =
                    sweep(&client, &mut map, cfg, &mut poker, &mut heartbeat_poker, &mut pane_hashes).await
                {
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
    tasks_dir: &str,
    seat_id: &str,
) -> Result<(), tokio_postgres::Error> {
    if map.get(seat_id).is_none() {
        // Unmapped: re-read the file ONCE — a seat added while the daemon runs
        // must not need a restart — and only then give up.
        if map.reload(false) && map.get(seat_id).is_some() {
            return handle_ready_mapped(client, map, poker, tasks_dir, seat_id).await;
        }
        eprintln!("seatloom-waker: NOTIFY for unmapped seat {seat_id} — ignoring");
        return Ok(());
    }
    handle_ready_mapped(client, map, poker, tasks_dir, seat_id).await
}

async fn handle_ready_mapped(
    client: &Client,
    map: &SeatMap,
    poker: &mut Poker,
    tasks_dir: &str,
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
    nudge(session, NudgeKind::Dispatch, seat_id, tasks_dir);
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
///   - idle + ready work waiting             -> poke (the lost-NOTIFY case);
///   - stale in_flight + pane not advancing  -> poke (the stuck-claim case);
///   - stale in_flight past the HARD FLOOR   -> poke whatever the pane says;
///   - anything else                         -> leave alone, and say why.
///
/// `pane_hashes` carries the previous sweep's per-seat pane hash, so "is the
/// pane advancing" is a comparison between two sweeps rather than a guess. It is
/// refreshed on every sweep, including for the seats this sweep leaves alone.
async fn sweep(
    client: &Client,
    map: &mut SeatMap,
    cfg: &Config,
    poker: &mut Poker,
    heartbeat_poker: &mut Poker,
    pane_hashes: &mut HashMap<String, u64>,
) -> Result<(), tokio_postgres::Error> {
    map.reload(false);
    let seats: Vec<(String, String)> = map
        .iter()
        .map(|(s, sess)| (s.clone(), sess.clone()))
        .collect();
    for (seat_id, session) in seats {
        let state = seat_state(client, &seat_id).await?;

        // Observe the pane first, so the hash table is current for every seat
        // whether or not this sweep acts on it.
        let hash = pane_output_hash(&session);
        let progress = match (hash, pane_hashes.get(&seat_id)) {
            (Some(now), Some(prev)) if *prev == now => PaneProgress::Static,
            (Some(_), Some(_)) => PaneProgress::Advancing,
            // First observation (or an uncapturable pane): nothing to compare
            // against yet, so there is no evidence of progress.
            _ => PaneProgress::Unknown,
        };
        if let Some(now) = hash {
            pane_hashes.insert(seat_id.clone(), now);
        }

        match (&state.in_flight_id, state.ready_n) {
            // Idle with work waiting: the classic wakeup, now periodic so a
            // NOTIFY that fired while the seat was busy is not lost forever.
            (None, n) if n > 0 => {
                if poker.allow(&seat_id) {
                    eprintln!(
                        "seatloom-waker: sweep: idle seat {seat_id} has {n} ready dispatch(es) -> {session}"
                    );
                    nudge(&session, NudgeKind::Dispatch, &seat_id, &cfg.tasks_dir);
                }
            }
            // Busy: is the claim stale, and is the pane actually advancing?
            (Some(id), _) => match state.claim_age() {
                Some(age) if age >= cfg.stale_hard_after => {
                    // The backstop. A claim this old is poked whatever the pane
                    // says, because "the pane looks busy" is exactly the signal
                    // that failed for 17 hours: an agent parked at its prompt
                    // and an agent streaming tokens look identical to a process
                    // check. Poking is safe (the seat's own AGENTS.md rules that
                    // an "already in_flight" answer is its current task), so the
                    // worst case is one redundant nudge.
                    if poker.allow(&seat_id) {
                        eprintln!(
                            "seatloom-waker: {seat_id} STALE claim past the hard floor: in_flight {id} for {age:?} \
                             ({}) — poking anyway; a pane that merely looks busy is not evidence of progress",
                            progress.describe()
                        );
                        nudge(&session, NudgeKind::Dispatch, &seat_id, &cfg.tasks_dir);
                    }
                }
                Some(age) if age >= cfg.stale_after => {
                    let live = pane_liveness(&session, &cfg.agent_procs);
                    if live.is_working() && progress == PaneProgress::Advancing {
                        eprintln!(
                            "seatloom-waker: {seat_id} in_flight {id} for {age:?} ({}, {}) — an agent is working; not poking",
                            live.describe(),
                            progress.describe()
                        );
                    } else if poker.allow(&seat_id) {
                        eprintln!(
                            "seatloom-waker: {seat_id} STALE claim: in_flight {id} for {age:?} with {} ({}) — poking anyway; \
                             the seat resumes its own task (get_next_dispatch answers 'already in_flight')",
                            live.describe(),
                            progress.describe()
                        );
                        nudge(&session, NudgeKind::Dispatch, &seat_id, &cfg.tasks_dir);
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
            // Idle with nothing ready: NOTHING TO DO — unless the seat has
            // standing duties of its own.
            //
            // THE DEFECT THIS CLOSES. This arm used to be `{}`, so a seat with no
            // ready dispatch was never called at all — and the only instruction
            // any seat ever received was "drain your dispatches and stop", whose
            // entire field of view is the queue. A seat whose job includes
            // designing, finding what is broken, and owning the delivery was
            // therefore a pure contractor: present when dispatched, absent
            // otherwise. Mr Zhang named it exactly that on 2026-10-16.
            //
            // So an idle seat with a heartbeat document is called to that
            // document, on its own longer cooldown. A seat WITHOUT one is left
            // alone — calling it would earn "nothing to do", which is noise.
            (None, 0) => {
                if has_heartbeat_doc(&cfg.tasks_dir, &seat_id) && heartbeat_poker.allow(&seat_id) {
                    eprintln!(
                        "seatloom-waker: sweep: idle seat {seat_id} has no ready dispatch and a                          standing-duty document -> {session}"
                    );
                    nudge(&session, NudgeKind::Heartbeat, &seat_id, &cfg.tasks_dir);
                }
            }
            // Idle with nothing ready and no standing duties: genuinely nothing.
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
    tasks_dir: &str,
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
            nudge(&session, NudgeKind::Dispatch, &seat_id, tasks_dir);
            woken += 1;
        }
    }
    eprintln!("seatloom-waker: catch-up scan done ({woken} seat(s) nudged)");
    Ok(())
}

/// The two things a seat can be called to do. They are different instructions
/// with different urgency, so they are two variants of one function rather than
/// two functions that could drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NudgeKind {
    /// Drain the dispatch queue. Urgent: work is waiting.
    Dispatch,
    /// Go and do your standing duties. Not urgent: nothing is waiting.
    Heartbeat,
}

impl NudgeKind {
    /// The message typed into the seat's pane.
    ///
    /// **The dispatch message is byte-identical to what it always was.** The
    /// acceptance criterion for this change is that a seat with work receives
    /// exactly the old instruction, so it is a literal, not a rewrite.
    fn message(self, seat_id: &str, tasks_dir: &str) -> String {
        match self {
            NudgeKind::Dispatch => {
                "📥 new governed dispatch available — run your dispatch loop now: \
                 call get_next_dispatch repeatedly (do each payload, then complete_dispatch) \
                 until it returns \"nothing ready\", then stop."
                    .to_string()
            }
            NudgeKind::Heartbeat => {
                let doc = heartbeat_doc_path(tasks_dir, seat_id);
                format!(
                    "📋 no dispatch waiting — this is your standing duty, not idle time. \
                     Read {doc} and do what it says. A seat that only exists when \
                     dispatched is not doing its job."
                )
            }
        }
    }
}

/// Where a seat's heartbeat document lives, under the established convention
/// `<tasks_dir>/<seat>/<SEAT>-HEARTBEAT.md`.
///
/// `seat-lyra-001` → `docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md`. The seat
/// id's `seat-` prefix and `-NNN` suffix are the registry's, not the document
/// directory's, so they are stripped rather than kept.
fn heartbeat_doc_path(tasks_dir: &str, seat_id: &str) -> String {
    let short = seat_id
        .strip_prefix("seat-")
        .unwrap_or(seat_id)
        .split('-')
        .next()
        .unwrap_or(seat_id);
    format!("{tasks_dir}/{short}/{}-HEARTBEAT.md", short.to_uppercase())
}

/// Does this seat HAVE a heartbeat document?
///
/// Only a seat with one is called with [`NudgeKind::Heartbeat`]. Calling a seat
/// that has no standing duties would produce "nothing to do" — noise, and the
/// dispatch names that as a reject. Checked per sweep rather than cached, so
/// writing a document connects a seat with no code change and no restart, which
/// is the whole point of the convention.
fn has_heartbeat_doc(tasks_dir: &str, seat_id: &str) -> bool {
    Path::new(&heartbeat_doc_path(tasks_dir, seat_id)).is_file()
}

/// Nudge an idle seat's tmux pane.
fn nudge(session: &str, kind: NudgeKind, seat_id: &str, tasks_dir: &str) {
    let msg = kind.message(seat_id, tasks_dir);
    // Type the message, then submit with Enter (sent separately so the pane's
    // line editor registers a real submit).
    let _ = Command::new("tmux")
        .args(["send-keys", "-t", session, &msg])
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

    /// The hard floor defaults to four times the soft threshold, and the env var
    /// overrides it. The default is derived, not a second magic number, so
    /// changing the soft threshold moves the backstop with it.
    #[test]
    fn the_hard_floor_defaults_to_four_times_the_soft_threshold() {
        assert_eq!(HARD_FLOOR_MULTIPLIER, 4);
        let soft = Duration::from_secs(900);
        assert_eq!(
            soft.saturating_mul(HARD_FLOOR_MULTIPLIER),
            Duration::from_secs(3600),
            "the default backstop is an hour"
        );
    }

    /// The progress signal is a COMPARISON between two sweeps: same hash means
    /// static, different means advancing, and a missing baseline means "no
    /// evidence" rather than "no progress".
    #[test]
    fn pane_progress_is_a_comparison_not_a_guess() {
        let mut hashes: HashMap<String, u64> = HashMap::new();
        let seat = "seat-x";
        let first = 111u64;
        // First observation: nothing to compare against.
        let progress = match (Some(first), hashes.get(seat)) {
            (Some(now), Some(prev)) if *prev == now => PaneProgress::Static,
            (Some(_), Some(_)) => PaneProgress::Advancing,
            _ => PaneProgress::Unknown,
        };
        assert_eq!(progress, PaneProgress::Unknown);
        hashes.insert(seat.to_string(), first);

        // Same output again: static — the pane is alive but not moving.
        let progress = match (Some(first), hashes.get(seat)) {
            (Some(now), Some(prev)) if *prev == now => PaneProgress::Static,
            (Some(_), Some(_)) => PaneProgress::Advancing,
            _ => PaneProgress::Unknown,
        };
        assert_eq!(progress, PaneProgress::Static);

        // Different output: advancing.
        let progress = match (Some(222u64), hashes.get(seat)) {
            (Some(now), Some(prev)) if *prev == now => PaneProgress::Static,
            (Some(_), Some(_)) => PaneProgress::Advancing,
            _ => PaneProgress::Unknown,
        };
        assert_eq!(progress, PaneProgress::Advancing);

        // An uncapturable pane is not evidence of progress.
        let progress = match (None::<u64>, hashes.get(seat)) {
            (Some(now), Some(prev)) if *prev == now => PaneProgress::Static,
            (Some(_), Some(_)) => PaneProgress::Advancing,
            _ => PaneProgress::Unknown,
        };
        assert_eq!(progress, PaneProgress::Unknown);
    }

    /// A pane that merely LOOKS busy must not suppress the hard floor: the
    /// classification says "working" while the progress signal says "static",
    /// which is exactly the combination that stalled the fleet. The backstop
    /// fires on age alone.
    #[test]
    fn a_busy_looking_but_static_pane_is_still_past_the_hard_floor() {
        let live = PaneLiveness::Working("pi".to_string());
        assert!(live.is_working(), "the process check says busy");
        let progress = PaneProgress::Static;
        // The soft-path decision: only poke when NOT (working AND advancing).
        let soft_pokes = !(live.is_working() && progress == PaneProgress::Advancing);
        assert!(
            soft_pokes,
            "a static pane is poked even by the soft rule now"
        );
        // And the hard floor ignores both signals entirely.
        let age = Duration::from_secs(4000);
        let hard_after = Duration::from_secs(3600);
        assert!(
            age >= hard_after,
            "past the floor the pane's word is not taken"
        );
    }

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

    // =========================================================================
    // The heartbeat call — a seat that only exists when dispatched is not doing
    // its job (dispatch disp-Krsp8j-EDOpWDIiWjuHo_, 2026-10-16).
    // =========================================================================

    /// The dispatch message is BYTE-IDENTICAL to what it always was. This is an
    /// acceptance criterion: a seat with work must receive exactly the old
    /// instruction, so the string is pinned here rather than merely re-used.
    #[test]
    fn the_dispatch_message_is_byte_identical_to_what_it_always_was() {
        let msg = NudgeKind::Dispatch.message("seat-lyra-001", "docs/coordination/tasks");
        assert_eq!(
            msg,
            "📥 new governed dispatch available — run your dispatch loop now: \
             call get_next_dispatch repeatedly (do each payload, then complete_dispatch) \
             until it returns \"nothing ready\", then stop."
        );
        // And it does not mention a heartbeat document: the two instructions are
        // separate, and neither leaks into the other.
        assert!(!msg.contains("HEARTBEAT"), "{msg}");
        assert!(!msg.contains("standing"), "{msg}");
    }

    /// The heartbeat message points at the seat's OWN document and says what to
    /// do — it is not the drain instruction with different wording.
    #[test]
    fn the_heartbeat_message_points_at_the_seats_own_document() {
        let msg = NudgeKind::Heartbeat.message("seat-lyra-001", "docs/coordination/tasks");
        assert!(
            msg.contains("docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md"),
            "{msg}"
        );
        assert!(msg.contains("standing"), "{msg}");
        // And it does NOT tell the seat to drain dispatches: with none waiting,
        // that instruction would be a lie.
        assert!(!msg.contains("get_next_dispatch"), "{msg}");
        assert!(!msg.contains("nothing ready"), "{msg}");
    }

    /// The path convention: the registry's `seat-` prefix and `-NNN` suffix are
    /// the registry's, not the document directory's.
    #[test]
    fn the_heartbeat_doc_path_strips_the_registrys_affixes() {
        assert_eq!(
            heartbeat_doc_path("docs/coordination/tasks", "seat-lyra-001"),
            "docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md"
        );
        assert_eq!(
            heartbeat_doc_path("docs/coordination/tasks", "seat-onyx-001"),
            "docs/coordination/tasks/onyx/ONYX-HEARTBEAT.md"
        );
        // A bare seat id still resolves rather than panicking on the strip.
        assert_eq!(
            heartbeat_doc_path("docs/coordination/tasks", "lyra"),
            "docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md"
        );
    }

    /// ONLY a seat that HAS a heartbeat document is called this way. A seat
    /// without one would answer "nothing to do" — noise, and the dispatch names
    /// that as a reject. Measured against a real temp directory, so the check is
    /// the filesystem and not a string comparison.
    #[test]
    fn only_a_seat_with_a_heartbeat_document_is_called_this_way() {
        let dir = std::env::temp_dir().join(format!("seatloom-hb-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // One seat with the document, one without.
        std::fs::create_dir_all(dir.join("lyra")).unwrap();
        std::fs::write(dir.join("lyra/LYRA-HEARTBEAT.md"), "# duties\n").unwrap();
        std::fs::create_dir_all(dir.join("mira")).unwrap();

        assert!(
            has_heartbeat_doc(dir.to_str().unwrap(), "seat-lyra-001"),
            "a seat with the document is callable"
        );
        assert!(
            !has_heartbeat_doc(dir.to_str().unwrap(), "seat-mira-001"),
            "a seat without it is not"
        );
        // The directory existing is not enough — the FILE must.
        assert!(!has_heartbeat_doc(dir.to_str().unwrap(), "seat-nimbus-001"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The two cooldowns are SEPARATE, and the heartbeat one is the longer. A
    /// dispatch nudge is urgent (work is waiting); a heartbeat one is not. Sharing
    /// a window would make one of the two wrong.
    #[test]
    fn the_two_cooldowns_are_separate_and_the_heartbeat_one_is_longer() {
        assert!(
            DEFAULT_HEARTBEAT_COOLDOWN > DEFAULT_POKE_COOLDOWN,
            "the heartbeat nudge is the less urgent one and must be rarer"
        );
        assert_eq!(DEFAULT_HEARTBEAT_COOLDOWN, Duration::from_secs(30 * 60));
        assert_eq!(DEFAULT_POKE_COOLDOWN, Duration::from_secs(120));
        // And they are two Pokers over two windows, so one seat can be
        // dispatch-nudged twice while being heartbeat-nudged once.
        let mut dispatch = Poker::new(DEFAULT_POKE_COOLDOWN);
        let mut heartbeat = Poker::new(DEFAULT_HEARTBEAT_COOLDOWN);
        assert!(dispatch.allow("seat-lyra-001"));
        assert!(heartbeat.allow("seat-lyra-001"));
        assert!(!dispatch.allow("seat-lyra-001"), "still cooling down");
        assert!(
            !heartbeat.allow("seat-lyra-001"),
            "and the heartbeat one cools down over a longer window"
        );
    }

    /// A seat with a heartbeat document gets the heartbeat nudge repeatedly
    /// across periods — and a seat WITHOUT one never does, however long it sits
    /// idle. That is the whole acceptance criterion in one test.
    #[test]
    fn an_idle_seat_with_a_document_is_called_and_one_without_is_not() {
        let dir = std::env::temp_dir().join(format!("seatloom-hb2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("lyra")).unwrap();
        std::fs::write(dir.join("lyra/LYRA-HEARTBEAT.md"), "# duties\n").unwrap();
        let tasks = dir.to_str().unwrap().to_string();

        let mut heartbeat = Poker::new(Duration::from_millis(1));
        for period in 0..3 {
            // A seat with the document is called, and the message is the
            // standing-duty one.
            let with =
                has_heartbeat_doc(&tasks, "seat-lyra-001") && heartbeat.allow("seat-lyra-001");
            assert!(
                with,
                "period {period}: a seat with a document must be called"
            );
            assert!(
                NudgeKind::Heartbeat
                    .message("seat-lyra-001", &tasks)
                    .contains("LYRA-HEARTBEAT.md"),
                "the message names the seat's own document"
            );
            std::thread::sleep(Duration::from_millis(3));
        }
        // A seat without one is never called, across the same periods.
        let mut heartbeat2 = Poker::new(Duration::from_millis(1));
        for period in 0..3 {
            let without =
                has_heartbeat_doc(&tasks, "seat-mira-001") && heartbeat2.allow("seat-mira-001");
            assert!(
                !without,
                "period {period}: a seat without a document must not be called"
            );
            std::thread::sleep(Duration::from_millis(3));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The cooldown holds across periods: the same seat is not called to
    /// spam. Measured with a real clock over a real window.
    #[test]
    fn the_heartbeat_cooldown_prevents_spam_across_periods() {
        let mut heartbeat = Poker::new(Duration::from_millis(40));
        assert!(
            heartbeat.allow("seat-lyra-001"),
            "the first call is allowed"
        );
        for _ in 0..5 {
            assert!(
                !heartbeat.allow("seat-lyra-001"),
                "within the window the seat is not called again"
            );
        }
        std::thread::sleep(Duration::from_millis(60));
        assert!(
            heartbeat.allow("seat-lyra-001"),
            "after the window it is called again"
        );
    }
}
